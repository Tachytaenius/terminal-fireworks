# Terminal Fireworks

This is a Rust program for fireworks that runs in a terminal.
It could be made better, but... it works on my machine, and hopefully yours too :3

**WARNING**: I haven't really got any failsafes or anything for the quit buttons failing (which they did for me when I overloaded the program with excessive smoke at one point), so make sure you can get to a context where you can `killall terminal-fireworks` or whatever, okay?

What's needed:
- For Linux, **switch to a TTY to run this.**
	Maybe that makes it Linux only, becuase I only designed it for Linux TTYs.
	It definitely fails for me in `gnome-terminal`.
- Ideally, set the font to a square font—the smaller the better.
	Make sure that the font has the bullet operator (which is ∙) because I use it to draw slower particles.
	I just found that I could run `setfont /usr/share/consolefonts/Lat7-VGA8.psf.gz`.
- Edit `NUM_COLUMNS` (width) and `NUM_ROWS` (height) in `lib.rs` as appropriate (for my 1920x1080 screen with an 8x8 font, I have them at 240 and 135. That has program width at screen width divided by font width, and the same thing is done for height).
- Do `cargo run --release` (you need Rust properly installed).
	If you run this in debug mode, it uses a lot more CPU.
