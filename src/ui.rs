use std::{
    cmp::{max, min},
    ops::Range,
};

use eframe::{
    egui::{self, Sense},
    epaint::{pos2, vec2, CircleShape, Color32, ColorImage, Rect, Rounding, Shape, Stroke},
};
use egui_extras::RetainedImage;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    geometry::{Circle, Point},
    scanner::Scanner,
    screen_transform::ScreenTransform,
};

#[derive(Copy, Clone)]
struct Cell {
    col: usize,
    row: usize,
}

#[derive(Debug)]
struct VisibleArea {
    rows: Range<usize>,
    cols: Range<usize>,
}

impl VisibleArea {
    pub fn is_too_big(&self) -> bool {
        let dr = self.rows.len() as i64;
        let dc = self.cols.len() as i64;
        dr * dc >= 20_000
    }
}

pub struct State {
    screen_transform: ScreenTransform,
    circles: Vec<Circle>,
    chosen_circle: Option<usize>,
    a: Vec<Vec<Option<i64>>>,
    snakes: Vec<Vec<Cell>>,
    min_cost: i64,
    max_cost: i64,
    ret_image: RetainedImage,
    snake_parts: Vec<Vec<Vec<Cell>>>,
}

const TEST_ID: usize = 5;
const SNAKE_COLOR: Color32 = Color32::BLUE;

impl State {
    pub fn new() -> Self {
        let mut input = Scanner::new(&format!("{TEST_ID}.txt"));
        let cols: usize = input.next();
        let rows: usize = input.next();
        let n: usize = input.next();
        let lens: Vec<_> = (0..n).map(|_| input.next::<usize>()).collect();

        let mut a = vec![vec![None; cols]; rows];
        for i in 0..rows {
            for j in 0..cols {
                let s: String = input.next();
                if s != "*" {
                    let value: i64 = s.parse().unwrap();
                    a[i][j] = Some(value);
                }
            }
        }
        eprintln!("Cols: {cols}, rows: {rows}");

        let mut snakes = vec![];
        {
            let mut out = Scanner::new(&format!("{TEST_ID}.out"));
            for &len in lens.iter() {
                let x: usize = out.next();
                let y: usize = out.next();
                let mut p = Cell { col: x, row: y };
                let mut snake = vec![p];
                for _ in 1..len {
                    let delta: String = out.next();
                    match delta.as_str() {
                        "R" => p.col = (p.col + 1) % cols,
                        "D" => p.row = (p.row + 1) % rows,
                        "L" => p.col = (p.col + cols - 1) % cols,
                        "U" => p.row = (p.row + rows - 1) % rows,
                        _ => unreachable!(),
                    }
                    snake.push(p);
                }
                snakes.push(snake);
            }
        }

        let (min_cost, max_cost) = field_min_max(&a);

        let mut snake_parts = vec![vec![vec![]; cols]; rows];
        let ret_image = {
            const W: usize = 5;
            let mut ci = ColorImage::new([W * a[0].len(), W * a.len()], Color32::WHITE);
            for r in 0..rows {
                for c in 0..cols {
                    let color = field_color(&a, min_cost, max_cost, r, c);
                    for dr in 0..W {
                        for dc in 0..W {
                            ci[(c * W + dc, r * W + dr)] = color;
                        }
                    }
                }
            }

            for snake in snakes.iter() {
                for w in snake.windows(2) {
                    let dist = w[0].row.abs_diff(w[1].row) + w[0].col.abs_diff(w[1].col);
                    if dist == 1 {
                        snake_parts[w[0].row][w[0].col].push(w[1]);

                        let p1 = Cell {
                            row: w[0].row * W + W / 2,
                            col: w[0].col * W + W / 2,
                        };
                        let p2 = Cell {
                            row: w[1].row * W + W / 2,
                            col: w[1].col * W + W / 2,
                        };
                        for r in min(p1.row, p2.row)..=max(p1.row, p2.row) {
                            for c in min(p1.col, p2.col)..=max(p1.col, p2.col) {
                                ci[(c, r)] = SNAKE_COLOR;
                            }
                        }
                    }
                }
            }

            RetainedImage::from_color_image("field", ci)
        };

        let mut rnd = StdRng::seed_from_u64(787788);
        let mut circles = vec![];
        const MAX_R: f64 = 0.2;
        const MAX_C: f64 = 100.0;
        for _ in 0..0 {
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
            a,
            snakes,
            min_cost,
            max_cost,
            ret_image,
            snake_parts,
        }
    }

