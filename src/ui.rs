use eframe::{
    egui::{self, Sense},
    epaint::{vec2, CircleShape, Color32, Rounding, Shape},
};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    geometry::{Circle, Point},
    screen_transform::ScreenTransform,
};

pub struct State {
    screen_transform: ScreenTransform,
    circles: Vec<Circle>,
    chosen_circle: Option<usize>,
}

impl State {
    pub fn new() -> Self {
        let mut rnd = StdRng::seed_from_u64(787788);
        let mut circles = vec![];
        const MAX_R: f64 = 0.05;
        const MAX_C: f64 = 100.0;
        for _ in 0..100_000 {
            let center = Point {
                x: rnd.gen_range(0.0..MAX_C),
                y: rnd.gen_range(0.0..MAX_C),
            };
            let r = rnd.gen_range(MAX_R / 10.0..MAX_R);
            circles.push(Circle { center, r })
        }
        let screen_transform = ScreenTransform::new(1.0, vec2(500.0, 500.0));
        Self {
            circles,
            screen_transform,
            chosen_circle: None,
        }
    }
}

impl eframe::App for State {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(2.0);
        if let Some(chosen_circle) = self.chosen_circle {
            egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
                ui.label(format!("Hello World! Chosen circle: {}", chosen_circle));
            });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");

            let available_rect = ui.available_size();
            // eprintln!("available: {available_rect:?}");
            let (rect, response) = ui.allocate_exact_size(available_rect, Sense::click_and_drag());
            // eprintln!("allocated: {rect:?}");

            let mut circle_shapes: Vec<_> = self
                .circles
                .iter()
                .map(|c| {
                    CircleShape::filled(
                        self.screen_transform.to_screen(c.center),
                        self.screen_transform.to_screen_dist(c.r),
                        Color32::BLUE,
                    )
                })
                .collect();

            if let Some(mouse_pos) = response.hover_pos() {
                ui.input(|i| {
                    let delta = i.scroll_delta.y;
                    if delta != 0.0 {
                        self.screen_transform.zoom(mouse_pos, delta);
                    }
                });
                let mut closest_circle = (None, 5.0);
                for (idx, circle) in circle_shapes.iter().enumerate() {
                    let dist = circle.center.distance(mouse_pos) - circle.radius;
                    if dist < closest_circle.1 {
                        closest_circle.0 = Some(idx);
                        closest_circle.1 = dist;
                    }
                }
                if let Some(closest_circle) = closest_circle.0 {
                    circle_shapes[closest_circle].fill = Color32::RED;
                    if response.clicked() {
                        self.chosen_circle = Some(closest_circle);
                    }
                }
            }
            if let Some(chosen_circle) = self.chosen_circle {
                circle_shapes[chosen_circle].fill = Color32::GREEN;
            }
            self.screen_transform.drag(response.drag_delta());

            let mut shapes = vec![Shape::rect_filled(rect, Rounding::none(), Color32::WHITE)];
            shapes.extend(circle_shapes.into_iter().map(|c| Shape::Circle(c)));

            ui.painter().with_clip_rect(rect).extend(shapes);
        });
    }
}
