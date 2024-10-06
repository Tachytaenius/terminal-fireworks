use std::{error::Error, io, sync::mpsc, thread, time::{Duration, Instant}};

use crossterm::{cursor::{Hide, Show}, event::{self, Event, KeyCode}, style::ResetColor, terminal::{self, Clear, ClearType}, ExecutableCommand};
use terminal_fireworks::{frame, render, state::State};

const SLEEP_MS: u64 = 1;

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
                    }
                    _ => {}
                }
            }
        }

        if !paused {
            state.update(delta);
        }

        state.draw(&mut current_frame);

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
