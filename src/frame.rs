use crossterm::style::Color;

use crate::{NUM_COLUMNS, NUM_ROWS};

#[derive(PartialEq)]
pub struct Tile {
	pub icon: char,
	pub background_colour: Color,
	pub foreground_colour: Color
}

pub type Frame = Vec<Vec<Tile>>;

pub fn new_frame() -> Frame {
	let mut columns = Vec::with_capacity(NUM_COLUMNS);
	for _ in 0..NUM_COLUMNS {
		let mut column = Vec::with_capacity(NUM_ROWS);
		for _ in 0..NUM_ROWS {
			column.push(Tile {
				icon: ' ',
				background_colour: Color::Black,
				foreground_colour: Color::White
			});
		}
		columns.push(column);
	}
	columns
}

pub trait Drawable {
	fn draw(&self, frame: &mut Frame);
}
