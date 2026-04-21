use std::fmt;

pub const MAX_SIZE: usize = 1024;
pub static GLOBAL_COUNTER: usize = 0;

pub type Alias = Vec<u8>;

pub enum Direction {
    North,
    South,
    East,
    West,
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub union RawVal {
    int: i32,
    float: f32,
}

pub trait Shape {
    type Unit;
    const DIMENSIONS: u32;

    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn name(&self) -> &str {
        "shape"
    }
    fn id() -> u64;
    fn default_id() -> u64 {
        0
    }
}

pub struct Circle {
    pub radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn perimeter(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
}

impl Circle {
    pub fn new(radius: f64) -> Self {
        Self { radius }
    }

    pub fn diameter(&self) -> f64 {
        self.radius * 2.0
    }
}

impl fmt::Display for Circle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Circle(r={})", self.radius)
    }
}

pub fn distance(a: &Point, b: &Point) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

macro_rules! assert_close {
    ($a:expr, $b:expr, $eps:expr) => {
        assert!(($a - $b).abs() < $eps);
    };
}
