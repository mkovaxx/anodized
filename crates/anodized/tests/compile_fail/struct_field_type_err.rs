#![no_main]

#[allow(unused_imports)]
use anodized::spec;

#[spec(maintains: self.x != "text")]
struct Point {
    x: f32,
    y: f32,
}
