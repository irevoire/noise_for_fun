use std::{
    ops::{Add, AddAssign},
    time::{Duration, Instant},
};

use minifb::{Window, WindowOptions};
use noise::{NoiseFn, Perlin};
use pastel::Color;
use rand::prelude::*;

/// A coordinate in the [-1:1] space
#[derive(Debug, Clone, Copy)]
struct Coord {
    x: f32,
    y: f32,
}

impl Coord {
    pub fn new(x: impl TryInto<f32>, y: impl TryInto<f32>) -> Self {
        Self {
            x: x.try_into().map_err(|_| ()).unwrap(),
            y: y.try_into().map_err(|_| ()).unwrap(),
        }
    }
    pub fn rand() -> Self {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-1.0..1.0);
        let y = rng.gen_range(-1.0..1.0);
        Self::new(x, y)
    }
}

#[derive(Debug, Clone)]
struct Particle {
    coord: Coord,
}

impl Particle {
    pub fn new() -> Self {
        Self {
            coord: Coord::rand(),
        }
    }

    pub fn update(&mut self, param: &Param) {
        let direction = param.noise.get([self.coord.x as f64, self.coord.y as f64]) * 180.;
        let direction = direction.to_radians() as f32;
        self.coord.x += direction.cos() / 1000.;
        self.coord.y += direction.sin() / 1000.;

        // The particle escaped the canvas
        // We should re-insert it into the canvas
        if !(-1.0..=1.0).contains(&self.coord.x) || !(-1.0..=1.0).contains(&self.coord.y) {
            self.coord = Coord::rand();
        }
    }

    pub fn to_coord(&self, param: &Param) -> usize {
        // range [0:2]
        let x = self.coord.x + 1.0;
        let y = self.coord.y + 1.0;

        // range [0:width]
        let x = x / 2. * (param.width - 1) as f32;
        // range [0:height]
        let y = y / 2. * (param.height - 1) as f32;

        x as usize + param.width * y as usize
    }

    pub fn colorize(&self, param: &Param) -> Color {
        Color::red()
        // pastel::HSLA {
        //     h: 360.,
        //     s: 1.0,
        //     l: 1.0,
        //     alpha: 0.,
        // }
    }
}

struct Param {
    noise: Perlin,
    iteration_speed: u8,
    width: usize,
    height: usize,
}

fn main() {
    let width = 1080;
    let height = 800;
    let framerate = Duration::from_secs(1) / 60;
    let nb_particles = 200_000;
    // let nb_particles = 1;

    let mut buffer = vec![0; width * height];

    let mut window = Window::new("Perlin", width, height, WindowOptions::default()).unwrap();

    let mut particles = Vec::with_capacity(width * height);

    let perlin = Perlin::new(14);

    let param = Param {
        noise: perlin,
        iteration_speed: 5,
        width,
        height,
    };

    for _ in 0..nb_particles {
        let mut particle = Particle::new();
        particle.update(&param);
        particles.push(particle);
    }

    loop {
        let now = Instant::now();

        // Make a funny trail
        // for buf in buffer.iter_mut() {
        //     let color = u32_to_color(*buf);
        //     *buf = color.rotate_hue(1.).to_u32();
        // }

        // reset the buffer to black entirely
        buffer.fill(0);

        // update and insert all the particle in the buffer
        for particle in particles.iter_mut() {
            particle.update(&param);

            buffer[particle.to_coord(&param)] = particle.colorize(&param).to_u32();
        }

        // dbg!(&particles[0]);

        window.update_with_buffer(&buffer, width, height).unwrap();

        let elapsed = now.elapsed();
        if elapsed >= framerate {
            println!("We're late by {:?}", elapsed - framerate);
        } else {
            std::thread::sleep(framerate - elapsed);
        }
    }
}

pub fn u32_to_color(n: u32) -> Color {
    let r = (n >> 16) & 0xff;
    let g = (n >> 8) & 0xff;
    let b = n & 0xff;

    Color::from_rgb(r as u8, g as u8, b as u8)
}

pub fn unrgb(n: u32) -> (u32, u32, u32) {
    ((n >> 16) & 0xff, (n >> 8) & 0xff, (n) & 0xff)
}

pub fn hue_to_rgb(hue: f32, saturation: f32, value: f32) -> u32 {
    assert!((0.0..=360.0).contains(&hue), "bad hue: {}", hue);
    assert!(
        (0.0..=1.0).contains(&saturation),
        "bad saturation: {}",
        saturation
    );
    assert!((0.0..=1.0).contains(&value), "bad value: {}", value);

    let c: f32 = saturation * value;
    let x: f32 = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m: f32 = value - c;
    let (r, g, b) = match hue as u32 {
        0..=59 | 360 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        300..=359 => (c, 0.0, x),
        _ => panic!("called with wrong value for hue"),
    };
    let (r, g, b) = ((r + m) * 255.0, (g + m) * 255.0, (b + m) * 255.0);
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
