#![deny(warnings)]

use std::hint::black_box;

use gotoo_pixel_engine::{Framebuffer, Pixel};

fn main() {
    let mut framebuffer = Framebuffer::new(320, 180);
    framebuffer.clear(Pixel::BLACK);
    framebuffer.fill_rect(8, 8, 32, 16, Pixel::WHITE);
    black_box(framebuffer.pixel(8, 8));
    black_box(framebuffer.as_rgba8().len());
}
