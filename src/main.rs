/*
    TODO:
    - sort out this spaghetti mess
    - separate structs into file
    - idk how to do that any of the above in rust >D.
*/
use rand::Rng;
use std;

use tcod::colors::*;
use tcod::console::*;

use tcod::input::Key;
use tcod::input::KeyCode::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 45;

pub const COLOR_DARK_WALL: Color = Color {
    r: 40,
    g: 40,
    b: 40,
};
pub const COLOR_DARK_GROUND: Color = Color {
    r: 70,
    g: 70,
    b: 70,
};
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // default FOV algorithm
const FOV_LIGHT_WALLS: bool = true; // light walls or not
const TORCH_RADIUS: i32 = 10;

pub struct Object {
    x: i32,
    y: i32,
    color: Color,
    glyph: char,
}

impl Object {
    pub fn new(x: i32, y: i32, glyph: char, color: Color) -> Self {
        Object { x, y, glyph, color }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {
            self.x += dx;
            self.y += dy;
        }
    }
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.glyph, BackgroundFlag::None);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    blocked: bool,
    explored: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        return Tile {
            blocked: false,
            explored: false,
            block_sight: false,
        };
    }

    pub fn wall() -> Self {
        return Tile {
            blocked: true,
            explored: false,
            block_sight: true,
        };
    }
}

pub type Map = Vec<Vec<Tile>>;

#[derive(Clone, Copy, Debug)]
pub struct RoomRect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl RoomRect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        RoomRect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    fn create_room(room: RoomRect, map: &mut Map) {
        for x in room.x1..=room.x2 {
            for y in room.y1..=room.y2 {
                map[x as usize][y as usize] = Tile::empty();
            }
        }
    }

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn intersects_with(&self, other: &RoomRect) -> bool {
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }

    fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
        for x in std::cmp::min(x1, x2)..(std::cmp::max(x1, x2) + 1) {
            map[x as usize][y as usize] = Tile::empty();
        }
    }

    fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
        for y in std::cmp::min(y1, y2)..(std::cmp::max(y1, y2) + 1) {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

pub fn make_map(player: &mut Object) -> Map {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        let mut rng = rand::thread_rng();

        let w = rng.gen_range(ROOM_MIN_SIZE..=ROOM_MAX_SIZE);
        let h = rng.gen_range(ROOM_MIN_SIZE..=ROOM_MAX_SIZE);
        let x = rng.gen_range(0..MAP_WIDTH - w);
        let y = rng.gen_range(0..MAP_HEIGHT - h);

        let new_room = RoomRect::new(x, y, w, h);

        let does_room_overlap = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !does_room_overlap {
            RoomRect::create_room(new_room, &mut map);

            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                player.x = new_x;
                player.y = new_y;
            } else {
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                RoomRect::create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                RoomRect::create_v_tunnel(prev_y, new_y, new_x, &mut map);
            }
            rooms.push(new_room);
        }
    }
    return map;
}

pub struct Game {
    map: Map,
}

struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap,
}

fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
    if fov_recompute {
        // recompute FOV if needed (the player moved or something)
        let player = &objects[0];
        tcod.fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = game.map[x as usize][y as usize].block_sight;

            let explored = &mut game.map[x as usize][y as usize].explored;

            let color = match (visible, wall) {
                // outside of field of view:
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                // inside fov:
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };
            if visible {
                // since it's visible, explore it
                *explored = true;
            }
            if *explored {
                tcod.con
                    .set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }

        for obj in objects {
            if tcod.fov.is_in_fov(obj.x, obj.y) {
                obj.draw(&mut tcod.con);
            }
        }
    }
    blit(
        &(tcod.con),
        (0, 0),
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        &mut (tcod.root),
        (0, 0),
        1.0,
        1.0,
    );
}

fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object) -> bool {
    let key = tcod.root.wait_for_keypress(true);

    match key {
        Key { code: Up, .. } => Object::move_by(player, 0, -1, game),
        Key { code: Down, .. } => Object::move_by(player, 0, 1, game),
        Key { code: Left, .. } => Object::move_by(player, -1, 0, game),
        Key { code: Right, .. } => Object::move_by(player, 1, 0, game),
        Key { code: Escape, .. } => return true,
        _ => {}
    }
    return false;
}

fn main() {
    let player = Object::new(25, 23, '@', WHITE);
    let mut game_obj_list = [player];

    let mut game = Game {
        map: make_map(&mut game_obj_list[0]),
    };

    let root: Root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("First Rogue-like")
        .init();

    let mut tcod = Tcod {
        root,
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    };

    tcod::system::set_fps(LIMIT_FPS);

    // populate the FOV map, according to the generated map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }

    // force FOV "recompute" first time through the game loop
    let mut previous_player_position = (-1, -1);

    // main game loop
    while !tcod.root.window_closed() {
        tcod.con.clear();

        // recompute only if the player has moved
        let fov_recompute = previous_player_position != (game_obj_list[0].x, game_obj_list[0].y);

        // render the screen
        render_all(&mut tcod, &mut game, &game_obj_list, fov_recompute);

        tcod.root.flush();
        let player = &mut game_obj_list[0];
        previous_player_position = (player.x, player.y);
        let exit = handle_keys(&mut tcod, &game, player);
        if exit {
            break;
        }
    }
}
