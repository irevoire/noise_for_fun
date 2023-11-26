use std::ops::{Add, AddAssign};

use minifb::{Window, WindowOptions};
use noise::{NoiseFn, Perlin};
use rand::prelude::*;

#[derive(Debug, Clone, Copy)]
struct Coord {
    x: u16,
    y: u16,
}

impl Coord {
    pub fn new(x: impl TryInto<u16>, y: impl TryInto<u16>) -> Self {
        Self {
            x: x.try_into().map_err(|_| ()).unwrap(),
            y: y.try_into().map_err(|_| ()).unwrap(),
        }
    }
    pub fn rand(param: &Param) -> Self {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..param.width);
        let y = rng.gen_range(0..param.height);
        Self::new(x, y)
    }

    pub fn scale(&self, width: usize, height: usize) -> [f64; 2] {
        let x = self.x as f64 / width as f64; // [0:1]
        let y = self.y as f64 / height as f64; // [0:1]

        [x, y].map(|coord| coord * 2. - 1.) // [-1:1]
    }
}

impl Add<Speed> for Coord {
    type Output = Self;

    fn add(self, rhs: Speed) -> Self::Output {
        let x = if rhs.x.is_negative() {
            self.x - ((-rhs.x) as u16)
        } else {
            self.x + rhs.x as u16
        };
        let y = if rhs.y.is_negative() {
            self.y - ((-rhs.y) as u16)
        } else {
            self.y + rhs.y as u16
        };

        Self { x, y }
    }
}

impl AddAssign<Speed> for Coord {
    fn add_assign(&mut self, rhs: Speed) {
        *self = *self + rhs;
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Speed {
    x: i8,
    y: i8,
}

impl Speed {
    pub fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }
}

impl Add<(i8, i8)> for Speed {
    type Output = Self;

    fn add(self, (x, y): (i8, i8)) -> Self::Output {
        Self {
            x: self.x + x,
            y: self.y + y,
        }
    }
}

impl AddAssign<(i8, i8)> for Speed {
    fn add_assign(&mut self, rhs: (i8, i8)) {
        *self = *self + rhs;
    }
}

impl From<(i8, i8)> for Speed {
    fn from((x, y): (i8, i8)) -> Self {
        Self::new(x, y)
    }
}

#[derive(Debug, Clone)]
struct Particle {
    coord: Coord,
    speed: Speed,
    ttl: u8,
}

impl Particle {
    pub fn new(x: u16, y: u16, ttl: u8) -> Self {
        Self {
            coord: Coord { x, y },
            speed: Speed { x: 0, y: 0 },
            ttl,
        }
    }

    pub fn update(&mut self, param: &Param) {
        self.coord += self.speed;

        // The particle escaped the canvas or died by its old age
        // We should re-insert it into the canvas
        if self.coord.x as usize >= param.width
            || self.coord.y as usize >= param.height
            || self.ttl == 0
        {
            self.coord = Coord::rand(param);
            self.ttl = param.ttl;
        }

        self.ttl -= 1;
        let direction = param.noise.get(self.coord.scale(param.width, param.height)) * 180.;
        self.speed =
            convert_radian_speed_to_cardinal_speed(param.iteration_speed, direction.to_radians())
                .into();
    }

    pub fn to_coord(&self, param: &Param) -> usize {
        self.coord.y as usize * param.width + self.coord.x as usize
    }

    pub fn colorize(&self, param: &Param) -> u32 {
        let ttl_ratio = self.ttl as f32 / param.ttl as f32;
        // dbg!(ttl_ratio);
        hue_to_rgb(ttl_ratio * 360., 1.0, 1.0)
    }
}

struct Param {
    noise: Perlin,
    iteration_speed: u8,
    ttl: u8,
    width: usize,
    height: usize,
}

fn convert_radian_speed_to_cardinal_speed(speed: u8, direction: f64) -> (i8, i8) {
    (
        (direction.cos() * speed as f64) as i8, // x
        (direction.sin() * speed as f64) as i8, // y
    )
}

fn main() {
    let width = 1080;
    let height = 800;
    let nb_particles = 100_000;

    let mut buffer = vec![0; width * height];

    let mut window = Window::new("Perlin", width, height, WindowOptions::default()).unwrap();

    let mut particles = Vec::with_capacity(width * height);

    let perlin = Perlin::new(14);

    let param = Param {
        noise: perlin,
        iteration_speed: 5,
        ttl: 1,
        width,
        height,
    };

    for i in 0..nb_particles {
        let mut particle = Particle::new(0, 0, 0);
        particle.update(&param);
        particles.push(particle);
    }

    loop {
        // reset the buffer to black entirely
        buffer.fill(0);

        // update and insert all the particle in the buffer
        for particle in particles.iter_mut() {
            particle.update(&param);

            buffer[particle.to_coord(&param)] = particle.colorize(&param);
        }

        dbg!(&particles[0]);

        window.update_with_buffer(&buffer, width, height).unwrap();
        std::thread::sleep_ms(1000 / 30);
        // std::thread::sleep_ms(100);

        println!("Printed one frame");
    }
}

pub fn rgb(r: u32, g: u32, b: u32) -> u32 {
    (r << 16) | (g << 8) | b
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
    let x: f32 = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs()) as f32;
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

#[cfg(test)]
mod test {
    use std::f64::consts::PI;

    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn convert_direction() {
        assert_debug_snapshot!(convert_radian_speed_to_cardinal_speed(2, 0.), @r###"
        (
            2,
            0,
        )
        "###);

        assert_debug_snapshot!(convert_radian_speed_to_cardinal_speed(2, PI / 4.), @r###"
        (
            1,
            1,
        )
        "###);

        assert_debug_snapshot!(convert_radian_speed_to_cardinal_speed(2, PI / 2.), @r###"
        (
            0,
            2,
        )
        "###);

        assert_debug_snapshot!(convert_radian_speed_to_cardinal_speed(2, 3. * PI / 4.), @r###"
        (
            -1,
            1,
        )
        "###);

        assert_debug_snapshot!(convert_radian_speed_to_cardinal_speed(2, PI), @r###"
        (
            -2,
            0,
        )
        "###);

        assert_debug_snapshot!(convert_radian_speed_to_cardinal_speed(2, 5. * PI / 4.), @r###"
        (
            -1,
            -1,
        )
        "###);

        assert_debug_snapshot!(convert_radian_speed_to_cardinal_speed(2, 3. * PI / 2.), @r###"
        (
            0,
            -2,
        )
        "###);

        assert_debug_snapshot!(convert_radian_speed_to_cardinal_speed(2, 7. * PI / 4.), @r###"
        (
            1,
            -1,
        )
        "###);

        assert_debug_snapshot!(convert_radian_speed_to_cardinal_speed(2, 2. * PI), @r###"
        (
            2,
            0,
        )
        "###);
    }
}
