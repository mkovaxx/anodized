use anodized::spec;

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[spec(
    captures: arr as [first, second, third],
    ensures: [
        first == arr[0],
        second == arr[1],
        third == arr[2],
    ],
)]
fn match_array(arr: [i32; 3]) {
    println!("Array elements: {}, {}, {}", arr[0], arr[1], arr[2]);
}

#[spec(
    captures: tuple as (a, b, c),
    ensures: [
        a + b + c == tuple.0 + tuple.1 + tuple.2,
    ],
)]
fn match_tuple(tuple: (i32, i32, i32)) {
    let (a, b, c) = tuple;
    println!("Sum: {}, Product: {}", a + b + c, a * b * c);
}

#[spec(
    captures: point as Point { x, y },
    ensures: [
        x == point.x,
        y == point.y,
    ],
)]
fn match_struct(point: Point) {
    println!("Point: ({}, {})", point.x, point.y);
}

#[spec(
    captures: [a, b, c] as slice,
    ensures: [
        slice[0] == a,
        slice[1] == b,
        slice[2] == c,
    ],
)]
fn capture_as_array(a: i32, b: i32, c: i32) {
    println!("Captured as array: {:?}", [a, b, c]);
}

fn main() {
    // Array pattern matching
    let numbers = [1, 2, 3];
    match_array(numbers);

    // Tuple pattern matching
    let coords = (3, 4, 5);
    match_tuple(coords);

    // Struct pattern matching
    let point = Point { x: 5, y: -3 };
    match_struct(point);

    // Array expression captured with alias
    capture_as_array(10, 20, 30);
}
