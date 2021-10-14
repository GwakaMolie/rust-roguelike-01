use tcod::colors::*;
use tcod::console::*;

use tcod::input::Key;
use tcod::input::KeyCode::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const LIMIT_FPS: i32 = 20;

mod object;

struct Tcod {
    root: Root,
    con: Offscreen,
}

fn handle_keys(tcod: &mut Tcod, player: &mut object::Object) -> bool {
    let key = tcod.root.wait_for_keypress(true);

    match key {
        Key { code: Up, .. } => object::Object::move_by(player, 0, -1),
        Key { code: Down, .. } => object::Object::move_by(player, 0, 1),
        Key { code: Left, .. } => object::Object::move_by(player, -1, 0),
        Key { code: Right, .. } => object::Object::move_by(player, 1, 0),
        // return exit value as true if escape is pressed
        Key { code: Escape, .. } => return true,
        _ => {}
    }
    return false;
}

fn main() {
    let player: object::Object =
        object::Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', WHITE);

    let npc: object::Object = object::Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, 'N', RED);

    let mut game_obj_list = [player, npc];
    // workaround ownership

    // init the root console
    let root: Root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("First Rogue-like")
        .init();

    let con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    let mut tcod = Tcod { root, con };

    tcod::system::set_fps(LIMIT_FPS);

    while !tcod.root.window_closed() {
        tcod.con.clear();

        for obj in &game_obj_list {
            obj.draw(&mut tcod.con);
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

        tcod.root.flush();
        let player = &mut game_obj_list[0];
        let exit = handle_keys(&mut tcod, player);
        if exit {
            break;
        }
    }
}
