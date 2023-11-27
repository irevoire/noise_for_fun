use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::{Duration, Instant},
};

use minifb::{Window, WindowOptions};
use noise::{
    Checkerboard, Fbm, HybridMulti, NoiseFn, OpenSimplex, Perlin, PerlinSurflet, RidgedMulti,
    Simplex, SuperSimplex, Worley,
};
use pastel::Color;
use rand::prelude::*;
use rayon::prelude::*;

trait Noise2D: NoiseFn<f64, 2> {}
impl<T> Noise2D for T where T: NoiseFn<f64, 2> {}

trait Noise3D: NoiseFn<f64, 3> {}
impl<T> Noise3D for T where T: NoiseFn<f64, 3> {}

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

    pub fn update<Noise: Noise3D>(&mut self, param: &Param<Noise>) {
        let direction = param.noise_get(self.coord.x, self.coord.y) * 180.;
        let direction = direction.to_radians() as f32;
        self.coord.x += direction.cos() / 1000.;
        self.coord.y += direction.sin() / 1000.;

        // The particle escaped the canvas
        // We should re-insert it into the canvas
        if !(-1.0..=1.0).contains(&self.coord.x) || !(-1.0..=1.0).contains(&self.coord.y) {
            self.coord = Coord::rand();
        }
    }

    pub fn to_coord<Noise>(&self, param: &Param<Noise>) -> usize {
        // range [0:2]
        let x = self.coord.x + 1.0;
        let y = self.coord.y + 1.0;

        // range [0:width]
        let x = x / 2. * (param.width - 1) as f32;
        // range [0:height]
        let y = y / 2. * (param.height - 1) as f32;

        x as usize + param.width * y as usize
    }

    pub fn colorize<Noise>(&self, param: &Param<Noise>) -> Color {
        Color::red()
        // pastel::HSLA {
        //     h: 360.,
        //     s: 1.0,
        //     l: 1.0,
        //     alpha: 0.,
        // }
    }
}

struct Param<Noise> {
    noise: Noise,
    iteration_speed: u8,
    iteration: u16,
    width: usize,
    height: usize,
}

impl<Noise: Noise3D> Param<Noise> {
    pub fn noise_get(&self, x: f32, y: f32) -> f64 {
        // we must bring back self.iteration in the range [-1:1]
        // now it's in the range [0:1]
        let iteration = self.iteration as f64 / u16::MAX as f64;
        self.noise.get([x as f64, y as f64, iteration * 2. - 1.])
    }
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

    // let noise = Perlin::new(14);
    // let noise = PerlinSurflet::new(14);
    // let noise = Checkerboard::new(14).set_size(32);
    // let noise = HybridMulti::<Perlin>::new(14);
    // let noise = OpenSimplex::new(14);
    // let noise = RidgedMulti::<Perlin>::new(14);
    // let noise = Simplex::new(14);
    // let noise = SuperSimplex::new(14);
    // let noise = Worley::new(14); // can't parallelize with this one
    let noise = Fbm::<Perlin>::new(14);

    let mut param = Param {
        noise,
        iteration_speed: 5,
        iteration: 0,
        width,
        height,
    };

    for _ in 0..nb_particles {
        let mut particle = Particle::new();
        particle.update(&param);
        particles.push(particle);
    }

    loop {
        param.iteration += 1;
        let now = Instant::now();

        // Make a funny trail
        buffer.par_iter_mut().for_each(|buf| {
            let color = u32_to_color(*buf);
            *buf = color.rotate_hue(1.).to_u32();
        });

        // reset the buffer to black entirely
        // buffer.fill(0);

        let shared_buffer: &[AtomicU32] = unsafe { std::mem::transmute(buffer.as_slice()) };

        // update and insert all the particle in the buffer
        particles.par_iter_mut().for_each(|particle| {
            particle.update(&param);

            shared_buffer[particle.to_coord(&param)]
                .store(particle.colorize(&param).to_u32(), Ordering::Relaxed);
        });

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
