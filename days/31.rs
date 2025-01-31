use nannou::prelude::*;
use rand::Rng;

const PIXEL_GRID_WIDTH: usize = 200;
const PIXEL_GRID_HEIGHT: usize = 200;
const DISPLAY_WINDOW_WIDTH: u32 = 800;
const DISPLAY_WINDOW_HEIGHT: u32 = 800;
const STEPS_PER_RANDOMIZATION: u32 = 50;
const NUM_RANDOMIZATIONS: usize = 2000;

struct Model {
    target: Vec<Rgb8>,
    current: Vec<Rgb8>,
    indices: Vec<usize>,
    randomization_step: usize,
    finished: bool,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(DISPLAY_WINDOW_WIDTH as u32, DISPLAY_WINDOW_HEIGHT as u32)
        .view(view)
        .build()
        .unwrap();

    // Generate target gradient
    let mut target = vec![Rgb8::new(0, 0, 0); PIXEL_GRID_WIDTH * PIXEL_GRID_HEIGHT];
    for y in 0..PIXEL_GRID_HEIGHT {
        for x in 0..PIXEL_GRID_WIDTH {
            let r = lerp(0.0, 255.0, y as f32 / PIXEL_GRID_HEIGHT as f32) as u8;
            let g = lerp(
                0.0,
                255.0,
                (x + y) as f32 / (PIXEL_GRID_WIDTH + PIXEL_GRID_HEIGHT) as f32,
            ) as u8;
            let b = lerp(255.0, 0.0, y as f32 / PIXEL_GRID_HEIGHT as f32) as u8;
            target[y * PIXEL_GRID_WIDTH + x] = Rgb8::new(r, g, b);
        }
    }

    // Start with ordered indices
    let indices: Vec<usize> = (0..target.len()).collect();

    Model {
        target: target.clone(),
        current: target,
        indices,
        randomization_step: 0,
        finished: false,
    }
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if model.finished {
        return;
    }

    let mut rng = nannou::rand::thread_rng();

    // Perform random swaps
    if model.randomization_step > 2 {
        for _ in 0..STEPS_PER_RANDOMIZATION {
            let i = rng.gen_range(0..model.indices.len());
            let j = rng.gen_range(0..model.indices.len());
            model.indices.swap(i, j);
        }
    }

    // Update current display
    model.current = model.indices.iter().map(|&i| model.target[i]).collect();

    model.randomization_step += 1;

    // Stop after certain number of steps
    if model.randomization_step > NUM_RANDOMIZATIONS {
        model.finished = true;
    }

    // Show target and delay at start
    if model.randomization_step == 2 {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    let pixel_size = DISPLAY_WINDOW_WIDTH as f32 / PIXEL_GRID_WIDTH as f32;

    // Draw current state
    for y in 0..PIXEL_GRID_HEIGHT {
        for x in 0..PIXEL_GRID_WIDTH {
            let idx = y * PIXEL_GRID_WIDTH + x;
            let color = model.current[idx];
            let out_min = -(DISPLAY_WINDOW_WIDTH as i32) as f32 / 2.0;
            let out_max = DISPLAY_WINDOW_WIDTH as f32 / 2.0;
            draw.rect()
                .x_y(
                    map_range(x as f32, 0.0, PIXEL_GRID_WIDTH as f32, out_min, out_max),
                    map_range(y as f32, 0.0, PIXEL_GRID_HEIGHT as f32, out_min, out_max),
                )
                .w_h(pixel_size, pixel_size)
                .color(color);
        }
    }

    watermark(&draw);
    draw.to_frame(app, &frame).unwrap();
}

fn watermark(draw: &Draw) {
    draw.text("1.31")
        .color(WHITE)
        .font_size(24)
        .align_text_bottom()
        .x_y(
            -(DISPLAY_WINDOW_WIDTH as f32) / 2.0 + 40.0,
            -(DISPLAY_WINDOW_HEIGHT as f32) / 2.0 + 110.0,
        );
}
