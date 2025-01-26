use nannou::prelude::*;

const OS_WINDOW_WIDTH: u32 = 800;
const OS_WINDOW_HEIGHT: u32 = 800;

struct Model {
    time: f32,
    num_points: usize,
    radius: f32,
    pulse_phase: f32,
    rotation_speed: f32,
    color_shift: f32,
    particle_systems: Vec<ParticleSystem>,
}

struct Particle {
    position: Point2,
    velocity: Vec2,
    life: f32,
    max_life: f32,
    color: Hsla,
}

struct ParticleSystem {
    particles: Vec<Particle>,
    origin: Point2,
    color: Hsla,
}

impl ParticleSystem {
    fn new(origin: Point2, color: Hsla) -> Self {
        ParticleSystem {
            particles: Vec::new(),
            origin,
            color,
        }
    }

    fn update(&mut self, _time: f32) {
        // Remove dead particles
        self.particles.retain(|p| p.life > 0.0);

        // Update existing particles
        for particle in &mut self.particles {
            particle.position += particle.velocity;
            particle.life -= 1.0;
            particle.velocity *= 0.98; // Add drag
        }

        // Add new particles with symmetrical distribution
        if random_f32() < 0.3 {
            let angle = random_f32() * TAU;
            let speed = random_range(0.5, 2.0);
            let velocity = vec2(angle.cos() * speed, angle.sin() * speed);
            let life = random_range(50.0, 150.0);

            self.particles.push(Particle {
                position: self.origin,
                velocity,
                life,
                max_life: life,
                color: self.color,
            });
        }
    }

    fn draw(&self, draw: &Draw) {
        for particle in &self.particles {
            let alpha = particle.life / particle.max_life;
            let color = hsla(
                particle.color.hue.into(),
                particle.color.saturation,
                particle.color.lightness,
                alpha,
            );

            draw.ellipse()
                .xy(particle.position)
                .w_h(3.0, 3.0)
                .color(color);
        }
    }
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(OS_WINDOW_WIDTH, OS_WINDOW_HEIGHT)
        .view(view)
        .build()
        .unwrap();

    Model {
        time: 0.0,
        num_points: 6,
        radius: 200.0,
        pulse_phase: 0.0,
        rotation_speed: 1.0,
        color_shift: 0.0,
        particle_systems: Vec::new(),
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    model.time = app.time;
    model.pulse_phase += 0.02;
    model.color_shift += 0.005;
    model.rotation_speed = 1.0 + (model.time * 0.1).sin() * 0.5;

    // Update particle systems
    for system in &mut model.particle_systems {
        system.update(model.time);
    }

    // Periodically reset particle systems
    if model.time.floor() != (model.time - _update.since_last.as_secs_f32()).floor() {
        model.particle_systems.clear();

        // Create new particle systems at symmetrical points
        for i in 0..model.num_points {
            let angle = (i as f32 / model.num_points as f32) * TAU;
            let radius = model.radius * 0.5;
            let origin = pt2(angle.cos() * radius, angle.sin() * radius);
            let hue = (model.color_shift + i as f32 / model.num_points as f32) % 1.0;
            let color = hsla(hue, 0.5, 0.5, 1.0);

            model
                .particle_systems
                .push(ParticleSystem::new(origin, color));
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let center = pt2(0.0, 0.0);
    let pulse = (model.pulse_phase.sin() * 0.2 + 1.0) * 0.5;

    // Draw shimmering background patterns
    for i in 0..8 {
        let phase = model.time * model.rotation_speed + i as f32 * PI / 4.0;
        let scale = (1.0 - (i as f32 * 0.1)) * pulse;
        let hue = (model.color_shift + i as f32 / 8.0) % 1.0;

        for j in 0..model.num_points {
            let angle = (j as f32 / model.num_points as f32) * TAU + phase;
            let point = center
                + vec2(
                    angle.cos() * model.radius * scale,
                    angle.sin() * model.radius * scale,
                );

            let next_angle = ((j + 1) as f32 / model.num_points as f32) * TAU + phase;
            let next_point = center
                + vec2(
                    next_angle.cos() * model.radius * scale,
                    next_angle.sin() * model.radius * scale,
                );

            // Draw main lines with glow effect
            for k in 0..3 {
                let alpha = 0.2 - (k as f32 * 0.05);
                let weight = 2.0 + (k as f32 * 2.0);

                draw.line()
                    .start(point)
                    .end(next_point)
                    .color(hsla(hue, 0.5, 0.5, alpha))
                    .stroke_weight(weight);
            }
        }
    }

    // Draw particle systems
    for system in &model.particle_systems {
        system.draw(&draw);
    }

    // Draw kaleidoscopic overlay
    let overlay_points: Vec<_> = (0..model.num_points * 2)
        .map(|i| {
            let angle = (i as f32 / (model.num_points * 2) as f32) * TAU;
            let r = model.radius * 0.3 * (1.0 + (model.time * 2.0 + angle * 2.0).sin() * 0.1);
            center + vec2(angle.cos() * r, angle.sin() * r)
        })
        .collect();

    for i in 0..overlay_points.len() {
        for j in i + 1..overlay_points.len() {
            let alpha = ((model.time + i as f32 * 0.1).sin() * 0.15 + 0.15).max(0.0);
            draw.line()
                .start(overlay_points[i])
                .end(overlay_points[j])
                .color(hsla(model.color_shift, 0.5, 0.5, alpha))
                .stroke_weight(1.0);
        }
    }

    watermark(&draw);
    draw.to_frame(app, &frame).unwrap();
}

fn watermark(draw: &Draw) {
    draw.text("1.26")
        .color(LINEN)
        .font_size(24)
        .align_text_bottom()
        .x_y(
            -(OS_WINDOW_WIDTH as f32) / 2.0 + 40.0,
            -(OS_WINDOW_HEIGHT as f32) / 2.0 + 110.0,
        );
}
