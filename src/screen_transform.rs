use eframe::epaint::{pos2, Pos2, Vec2};

use crate::geometry::Point;

pub struct ScreenTransform {
    zoom_log: f64,
    shift: Vec2,
}

const ZOOM_DELTA_COEF: f32 = 500.0;

impl ScreenTransform {
    pub fn new(zoom_log: f64, shift: Vec2) -> Self {
        Self { zoom_log, shift }
    }

    pub fn get_zoom(&self) -> f64 {
        self.zoom_log.exp()
    }

    pub fn to_screen(&self, p: Point) -> Pos2 {
        pos2(
            (self.get_zoom() * p.x) as f32,
            (self.get_zoom() * p.y) as f32,
        ) + self.shift
    }

    pub fn to_screen_dist(&self, d: f64) -> f32 {
        (d * self.get_zoom()) as f32
    }

    pub fn from_screen(&self, p: Pos2) -> Point {
        let res = p - self.shift;
        Point::new(
            (res.x as f64) / self.get_zoom(),
            (res.y as f64) / self.get_zoom(),
        )
    }

    pub fn zoom(&mut self, mouse_screen_pos: Pos2, mouse_scroll_delta: f32) {
        let prev_zoom = self.get_zoom();
        self.zoom_log += (mouse_scroll_delta / ZOOM_DELTA_COEF) as f64;
        let zoom_delta = (self.get_zoom() / prev_zoom) as f32;
        self.shift =
            (mouse_screen_pos + (self.shift - mouse_screen_pos.to_vec2()) * zoom_delta).to_vec2();
    }

    pub(crate) fn drag(&mut self, drag_delta: Vec2) {
        self.shift += drag_delta;
    }
}
