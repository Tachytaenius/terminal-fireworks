use std::{f32::{consts::TAU, NEG_INFINITY}, ops::Add, time::Duration};

use crossterm::style::Color;
use enum_map::{Enum, EnumMap};
use glam::Vec2;
use rand::{distributions::Standard, prelude::Distribution, random, Rng};
use crate::{frame::{Frame, Tile}, NUM_COLUMNS, NUM_ROWS};

pub const BLUR_SPEED: f32 = 16.0;
pub const GRAVITY: Vec2 = Vec2::new(0.0, 8.0);
pub const SMOKE_MAX_DENSITY: f32 = 8.0;
pub const SMOKE_DISSIPATION_RATE: f32 = 0.5;
pub const SMOKE_EMISSION_REDUCTION_POWER: f32 = 0.4;
pub const SMOKE_TOTAL_DENSITY_PROBABILITY_START: f32 = 0.25;
pub const ALLOW_COLOUR_FLICKER: bool = true;
pub const ALLOW_DENSITY_FLICKER: bool = true;

#[derive(Clone)]
pub struct ContainedParticle {
	colour: SimulationColour,
	base_smoke_emission: f32,
	contained_particles: Option<Vec<ContainedParticle>>,
	explosion_speed: f32,
	timer_length: Duration
}

#[derive(Clone)]
pub struct Particle {
	position: Vec2,
	velocity: Vec2,
	base_smoke_emission: f32,
	time_remaining: f32,
	to_remove: bool,

	colour: SimulationColour,
	timer_length: Duration,
	contained_particles: Option<Vec<ContainedParticle>>
}

#[derive(Enum, Copy, Clone)]
pub enum SimulationColour { // Shades decided by the rest of the program 
	Grey, // Dark grey and black
	White, // White and grey

	Red, // Red and dark red
	Yellow, // Etc
	Green,
	Cyan,
	Blue,
	Magenta
}

impl Distribution<SimulationColour> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SimulationColour {
		match rng.gen_range(0..=7) {
			0 => SimulationColour::Grey,
			1 => SimulationColour::White,
			2 => SimulationColour::Red,
			3 => SimulationColour::Yellow,
			4 => SimulationColour::Green,
			5 => SimulationColour::Cyan,
			6 => SimulationColour::Blue,
			7 => SimulationColour::Magenta,

			_ => SimulationColour::White // Never, but okay
		}
	}
}

pub struct SmokeTile {
	colour_densities: EnumMap<SimulationColour, f32>
}

impl SmokeTile {
	pub fn new() -> Self {
		let colour_densities = EnumMap::default(); // Should be all zero
		return Self {colour_densities}
	}
}

pub struct State {
	pub time: Duration,
	pub particles: Vec<Particle>,
	pub smoke_tiles: Vec<Vec<SmokeTile>>,
	pub new_firework_timer: f32,
	pub new_firework_timer_length: f32
}

pub fn random_vec2_in_circle(radius: f32) -> Vec2 {
	return Vec2::from_angle(random::<f32>() * TAU) * random::<f32>().sqrt() * radius;
}

pub fn get_tile_pos(position: Vec2) -> Option<(usize, usize)> {
	let tile_x = position.x as isize;
	let tile_y = position.y as isize;
	if
		0 <= tile_x && tile_x < NUM_COLUMNS as isize &&
		0 <= tile_y && tile_y < NUM_ROWS as isize
	{
		return Some((tile_x as usize, tile_y as usize));
	}
	return None;
}

