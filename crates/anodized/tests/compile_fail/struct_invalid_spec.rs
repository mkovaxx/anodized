#![no_main]

use anodized::spec;

#[spec(
    requires: self.x > 0.0,
    maintains: self.x != self.y,
    captures: old_x = self.x,
    inspects: ret_val,
    ensures: self.y > 0.0,
)]
struct Point {
    x: f32,
    y: f32,
}
