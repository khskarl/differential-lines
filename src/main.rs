use nannou::prelude::*;
use std::f32::consts::PI;

fn main() {
    nannou::app(model).update(update).run();
}

fn wrap(num: i32, max: i32) -> usize {
    let wrapped = if num < 0 {
        max - 1
    } else if num == max {
        0
    } else {
        num
    };

    wrapped as usize
}

struct Model {
    ps: ParticleSystem,
}

struct ParticleSystem {
    particle_radius: f32,
    influence_radius: f32,
    max_pressure_index: usize,
    max_attraction_index: usize,
    max_neighbors_index: usize,
    num_particles: usize,
    positions: Vec<Point2>,
    colors: Vec<Rgba<f32>>,
    edges: Vec<(usize, usize)>,
    pressures: Vec<Vector2>,
    attractions: Vec<Vector2>,
    num_neighbors: Vec<usize>,
}

impl ParticleSystem {
    fn new() -> Self {
        let positions = Vec::new();
        let colors = Vec::new();
        let edges = Vec::new();
        let pressures = Vec::new();
        let attractions = Vec::new();
        let num_neighbors = Vec::new();

        ParticleSystem {
            particle_radius: 4.0,
            influence_radius: 12.0,
            max_pressure_index: 0,
            max_attraction_index: 0,
            max_neighbors_index: 0,
            num_particles: 0,
            positions,
            colors,
            edges,
            pressures,
            attractions,
            num_neighbors,
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
        self.num_neighbors.push(0);
        self.num_particles += 1;
    }


    fn spawn_particles(&mut self, num_particles: usize, spawn_radius: f32) {
        let delta_phi = (2.0 * PI) / num_particles as f32;
        let mut phi = 0.0;

        for i in 0..num_particles {
            let direction = vec2(phi.cos(), phi.sin());
            let offset = (phi * 6.2).sin() * 50.0;
            let position = direction * (spawn_radius + offset);

            let l = random_f32() * 0.8 + 0.1;
            let color = Rgba::new(l, l - random_f32() * 0.2, l - random_f32() * 0.1, 1.0);

            let prev_particle = wrap(i as i32 - 1, num_particles as i32);
            let next_particle = wrap(i as i32 + 1, num_particles as i32);

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
            self.num_neighbors[i] = neighbors.len();

            if self.num_neighbors[self.max_neighbors_index] < neighbors.len() {
                self.max_neighbors_index = i;
            }

            let attraction = {
                let (b0, b1) = self.edges[i];
                (old_positions[b0] + old_positions[b1]) / 2.0 - old_positions[i]
            };
            self.attractions[i] = attraction;
            self.positions[i] += attraction * 0.6;
            if self.attractions[self.max_attraction_index].magnitude() < attraction.magnitude() {
                self.max_attraction_index = i;
            }

            let pressure = {
                let mut pressure = vec2(0.0, 0.0);
                for j in neighbors {
                    pressure +=
                        (self.positions[i] - self.positions[j]) / (self.influence_radius * 0.5);
                }

                pressure.limit_magnitude(2.0)
            };
            self.pressures[i] = pressure;
            self.positions[i] += (pressure) * 0.2;
            if self.pressures[self.max_pressure_index].magnitude() < pressure.magnitude() {
                self.max_pressure_index = i;
            }
        }

        for i in 0..self.num_particles {
            let p =
                self.pressures[i].magnitude() / self.pressures[self.max_pressure_index].magnitude();
            let a = self.attractions[i].magnitude()
                / self.attractions[self.max_attraction_index].magnitude();
            let n = 1.0
                - self.num_neighbors[i] as f32
                    / self.num_neighbors[self.max_neighbors_index] as f32;
            self.colors[i] = Rgba::new(p, a, p * a + 0.1, 1.0);
        }

        for e in 0..self.edges.len() {
            let (p0, p1) = (e, self.edges[e].1);
            let avg_pressure = (self.pressures[p0] + self.pressures[p1]) / 2.0;
            let relative_magnitude =
                avg_pressure.magnitude() / self.pressures[self.max_pressure_index].magnitude();

            let tolerance = 0.05;
            if self.num_neighbors[p0] + self.num_neighbors[p1] < 16 && random_f32() < 0.05 {
                // self.colors[b0] = Rgba::new(0.2, 0.3, 1.0, 1.0);
                // self.colors[i] = Rgba::new(0.2, 0.3, 1.0, 1.0);
                self.split_at(p0, p1);
            } else {
                // self.colors[b0] = Rgba::new(1.0, 0.3, 0.2, 1.0);
                // self.colors[i] = Rgba::new(1.0, 0.3, 0.2, 1.0);
            }
        }
    }


    fn split_at(&mut self, p0: usize, p1: usize) {
        let new_index = self.positions.len();

        let position = (self.positions[p0] + self.positions[p1]) / 2.0
            + self.pressures[p0]
            + self.pressures[p1];
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
            if index == j {
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

            // draw.line()
            //     .start(self.positions[i])
            //     .end(self.positions[i] + self.pressures[i] * 2.0)
            //     .thickness(thickness * 10.0)
            //     .rgba(1.0, 0.3, 0.3, 1.0);

            // draw.line()
            //     .start(self.positions[i])
            //     .end(self.positions[i] + self.attractions[i] * 2.0)
            //     .thickness(thickness * 10.0)
            //     .rgba(0.3, 1.0, 0.3, 1.0);

            // draw.line()
            //     .start(self.positions[i])
            //     .end(self.positions[i] + (self.attractions[i] + self.pressures[i]) * 2.0)
            //     .thickness(thickness * 30.0)
            //     .rgba(1.0, 1.0, 1.0, 1.0);
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
    let num_particles = 100;
    let spawn_radius = 100.0;
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
