#![no_main]

#[allow(unused_imports)]
use anodized::spec;

#[spec(maintains: self.z < 42.0)]
struct Point {
    x: f32,
    y: f32,
}