pub fn simulation_colour_to_crossterm_colour(colour: SimulationColour, darken: bool) -> Color {
	match colour {
		SimulationColour::Grey => {if darken {Color::Black} else {Color::DarkGrey}},
		SimulationColour::White => {if darken {Color::Grey} else {Color::White}},
		SimulationColour::Red => {if darken {Color::DarkRed} else {Color::Red}},
		SimulationColour::Yellow => {if darken {Color::DarkYellow} else {Color::Yellow}},
		SimulationColour::Green => {if darken {Color::DarkGreen} else {Color::Green}},
		SimulationColour::Cyan => {if darken {Color::DarkCyan} else {Color::Cyan}},
		SimulationColour::Blue => {if darken {Color::DarkBlue} else {Color::Blue}},
		SimulationColour::Magenta => {if darken {Color::DarkMagenta} else {Color::Magenta}}
	}
}

impl State {
	pub fn spawn_firework(&mut self) {
		let mut rng = rand::thread_rng();
		
		let num_glitter_particles = rng.gen_range(64..=256);
		let speed = rng.gen_range(32.0..64.0);
		let position = Vec2::new(
			rng.gen_range(NUM_COLUMNS as f32 * 0.4 .. NUM_COLUMNS as f32 * 0.6),
			NUM_ROWS as f32
		);
		let target = Vec2::new(NUM_COLUMNS as f32 / 2.0, 0.0) + random_vec2_in_circle(NUM_COLUMNS as f32 * 0.2);
		let timer_length_f32 = rng.gen_range(1.25..2.0);
		let mut contained_particles_vec = Vec::with_capacity(num_glitter_particles);
		for _ in 0..num_glitter_particles {
			contained_particles_vec.push(ContainedParticle {
				colour: rand::random(),
				base_smoke_emission: rng.gen_range(4.0..8.0),
				contained_particles: None,
				explosion_speed: random::<f32>().sqrt() * 32.0, // Sqrt for uniform distribution
				timer_length: Duration::from_secs_f32(rng.gen_range(1.5..3.5))
			})
		}
		self.particles.push(Particle {
			position,
			velocity: (target - position).normalize_or_zero() * speed,
			base_smoke_emission: rng.gen_range(2.0..4.0),
			time_remaining: timer_length_f32,
			to_remove: false,
			colour: SimulationColour::White,
			timer_length: Duration::from_secs_f32(timer_length_f32),
			contained_particles: Some(contained_particles_vec)
		});
	}

	pub fn new() -> Self {
		let particles = Vec::new();

		let mut smoke_tiles = Vec::with_capacity(NUM_COLUMNS);
		for _ in 0..NUM_COLUMNS {
			let mut column = Vec::with_capacity(NUM_ROWS);
			for _ in 0..NUM_ROWS {
				column.push(SmokeTile::new());
			}
			smoke_tiles.push(column);
		}

		let ret = Self {
			time: Duration::ZERO,
			particles,
			smoke_tiles,
			new_firework_timer: 0.0,
			new_firework_timer_length: 5.0
		};
		return ret;
	}

