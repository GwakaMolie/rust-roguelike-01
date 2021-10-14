use tcod::colors::*;
use tcod::console::*;

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
	pub fn move_by(&mut self, dx: i32, dy: i32) {
		self.x += dx;
		self.y += dy;
	}

	/// set the color and then draw the character that represents this object at its position
	pub fn draw(&self, con: &mut dyn Console) {
		con.set_default_foreground(self.color);
		con.put_char(self.x, self.y, self.glyph, BackgroundFlag::None);
	}
}
