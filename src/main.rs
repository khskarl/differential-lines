use nannou::prelude::*;
use std::f32::consts::PI;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    ps: ParticleSystem,
}

struct ParticleSystem {
    num_particles: usize,
    positions: Vec<Point2>,
    colors: Vec<Rgba<f32>>,
    edges: Vec<(usize, usize)>,
}

impl ParticleSystem {
    fn new() -> Self {
        let positions = Vec::new();
        let colors = Vec::new();
        let edges = Vec::new();

        ParticleSystem {
            num_particles: 0,
            positions,
            colors,
            edges,
        }
    }

    fn spawn_particles(&mut self, num_particles: usize) {
        let delta_phi = (2.0 * PI) / num_particles as f32;
        let mut phi = 0.0;

        for i in 0..num_particles {
            let direction = vec2(phi.cos(), phi.sin());
            let position = direction * 100.0;

            let l = random_f32() * 0.8 + 0.1;
            let color = Rgba::new(l, l - random_f32() * 0.2, l - random_f32() * 0.1, 1.0);

            self.positions.push(position);
            self.colors.push(color);

            let prev_particle = {
                if i as i32 - 1 < 0 {
                    num_particles - 1
                } else {
                    i - 1
                }
            };

            let next_particle = {
                if i + 1 == num_particles {
                    0
                } else {
                    i + 1
                }
            };

            self.edges.push((prev_particle, next_particle));

            phi += delta_phi;
        }

        self.num_particles = num_particles;
    }

    fn update(&mut self) {
        for i in 0..self.num_particles {
            let neighbors = self.get_neighbors_of_particle(i);

            let brotherhood_center = {
                let (b0, b1) = self.edges[i];
                (self.positions[b0] + self.positions[b1]) / 2.0
            };
            let direction_to_brotherhood = brotherhood_center - self.positions[i];
            self.positions[i] += direction_to_brotherhood.normalize() * 0.5;

            let hateship_center = {
                let quantity = neighbors.len() as f32;
                let mut center = vec2(0.0, 0.0);
                for j in neighbors {
                    center += self.positions[j];
                }
                center / quantity.max(1.0)
            };

            let direction_from_hate = self.positions[i] - hateship_center;
            self.positions[i] += direction_from_hate.normalize() * 0.3;

            {
                let (b0, b1) = self.edges[i];
                let (pos_b0, pos_b1) = (self.positions[b0], self.positions[b1]);
                let pos_i = self.positions[i];

                let to_b0 = (pos_b0 - pos_i).normalize();
                let to_b1 = (pos_b1 - pos_i).normalize();
                let to_center = ((pos_b0 + pos_b1 + pos_i) / 3.0 - pos_i).normalize();

                let tolerance = 0.05;
                if to_b0.dot(to_center).abs() < tolerance && random_f32() < 0.05 {
                    self.colors[b0] = Rgba::new(0.2, 0.3, 1.0, 1.0);
                    self.colors[i] = Rgba::new(0.2, 0.3, 1.0, 1.0);
                    self.split_at(b0, i);
                } else {
                    self.colors[b0] = Rgba::new(1.0, 0.3, 0.2, 1.0);
                    self.colors[i] = Rgba::new(1.0, 0.3, 0.2, 1.0);
                }

                if to_b1.dot(to_center).abs() < tolerance && random_f32() < 0.05 {
                    self.colors[i] = Rgba::new(0.2, 0.3, 1.0, 1.0);
                    self.colors[b1] = Rgba::new(0.2, 0.3, 1.0, 1.0);
                    self.split_at(i, b1);
                } else {
                    self.colors[i] = Rgba::new(1.0, 0.3, 0.2, 1.0);
                    self.colors[b1] = Rgba::new(1.0, 0.3, 0.2, 1.0);
                }
            }
        }
    }

    fn split_at(&mut self, p0: usize, p1: usize) {
        let new_index = self.positions.len();
        let new_pos = (self.positions[p0] + self.positions[p1]) / 2.0;
        let new_color = (self.colors[p0] + self.colors[p1]) / 2.0;

        self.positions.push(new_pos);
        self.colors.push(new_color);
        self.edges.push((p0, p1));
        self.num_particles += 1;

        self.edges[p0].1 = new_index;
        self.edges[p1].0 = new_index;
    }

    fn get_neighbors_of_particle(&self, index: usize) -> Vec<usize> {
        let mut neighbors = Vec::<usize>::new();

        for j in 0..self.num_particles {
            if index == j || j == self.edges[index].0 || j == self.edges[index].1 {
                continue;
            }

            let distance = (self.positions[index] - self.positions[j]).magnitude();

            let radius = 12.0;
            if distance <= radius {
                neighbors.push(j);
            }
        }

        neighbors
    }

    fn draw(&self, draw: &app::Draw) {
        let thickness = 0.1;

        for i in 0..self.edges.len() {
            let (_, next) = self.edges[i];

            draw.line()
                .start(self.positions[i])
                .end(self.positions[next])
                .thickness(thickness)
                .rgba(0.8, 0.8, 0.8, 0.1);
        }

        for i in 0..self.num_particles {
            let size = 4.0;

            draw.ellipse().xy(self.positions[i]).w_h(size, size).rgba(
                self.colors[i].red,
                self.colors[i].green,
                self.colors[i].blue,
                self.colors[i].alpha,
            );
        }
    }
}

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(800, 600)
        .view(view)
        .build()
        .unwrap();

    // let (_w, h) = app.window_rect().w_h();
    let mut ps = ParticleSystem::new();
    ps.spawn_particles(100);

    Model { ps }
}

fn update(_app: &App, m: &mut Model, _update: Update) {
    m.ps.update();
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    let draw = app.draw();
    // draw.background().color(Rgba::new(0.01, 0.01, 0.01, 0.2));
    draw.rect().w_h(1280.0, 720.0).rgba(0.01, 0.01, 0.01, 0.09);

    m.ps.draw(&draw);

    draw.to_frame(app, &frame).unwrap();

    frame
}