	pub fn update(&mut self, dt: Duration) {
		let dt_f32 = dt.as_secs_f32();

		// Update smoke
		for column in self.smoke_tiles.iter_mut() {
			for tile in column.iter_mut() {
				for (_, density) in tile.colour_densities.iter_mut() {
					*density = (*density - SMOKE_DISSIPATION_RATE * dt_f32).max(0.0);
				}
			}
		}

		// Update particles
		let mut particles_to_add = Vec::new();
		for particle in self.particles.iter_mut() {
			// Smoke
			if let Some((tile_x, tile_y)) = get_tile_pos(particle.position) {
				let density = &mut self.smoke_tiles[tile_x][tile_y].colour_densities[particle.colour];
				let length_f32 = particle.timer_length.as_secs_f32();
				let emission_multiplier = (1.0 - (length_f32 - particle.time_remaining) / length_f32).powf(SMOKE_EMISSION_REDUCTION_POWER);
				let effective_base_smoke_emission = particle.base_smoke_emission * emission_multiplier;
				*density = (*density + effective_base_smoke_emission * dt_f32).min(SMOKE_MAX_DENSITY)
			}

			// Motion
			particle.velocity += GRAVITY * dt_f32;
			particle.position += particle.velocity * dt_f32;

			// Time and removal
			particle.time_remaining -= dt_f32;
			if particle.time_remaining <= 0.0 {
				particle.to_remove = true;
				if particle.contained_particles.is_none() {
					continue;
				}
				for contained_particle in particle.contained_particles.as_ref().unwrap().iter() {
					particles_to_add.push(Particle {
						position: particle.position,
						velocity: contained_particle.explosion_speed * Vec2::from_angle(random::<f32>() * TAU), // No additive velocity
						base_smoke_emission: contained_particle.base_smoke_emission,
						time_remaining: contained_particle.timer_length.as_secs_f32(),
						to_remove: false,
						colour: contained_particle.colour,
						timer_length: contained_particle.timer_length,
						contained_particles: contained_particle.contained_particles.clone()
					})
				}
			}
		}
		self.particles.retain(|particle| !particle.to_remove);
		for particle in particles_to_add.iter() {
			self.particles.push((*particle).clone());
		}

		// Spawn new fireworks
		self.new_firework_timer -= dt_f32;
		if self.new_firework_timer <= 0.0 {
			self.new_firework_timer = self.new_firework_timer_length;
			self.spawn_firework();
			if random::<f32>() < 0.3 {
				self.spawn_firework();
			}
			if random::<f32>() < 0.3 {
				self.spawn_firework();
			}
		}
		
		self.time = self.time.saturating_add(dt);
	}

	pub fn draw(&self, frame: &mut Frame) {
		for particle in self.particles.iter() {
			if let Some((tile_x, tile_y)) = get_tile_pos(particle.position) {
				let icon = if particle.velocity.length() < BLUR_SPEED {
					'\u{2219}' // Bullet operator
				} else {
					 // I think there might be a better way to get the direction in 8ths involving the difference between the x and the y of the normalised version
					let angle = particle.velocity.to_angle().add(TAU / 16.0).rem_euclid(TAU); // rem_euclid is true modulo (iirc) in that it handles negatives
					let direction_identifier = (angle / TAU * 8.0) as isize;
					match direction_identifier {
						0 => {'-'},
						1 => {'\\'},
						2 => {'|'},
						3 => {'/'},
						4 => {'-'},
						5 => {'\\'},
						6 => {'|'},
						7 => {'/'},
						_ => {'?'}
					}
				};
				frame[tile_x][tile_y] = Tile {
					icon,
					background_colour: Color::Blue, // Temporary
					foreground_colour: simulation_colour_to_crossterm_colour(particle.colour, false)
				};
			}
		}

		// Add smoke
		for x in 0..NUM_COLUMNS {
			for y in 0..NUM_ROWS {
				let mut chosen_colour = None;
				let mut density_total = 0.0;

				if ALLOW_COLOUR_FLICKER {
					for (_, &density) in self.smoke_tiles[x][y].colour_densities.iter() {
						density_total += density;
					}
					let mut chooser = random::<f32>() * density_total;
					for (colour, &density) in self.smoke_tiles[x][y].colour_densities.iter() {
						if chooser < density {
							chosen_colour = Some(colour);
							break;
						}
						chooser -= density;
					}
				} else {
					let mut highest_density = NEG_INFINITY;
					for (colour, &density) in self.smoke_tiles[x][y].colour_densities.iter() {
						density_total += density;
						if density > highest_density {
							highest_density = density;
							chosen_colour = Some(colour);
						}
					}
				}

				let blink = if ALLOW_DENSITY_FLICKER {
					random::<f32>() < 1.0 - density_total / SMOKE_TOTAL_DENSITY_PROBABILITY_START
				} else {
					false
				};

				frame[x][y].background_colour = if blink || chosen_colour.is_none() || density_total == 0.0 {
					Color::Black
				} else {
					simulation_colour_to_crossterm_colour(chosen_colour.unwrap(), true)
				};
			}
		}
	}
}
