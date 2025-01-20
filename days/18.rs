//! What does wind look like?

use clap::Parser;
use nannou::noise::{NoiseFn, OpenSimplex, Perlin, Value};
use nannou::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about = "Wind visualization using nannou")]
struct Args {
    /// Type of noise to use (perlin, simplex, value)
    #[arg(short, long, default_value = "perlin")]
    noise_type: String,

    /// Window width
    #[arg(long, default_value_t = 800)]
    width: u32,

    /// Window height
    #[arg(long, default_value_t = 800)]
    height: u32,

    /// Particle life duration (higher = longer trails)
    #[arg(short, long, default_value_t = 0.005)]
    life_reduction: f32,

    /// Maximum number of particles
    #[arg(short, long, default_value_t = 1000)]
    max_particles: usize,
}

enum NoiseGenerator {
    Perlin(Perlin),
    Simplex(OpenSimplex),
    Value(Value),
}

impl NoiseGenerator {
    fn get_noise(&self, x: f64, y: f64, z: f64) -> f64 {
        match self {
            NoiseGenerator::Perlin(noise) => noise.get([x, y, z]),
            NoiseGenerator::Simplex(noise) => noise.get([x, y, z]),
            NoiseGenerator::Value(noise) => noise.get([x, y, z]),
        }
    }
}

struct Model {
    particles: Vec<Particle>,
    noise: NoiseGenerator,
    flow_field: Vec<Vec2>,
    grid_size: usize,
    cell_size: f32,
    args: Args,
}

struct Particle {
    position: Point2,
    velocity: Vec2,
    prev_position: Point2,
    life: f32,
}

impl Particle {
    fn new(x: f32, y: f32) -> Self {
        Particle {
            position: pt2(x, y),
            velocity: vec2(0.0, 0.0),
            prev_position: pt2(x, y),
            life: random_range(0.5, 1.0),
        }
    }

    fn update(
        &mut self,
        rect: Rect,
        flow_field: &[Vec2],
        grid_size: usize,
        cell_size: f32,
        life_reduction: f32,
    ) {
        self.prev_position = self.position;

        // Get grid position
        let grid_x = ((self.position.x - rect.left()) / cell_size).floor() as usize;
        let grid_y = ((self.position.y - rect.bottom()) / cell_size).floor() as usize;

        // Ensure we're within bounds
        if grid_x < grid_size && grid_y < grid_size {
            let index = grid_y * grid_size + grid_x;
            if index < flow_field.len() {
                // Apply force from flow field
                let force = flow_field[index];
                self.velocity += force * 0.5;
            }
        }

        // Update position
        self.velocity = self.velocity.clamp_length_max(2.0);
        self.position += self.velocity;

        // Reduce life
        self.life -= life_reduction;

        // Wrap around edges
        if self.position.x < rect.left() {
            self.position.x = rect.right();
            self.prev_position.x = rect.right();
        }
        if self.position.x > rect.right() {
            self.position.x = rect.left();
            self.prev_position.x = rect.left();
        }
        if self.position.y < rect.bottom() {
            self.position.y = rect.top();
            self.prev_position.y = rect.top();
        }
        if self.position.y > rect.top() {
            self.position.y = rect.bottom();
            self.prev_position.y = rect.bottom();
        }
    }
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let args = Args::parse();
    let _window = app
        .new_window()
        .size(args.width, args.height)
        .view(view)
        .build()
        .unwrap();

    let grid_size = 32;
    let cell_size = args.width as f32 / grid_size as f32;

    // Initialize noise generator based on argument
    let noise = match args.noise_type.to_lowercase().as_str() {
        "simplex" => NoiseGenerator::Simplex(OpenSimplex::new()),
        "value" => NoiseGenerator::Value(Value::new()),
        _ => NoiseGenerator::Perlin(Perlin::new()),
    };

    // Initialize flow field
    let mut flow_field = Vec::with_capacity(grid_size * grid_size);

    for y in 0..grid_size {
        for x in 0..grid_size {
            let angle = noise.get_noise(x as f64 * 0.1, y as f64 * 0.1, app.time as f64 * 0.1)
                * core::f64::consts::PI
                * 2.0;

            flow_field.push(vec2(angle.cos() as f32, angle.sin() as f32));
        }
    }

    // Create initial particles
    let particles = (0..args.max_particles)
        .map(|_| {
            Particle::new(
                random_range(-(args.width as f32) / 2.0, args.width as f32 / 2.0),
                random_range(-(args.height as f32) / 2.0, args.height as f32 / 2.0),
            )
        })
        .collect();

    Model {
        particles,
        noise,
        flow_field,
        grid_size,
        cell_size,
        args,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // Update flow field
    model.flow_field.clear();
    for y in 0..model.grid_size {
        for x in 0..model.grid_size {
            let angle =
                model
                    .noise
                    .get_noise(x as f64 * 0.1, y as f64 * 0.1, app.time as f64 * 0.1)
                    * core::f64::consts::PI
                    * 2.0;

            model
                .flow_field
                .push(vec2(angle.cos() as f32, angle.sin() as f32));
        }
    }

    // Update particles
    let rect = app.window_rect();
    for particle in &mut model.particles {
        particle.update(
            rect,
            &model.flow_field,
            model.grid_size,
            model.cell_size,
            model.args.life_reduction,
        );
    }

    // Remove dead particles and add new ones
    model.particles.retain(|p| p.life > 0.0);
    while model.particles.len() < model.args.max_particles {
        model.particles.push(Particle::new(
            random_range(
                -(model.args.width as f32) / 2.0,
                model.args.width as f32 / 2.0,
            ),
            random_range(
                -(model.args.height as f32) / 2.0,
                model.args.height as f32 / 2.0,
            ),
        ));
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    // Clear with a dark background
    draw.background().color(LINEN);

    // Draw date in bottom left
    draw.text("1.18")
        .color(rgba(0.0, 0.0, 0.0, 0.5))
        .font_size(24)
        .align_text_bottom()
        .x_y(
            -(model.args.width as f32) / 2.0 + 40.0,
            -(model.args.height as f32) / 2.0 + 110.0,
        );

    // Draw particles as lines from previous position
    for particle in &model.particles {
        draw.line()
            .start(particle.prev_position)
            .end(particle.position)
            .color(rgba(0.0, 0.0, 0.0, particle.life))
            .stroke_weight(2.0);
    }

    draw.to_frame(app, &frame).unwrap();
}
