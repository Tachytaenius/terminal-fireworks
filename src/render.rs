use std::{error::Error, io::{Stdout, Write}};

use crossterm::{cursor::MoveTo, style::{Color, SetBackgroundColor, SetForegroundColor}, terminal::{Clear, ClearType, SetSize}, QueueableCommand};

use crate::{frame::Frame, NUM_COLUMNS, NUM_ROWS};

pub fn render(stdout: &mut Stdout, last_frame: &Frame, current_frame: &Frame, force_redraw: bool) -> Result <(), Box<dyn Error>> {
	if force_redraw {
		stdout.queue(SetBackgroundColor(Color::Black))?;
		stdout.queue(Clear(ClearType::All))?;
		stdout.queue(SetBackgroundColor(Color::Black))?;
		stdout.queue(SetForegroundColor(Color::White))?;
	}

	stdout.queue(SetSize(NUM_COLUMNS as u16, NUM_ROWS as u16))?;

	for (x, column) in current_frame.iter().enumerate() {
		for (y, tile) in column.iter().enumerate() {
			if force_redraw || *tile != last_frame[x][y] {
				stdout.queue(MoveTo(x as u16, y as u16))?;
				stdout.queue(SetBackgroundColor(tile.background_colour))?;
				stdout.queue(SetForegroundColor(tile.foreground_colour))?;
				print!("{}", tile.icon);
			}
		}
	}

	stdout.flush()?;

	return Ok(())
}
