use std::hint::black_box;
use std::time::Instant;

use gotoo_pixel_engine::{Framebuffer, Pixel};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
const ITERATIONS: u32 = 2_000;

fn main() {
    println!("M1.2 CPU primitive baseline");
    println!("framebuffer: {WIDTH}x{HEIGHT}, iterations: {ITERATIONS}");

    measure("draw_line", |framebuffer| {
        for i in 0..64_i32 {
            framebuffer.draw_line(
                0,
                i * 3 % HEIGHT as i32,
                WIDTH as i32 - 1,
                179 - i,
                Pixel::WHITE,
            );
        }
    });

    measure("draw_rect", |framebuffer| {
        for i in 0..64_i32 {
            framebuffer.draw_rect(i % 160, i % 90, 64, 32, Pixel::RED);
        }
    });

    measure("fill_rect", |framebuffer| {
        for i in 0..32_i32 {
            framebuffer.fill_rect(i * 3 % 240, i * 5 % 120, 48, 24, Pixel::GREEN);
        }
    });

    measure("draw_circle", |framebuffer| {
        for i in 0..48_i32 {
            framebuffer.draw_circle(32 + i * 5 % 256, 24 + i * 3 % 132, 18, Pixel::BLUE);
        }
    });

    measure("fill_circle", |framebuffer| {
        for i in 0..24_i32 {
            framebuffer.fill_circle(32 + i * 9 % 256, 24 + i * 7 % 132, 16, Pixel::WHITE);
        }
    });
}

fn measure(name: &str, mut draw: impl FnMut(&mut Framebuffer)) {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let start = Instant::now();

    for _ in 0..ITERATIONS {
        framebuffer.clear(Pixel::BLACK);
        draw(&mut framebuffer);
        black_box(framebuffer.as_rgba8());
    }

    let elapsed = start.elapsed();
    println!(
        "{name:<12} total={:>10?} avg/iteration={:>10?} checksum={}",
        elapsed,
        elapsed / ITERATIONS,
        checksum(&framebuffer)
    );
}

fn checksum(framebuffer: &Framebuffer) -> u64 {
    let pixels = framebuffer.as_rgba8();
    let sum = pixels
        .iter()
        .fold(0_u64, |acc, channel| acc.wrapping_add(u64::from(*channel)));

    black_box(sum)
}
