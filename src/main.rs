use std::{error::Error, io, sync::mpsc, thread, time::{Duration, Instant}};

use crossterm::{cursor::{Hide, Show}, event::{self, Event, KeyCode}, style::{Color, ResetColor}, terminal::{self, Clear, ClearType}, ExecutableCommand};
use terminal_fireworks::{frame::{self, Tile}, render, state::State, NUM_COLUMNS, NUM_ROWS};

const SLEEP_MS: u64 = 8;

fn main() -> Result <(), Box<dyn Error>> {
    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    // stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread
    let (render_transceiver, render_receiver) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true).unwrap();
        loop {
            let current_frame = match render_receiver.recv() {
                Ok(x) => x,
                Err(_) => break
            };
            render::render(&mut stdout, &last_frame, &current_frame, false).unwrap();
            last_frame = current_frame;
        }
    });

    let mut state = State::new();
    let mut paused = false;
    let mut show_help = true;
    let mut instant = Instant::now();
    'mainloop: loop {
        // Per-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut current_frame = frame::new_frame();

        // Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break 'mainloop;
                    },
                    KeyCode::Char(' ') | KeyCode::Pause => {
                        paused = !paused;
                    },
                    KeyCode::Char('h') => {
                        show_help = !show_help;
                    }
                    _ => {}
                }
            }
        }

        if !paused {
            state.update(delta);
        }

        state.draw(&mut current_frame);
        if show_help {
            let window_x = 1;
            let window_y = 1;
            let lines = [
                "Help:",
                "Q, Esc: Quit",
                "Space, Pause: Pause",
                "H: Toggle help"
            ];
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
                        background_colour: Color::Black,
                        foreground_colour: Color::White
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

        // Finish iteration
        let _ = render_transceiver.send(current_frame);
        thread::sleep(Duration::from_millis(SLEEP_MS));
    }

    // Cleanup
    drop(render_transceiver);
    render_handle.join().unwrap();
    stdout.execute(ResetColor)?;
    stdout.execute(Show)?;
    stdout.execute(Clear (ClearType::All))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;
    terminal::disable_raw_mode()?;
    Ok(())
}
