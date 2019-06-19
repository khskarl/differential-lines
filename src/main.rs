use nannou::prelude::*;
use std::f32::consts::PI;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    ps: ParticleSystem,
}

struct ParticleSystem {
    particle_radius: f32,
    influence_radius: f32,
    num_particles: usize,
    positions: Vec<Point2>,
    colors: Vec<Rgba<f32>>,
    edges: Vec<(usize, usize)>,
    pressures: Vec<Vector2>,
    attractions: Vec<Vector2>,
}

impl ParticleSystem {
    fn new() -> Self {
        let positions = Vec::new();
        let colors = Vec::new();
        let edges = Vec::new();
        let pressures = Vec::new();
        let attractions = Vec::new();

        ParticleSystem {
            particle_radius: 4.0,
            influence_radius: 50.0,
            num_particles: 0,
            positions,
            colors,
            edges,
            pressures,
            attractions,
        }
    }

    fn add_particle(
        &mut self,
        position: Point2,
        color: Rgba<f32>,
        edges: (usize, usize),
        pressure: Vector2,
        attraction: Vector2,
    ) {
        self.positions.push(position);
        self.colors.push(color);
        self.edges.push(edges);
        self.pressures.push(pressure);
        self.attractions.push(attraction);
        self.num_particles += 1;
    }


    fn spawn_particles(&mut self, num_particles: usize, spawn_radius: f32) {
        let delta_phi = (2.0 * PI) / num_particles as f32;
        let mut phi = 0.0;

        for i in 0..num_particles {
            let direction = vec2(phi.cos(), phi.sin());
            let position = direction * spawn_radius;

            let l = random_f32() * 0.8 + 0.1;
            let color = Rgba::new(l, l - random_f32() * 0.2, l - random_f32() * 0.1, 1.0);

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

            let edges = (prev_particle, next_particle);
            let pressure = vec2(0.0, 0.0);
            let attraction = vec2(0.0, 0.0);

            self.add_particle(position, color, edges, pressure, attraction);

            phi += delta_phi;
        }

        self.num_particles = num_particles;
    }

    fn update(&mut self) {

        let old_positions = self.positions.clone();
        for i in 0..self.num_particles {
            let neighbors = self.get_neighbors_of_particle(i);
            let brotherhood_center = {
                let (b0, b1) = self.edges[i];
                (old_positions[b0] + old_positions[b1]) / 2.0
            };
            self.attractions[i] = brotherhood_center - old_positions[i];
            self.positions[i] += (brotherhood_center - old_positions[i]).normalize() * 0.5;
            let hateship_center = {
                let quantity = neighbors.len() as f32;
                let mut center = vec2(0.0, 0.0);
                if quantity < 1.0 {
                    center = old_positions[i];
                }
                for j in neighbors {
                    center += old_positions[j];
                }
                center / quantity.max(1.0)
            };

            self.pressures[i] = old_positions[i] - hateship_center;
            self.positions[i] += (old_positions[i] - hateship_center).normalize() * 0.5;
        }

        for i in 0..self.num_particles {
            // let (b0, b1) = self.edges[i];
            // let (pos_b0, pos_b1) = (self.positions[b0], self.positions[b1]);
            // let pos_i = self.positions[i];

            // let to_b0 = (pos_b0 - pos_i).normalize();
            // let to_b1 = (pos_b1 - pos_i).normalize();
            // let to_center = ((pos_b0 + pos_b1 + pos_i) / 3.0 - pos_i).normalize();

            // let tolerance = 0.05;
            // if to_b0.dot(to_center).abs() < tolerance && random_f32() < 0.05 {
            //     self.colors[b0] = Rgba::new(0.2, 0.3, 1.0, 1.0);
            //     self.colors[i] = Rgba::new(0.2, 0.3, 1.0, 1.0);
            //     self.split_at(b0, i);
            // } else {
            //     self.colors[b0] = Rgba::new(1.0, 0.3, 0.2, 1.0);
            //     self.colors[i] = Rgba::new(1.0, 0.3, 0.2, 1.0);
            // }

            // if to_b1.dot(to_center).abs() < tolerance && random_f32() < 0.05 {
            //     self.colors[i] = Rgba::new(0.2, 0.3, 1.0, 1.0);
            //     self.colors[b1] = Rgba::new(0.2, 0.3, 1.0, 1.0);
            //     self.split_at(i, b1);
            // } else {
            //     self.colors[i] = Rgba::new(1.0, 0.3, 0.2, 1.0);
            //     self.colors[b1] = Rgba::new(1.0, 0.3, 0.2, 1.0);
            // }
        }
    }


    fn split_at(&mut self, p0: usize, p1: usize) {
        let new_index = self.positions.len();

        let position = (self.positions[p0] + self.positions[p1]) / 2.0;
        let color = (self.colors[p0] + self.colors[p1]) / 2.0;
        let edges = (p0, p1);
        let pressure = vec2(0.0, 0.0);
        let attraction = vec2(0.0, 0.0);

        self.edges[p0].1 = new_index;
        self.edges[p1].0 = new_index;
        self.add_particle(position, color, edges, pressure, attraction);
    }

    fn get_neighbors_of_particle(&self, index: usize) -> Vec<usize> {
        let mut neighbors = Vec::<usize>::new();

        for j in 0..self.num_particles {
            if index == j || j == self.edges[index].0 || j == self.edges[index].1 {
                continue;
            }

            let distance = (self.positions[index] - self.positions[j]).magnitude();

            if distance <= self.influence_radius {
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
            let size = self.particle_radius;

            draw.ellipse().xy(self.positions[i]).w_h(size, size).rgba(
                self.colors[i].red,
                self.colors[i].green,
                self.colors[i].blue,
                self.colors[i].alpha,
            );


            draw.line()
                .start(self.positions[i])
                .end(self.positions[i] + self.pressures[i])
                .thickness(thickness)
                .rgba(1.0, 0.6, 0.6, 1.0);

            draw.line()
                .start(self.positions[i])
                .end(self.positions[i] + self.attractions[i])
                .thickness(thickness)
                .rgba(0.6, 1.0, 0.6, 1.0);
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
    let num_particles = 20;
    let spawn_radius = 150.0;
    ps.spawn_particles(num_particles, spawn_radius);

    Model { ps }
}

fn update(_app: &App, m: &mut Model, _update: Update) {
    m.ps.update();
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    let draw = app.draw();
    draw.background().color(Rgba::new(0.01, 0.01, 0.01, 0.2));
    // draw.rect().w_h(1280.0, 720.0).rgba(0.01, 0.01, 0.01, 0.09);

    m.ps.draw(&draw);

    draw.to_frame(app, &frame).unwrap();

    frame
}
