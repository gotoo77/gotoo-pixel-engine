#![deny(warnings)]

use std::hint::black_box;

fn main() {
    black_box((320_u32, 180_u32));
}
