use tcod::colors::*;
use tcod::console::*;

use tcod::input::Key;
use tcod::input::KeyCode::*;

mod map;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

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

pub struct Game {
    map: map::Map,
}

struct Tcod {
    root: Root,
    con: Offscreen,
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    for obj in objects {
        obj.draw(&mut tcod.con);
    }

    for y in 0..map::MAP_HEIGHT {
        for x in 0..map::MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].block_sight;
            if wall {
                tcod.con
                    .set_char_background(x, y, map::COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.con
                    .set_char_background(x, y, map::COLOR_DARK_GROUND, BackgroundFlag::Set);
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
    let game = Game {
        map: map::make_map(),
    };

    let player: Object = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', WHITE);

    let npc: Object = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, 'N', RED);

    let mut game_obj_list = [player, npc];
    // workaround ownership

    // init the root console
    let root: Root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("First Rogue-like")
        .init();

    let con = Offscreen::new(map::MAP_WIDTH, map::MAP_HEIGHT);

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