    fn draw_snakes(&self, shapes: &mut Vec<Shape>) {
        let stroke = Stroke::new(2.0, Color32::RED);
        for snake in self.snakes.iter() {
            for w in snake.windows(2) {
                let dist = w[0].row.abs_diff(w[1].row) + w[0].col.abs_diff(w[1].col);
                if dist == 1 {
                    let p1 = self
                        .screen_transform
                        .to_screen(Point::new(w[0].col as f64 + 0.5, w[0].row as f64 + 0.5));
                    let p2 = self
                        .screen_transform
                        .to_screen(Point::new(w[1].col as f64 + 0.5, w[1].row as f64 + 0.5));
                    shapes.push(Shape::line_segment([p1, p2], stroke));
                }
            }
        }
    }

    fn draw_snakes_parts(&self, shapes: &mut Vec<Shape>, vis: &VisibleArea) {
        let stroke = Stroke::new(2.0, SNAKE_COLOR);
        for r in vis.rows.clone() {
            for c in vis.cols.clone() {
                for to in self.snake_parts[r][c].iter() {
                    let p1 = self
                        .screen_transform
                        .to_screen(Point::new(c as f64 + 0.5, r as f64 + 0.5));
                    let p2 = self
                        .screen_transform
                        .to_screen(Point::new(to.col as f64 + 0.5, to.row as f64 + 0.5));
                    shapes.push(Shape::line_segment([p1, p2], stroke));
                }
            }
        }
    }

    fn draw_field(&self, shapes: &mut Vec<Shape>, vis: &VisibleArea) {
        for r in vis.rows.clone() {
            for c in vis.cols.clone() {
                let color = field_color(&self.a, self.min_cost, self.max_cost, r, c);
                let min = self
                    .screen_transform
                    .to_screen(Point::new(c as f64, r as f64));
                let max = self
                    .screen_transform
                    .to_screen(Point::new(c as f64 + 1.0, r as f64 + 1.0));
                shapes.push(Shape::rect_filled(
                    Rect::from_min_max(min, max),
                    Rounding::none(),
                    color,
                ));
            }
        }
    }

    fn calc_visible_area(&self, screen_window: Rect) -> VisibleArea {
        const SHIFT: i64 = 3;

        let p_min = self.screen_transform.from_screen(screen_window.min);
        let p_max = self.screen_transform.from_screen(screen_window.max);
        let rows = self.a.len();
        let cols = self.a[0].len();

        let calc_visible = |max: usize, from: f64, to: f64| -> Range<usize> {
            std::cmp::max(from as i64 - SHIFT, 0) as usize
                ..std::cmp::min(max as i64, to as i64 + SHIFT) as usize
        };

        VisibleArea {
            rows: calc_visible(rows, p_min.y, p_max.y),
            cols: calc_visible(cols, p_min.x, p_max.x),
        }
    }
}

fn field_min_max(a: &[Vec<Option<i64>>]) -> (i64, i64) {
    let mut min_cost = 0;
    let mut max_cost = 0;
    for i in 0..a.len() {
        for j in 0..a[i].len() {
            if let Some(value) = a[i][j] {
                min_cost = std::cmp::min(min_cost, value);
                max_cost = std::cmp::max(max_cost, value);
            }
        }
    }
    (min_cost, max_cost)
}

fn field_color(
    a: &[Vec<Option<i64>>],
    min_cost: i64,
    max_cost: i64,
    row: usize,
    col: usize,
) -> Color32 {
    if let Some(value) = a[row][col] {
        if value < 0 {
            let part = value as f64 / min_cost as f64;
            let r = (255.0 * part) as u8;
            Color32::from_rgb(r, 0, 0)
        } else if value > 0 {
            let part = value as f64 / max_cost as f64;
            let g = (255.0 * part) as u8;
            Color32::from_rgb(0, g, 0)
        } else {
            Color32::GRAY
        }
    } else {
        Color32::WHITE
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
            // ui.heading("My egui Application");

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
            {
                let min = self.screen_transform.to_screen(Point::new(0.0, 0.0));
                let max = self
                    .screen_transform
                    .to_screen(Point::new(self.a[0].len() as f64, self.a.len() as f64));
                shapes.push(Shape::image(
                    self.ret_image.texture_id(ctx),
                    Rect::from_min_max(min, max),
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                ));
            }
            shapes.extend(circle_shapes.into_iter().map(|c| Shape::Circle(c)));

            {
                let vis = self.calc_visible_area(rect);
                if !vis.is_too_big() {
                    self.draw_field(&mut shapes, &vis);
                    self.draw_snakes_parts(&mut shapes, &vis);
                }
                // eprintln!("Vis: {vis:?}, big: {}", vis.is_too_big());
            }

            ui.painter().with_clip_rect(rect).extend(shapes);
        });
    }
}
