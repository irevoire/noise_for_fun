use minifb::{Window, WindowOptions};
use noise::{NoiseFn, Perlin};

fn main() {
    let width = 1080;
    let height = 800;
    let mut buffer = vec![0; width * height];

    let mut window = Window::new("Perlin", width, height, WindowOptions::default()).unwrap();

    for seed in 0.. {
        // let seed = 1;
        // loop {
        let perlin = Perlin::new(seed);

        for w in 0..width {
            for h in 0..height {
                let x = w as f64 / width as f64 * 2. - 1.;
                let y = h as f64 / height as f64 * 2. - 1.;
                let val = perlin.get([x, y]);
                // val is in the range -1, 1. Move it to the range 0, 1
                let val = (val + 1.) / 2.;
                let val = (val * 255.) as u32;

                let (r, g, b) = unrgb(buffer[width * h + w]);
                let color = rgb(
                    r.max(val) - r.min(val),
                    g.max(r) - g.min(r),
                    b.max(g) - b.min(g),
                );
                buffer[width * h + w] = color;
            }
        }
        window.update_with_buffer(&buffer, width, height).unwrap();
        std::thread::sleep_ms(1000 / 60);
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
