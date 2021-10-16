use tcod::colors::*;
use tcod::console::*;

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 45;

pub const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
pub const COLOR_DARK_GROUND: Color = Color {
	r: 50,
	g: 50,
	b: 150,
};

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

pub fn make_map() -> Map {
	// fill map with "unblocked" tiles
	let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
	map[30][22] = Tile::wall();
	map[50][22] = Tile::wall();
	return map;
}