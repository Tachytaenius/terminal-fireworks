use crossterm::style::Color;

use crate::{frame::{Frame, Tile}, NUM_COLUMNS, NUM_ROWS};

pub fn draw_window(window_x: usize, window_y: usize, lines: Vec<&str>, current_frame: &mut Frame, foreground_colour: Color, background_colour: Color) {
	let mut greatest_width = 0;
	for &line in lines.iter() {
		greatest_width = greatest_width.max(line.chars().count());
	}
	let window_width = greatest_width + 4;
	let window_height = lines.len() + 4;
	for local_y in 0..=lines.len()+3 {
		let y = window_y + local_y;
		if y >= NUM_ROWS {
			break;
		}
		for local_x in 0..=greatest_width+3 {
			let x = local_x + window_x;
			if x >= NUM_COLUMNS {
				break;
			}
			let icon = if local_x == 0 && local_y == 0 || local_x == window_width - 1 && local_y == window_height - 1 {
				'/'
			} else if local_x == 0 && local_y == window_height - 1 || local_x == window_width - 1 && local_y == 0 {
				'\\'
			} else if local_x == 0 || local_x == window_width - 1 {
				'|'
			} else if local_y == 0 || local_y == window_height - 1 {
				'-'
			} else {
				' '
			};
			current_frame[x][y] = Tile {
				icon,
				background_colour,
				foreground_colour
			}
		}
	}
	for (line_number, &line) in lines.iter().enumerate() {
		let y_index = window_y + 2 + line_number;
		if y_index >= NUM_ROWS {
			break;
		}
		for (char_number, char) in line.chars().into_iter().enumerate() {
			let x_index = window_x + 2 + char_number;
			if x_index >= NUM_COLUMNS {
				break;
			}
			current_frame[x_index][y_index].icon = char;
		}
	}
}
