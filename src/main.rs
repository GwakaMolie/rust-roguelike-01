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

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 45;

pub const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
pub const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};

//parameters for dungeon generator
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

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

    /// move by the given amount
    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {
            self.x += dx;
            self.y += dy;
        }
    }
    /// set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.glyph, BackgroundFlag::None);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub blocked: bool,
    pub block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        return Tile {
            blocked: false,
            block_sight: false,
        };
    }

    pub fn wall() -> Self {
        return Tile {
            blocked: true,
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
        // go through the tiles in the rectangle and make them passable
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
        // returns true if this rectangle intersects with another one
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }

    fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
        // horizontal tunnel. `min()` and `max()` are used in case `x1 > x2`
        for x in std::cmp::min(x1, x2)..(std::cmp::max(x1, x2) + 1) {
            map[x as usize][y as usize] = Tile::empty();
        }
    }

    fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
        // vertical tunnel
        for y in std::cmp::min(y1, y2)..(std::cmp::max(y1, y2) + 1) {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

pub fn make_map(player: &mut Object) -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        // random width and height
        let mut rng = rand::thread_rng();

        let w = rng.gen_range(ROOM_MIN_SIZE..=ROOM_MAX_SIZE);
        let h = rng.gen_range(ROOM_MIN_SIZE..=ROOM_MAX_SIZE);
        // random position without going out of the boundaries of the map
        let x = rng.gen_range(0..MAP_WIDTH - w);
        let y = rng.gen_range(0..MAP_HEIGHT - h);

        let new_room = RoomRect::new(x, y, w, h);

        // run through the other rooms and see if they intersect with this one
        //iter iterates over refs of each item in rooms PRETTY COOL
        let does_room_overlap = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !does_room_overlap {
            // this means there are no intersections, so this room is valid

            // "paint" it to the map's tiles
            RoomRect::create_room(new_room, &mut map);

            // center coordinates of the new room, will be useful later
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                // this is the first room, where the player starts at
                player.x = new_x;
                player.y = new_y;
            } else {
                // all rooms after the first:
                // connect it to the previous room with a tunnel
                // center coordinates of the previous room
                // since the current room is the last room in the vec len()-1 is the previous
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
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    for obj in objects {
        obj.draw(&mut tcod.con);
    }

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].block_sight;
            if wall {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
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
        // return exit value as true if escape is pressed
        Key { code: Escape, .. } => return true,
        _ => {}
    }
    return false;
}

fn main() {
    let player = Object::new(25, 23, '@', WHITE);
    let mut game_obj_list = [player];

    let game = Game {
        map: make_map(&mut game_obj_list[0]),
    };

    // workaround ownership

    // init the root console
    let root: Root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("First Rogue-like")
        .init();

    // a console that blits onto the root tcod
    let con: Offscreen = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    // all the used tcods
    let mut tcod = Tcod { root, con };

    tcod::system::set_fps(LIMIT_FPS);

    while !tcod.root.window_closed() {
        tcod.con.clear();

        render_all(&mut tcod, &game, &game_obj_list);

        tcod.root.flush();
        let player = &mut game_obj_list[0];
        let exit = handle_keys(&mut tcod, &game, player);
        if exit {
            break;
        }
    }
}
