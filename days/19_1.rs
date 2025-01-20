//! Op art.
//! With zoom.

use clap::Parser;
use nannou::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about = "Wind visualization using nannou")]
struct Args {
    /// Window width
    #[arg(long, default_value_t = 800)]
    width: u32,

    /// Window height
    #[arg(long, default_value_t = 800)]
    height: u32,

    #[arg(long, default_value_t = 0.001)]
    rotation_speed: f32,

    /// Speed of zoom effect
    #[arg(long, default_value_t = 0.01)]
    zoom_speed: f32,

    /// Number of zig-zag lines
    #[arg(long, default_value_t = 72)]
    num_lines: u32,

    /// The radius of the circle that the lines form
    #[arg(long, default_value_t = 350.0)]
    radius: f32,

    /// Factor of how zig-zaggy each line is
    #[arg(long, default_value_t = 5.0)]
    zig_zagginess: f32,
}

struct Model {
    width: u32,
    height: u32,
    rotation: f32,
    rotation_speed: f32,
    zoom: f32,
    zoom_speed: f32,
    num_lines: u32,
    radius: f32,
    zig_zagginess: f32,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let args = Args::parse();
    app.new_window()
        .size(args.width, args.height)
        .view(view)
        .build()
        .unwrap();

    Model {
        width: args.width,
        height: args.height,
        rotation: 0.0,
        rotation_speed: args.rotation_speed,
        zoom: 1.0, // Initial zoom state
        zoom_speed: args.zoom_speed,
        num_lines: args.num_lines,
        radius: args.radius,
        zig_zagginess: args.zig_zagginess,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.rotation += model.rotation_speed;
    model.zoom += model.zoom_speed;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(LINEN);

    let center = pt2(0.0, 0.0);
    let angle_step = TAU / model.num_lines as f32;
    let effective_radius = model.radius * model.zoom;

    for i in 0..model.num_lines {
        let angle = i as f32 * angle_step + model.rotation;
        let mut points = Vec::new();

        let segments = 20;
        let segment_length = effective_radius / segments as f32;
        let zigzag_width = angle_step * model.zig_zagginess;

        for j in 0..=segments {
            let dist = j as f32 * segment_length;
            let base_dist = dist / model.zoom; // Unscaled distance for zigzag calculation
            let offset = if j % 2 == 0 {
                zigzag_width
            } else {
                -zigzag_width
            };

            // Scale the zigzag effect based on distance from center
            let point_angle = angle + (offset * (1.0 - base_dist / model.radius));

            let x = center.x + dist * point_angle.cos();
            let y = center.y + dist * point_angle.sin();
            points.push(pt2(x, y));
        }

        draw.polyline()
            .stroke_weight(2.0)
            .points(points)
            .color(BLACK);
    }

    watermark(model, &draw);
    draw.to_frame(app, &frame).unwrap();
}

fn watermark(model: &Model, draw: &Draw) {
    draw.text("1.19")
        .color(rgba(0.0, 0.0, 0.0, 0.5))
        .font_size(24)
        .align_text_bottom()
        .x_y(
            -(model.width as f32) / 2.0 + 40.0,
            -(model.height as f32) / 2.0 + 110.0,
        );
}
