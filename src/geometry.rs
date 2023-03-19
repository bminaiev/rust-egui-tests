use eframe::epaint::Pos2;

#[derive(Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn pos2(&self) -> Pos2 {
        Pos2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }

    pub(crate) fn new(x: f64, y: f64) -> Point {
        Self { x, y }
    }
}

impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}, {})", self.x, self.y))
    }
}

pub struct Circle {
    pub center: Point,
    pub r: f64,
}
