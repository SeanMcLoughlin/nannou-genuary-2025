use nannou::ease;
use nannou::prelude::*;
use rand::seq::SliceRandom;
use rand::SeedableRng;

const OS_WINDOW_WIDTH: u32 = 800;
const OS_WINDOW_HEIGHT: u32 = 800;
const BASE_SIZE: f32 = 60.0;
const ISO_ANGLE_RADIANS: f32 = 0.0;
const BUILDING_ANIMATION_SPEED: f32 = 0.5;
const PHI: f32 = 1.618033988749894848204586834365638118;
const BUILDING_HEIGHT: f32 = BASE_SIZE * PHI;
const NUM_WINDOW_ROWS: u32 = 3;
const NUM_WINDOW_COLS: u32 = 4;
const WINDOW_SIZE: f32 = 5.0;
const WINDOW_ISO_STAGGER_FACTOR: f32 = 15.0; // Would be nice to set in radians but oh well

const WINDOW_ANIMATION_DURATION: f32 = 3.0;
const WINDOW_ANIMATION_DELAY: f32 = 0.2; // Delay between windows appearing

struct Building {
    center: Point2,
    height: f32,
}

impl Building {
    fn new(center: Point2, height: f32) -> Self {
        Building { center, height }
    }

    pub fn draw(self, draw: &Draw) {
        let mut vertices = Vec::new();
        let ang = ISO_ANGLE_RADIANS;
        let size = BASE_SIZE;

        // Note that this makes vertices for two *diamonds* and not two *squares*.
        // This naturally provides an isometric perspective. But an angle parameter is still
        // provided in case it makes the end result look better.

        // Bottom face vertices
        vertices.push(self.center + vec2(-size * ang.cos(), -size * ang.sin())); // bottom left
        vertices.push(self.center + vec2(size * ang.cos(), -size * ang.sin())); // bottom right
        vertices.push(self.center + vec2(0.0, -size)); // bottom front
        vertices.push(self.center + vec2(0.0, size)); // bottom back

        // If looking from above, bottom face vertices are:
        //  3
        // 0 1
        //  2

        // Top face vertices are simply the bottom vertices with a height offset.
        vertices.push(vertices[0] + vec2(0.0, self.height)); // top left
        vertices.push(vertices[1] + vec2(0.0, self.height)); // top right
        vertices.push(vertices[2] + vec2(0.0, self.height)); // top front
        vertices.push(vertices[3] + vec2(0.0, self.height)); // top back

        // If looking from above, top face vertices are:
        //  7
        // 4 5
        //  6

        // The edge created by vertices 6 and 2 faces the camera.

        let right_color = rgba(0.0, 0.0, 0.0, 0.6);
        let right_vertices = vec![vertices[1], vertices[2], vertices[6], vertices[5]];
        draw.polygon().points(right_vertices).color(right_color);

        let left_color = rgba(0.0, 0.0, 0.0, 0.4);
        let left_vertices = vec![vertices[0], vertices[2], vertices[6], vertices[4]];
        draw.polygon().points(left_vertices).color(left_color);

        let top_color = rgba(0.0, 0.0, 0.0, 0.8);
        let top_vertices = vec![vertices[4], vertices[6], vertices[5], vertices[7]];
        draw.polygon().points(top_vertices).color(top_color);
    }
}

struct Model {
    building_height: f32,
    building_animation_progress: f32,
    window_animation_start_times: Vec<Vec<f32>>, // Time when each window starts animating
}

struct Window {
    row: usize,
    col: usize,
    side: String,
    pub vertices: Vec<Vec2>,
    pub scale: f32, // Current scale of the window
}

impl Window {
    fn new(row: usize, col: usize, side: String) -> Self {
        Window {
            row,
            col,
            side,
            vertices: Vec::new(),
            scale: 0.0,
        }
    }

    pub fn draw(&mut self, draw: &Draw, app_time: f32, start_times: &Vec<Vec<f32>>) {
        self.calculate_scale(app_time, start_times);
        self.calculate_vertices();
        let center = self.calculate_center();
        let scaled_vertices: Vec<Vec2> = self
            .vertices
            .iter()
            .map(|v| center + (*v - center) * self.scale)
            .collect();
        draw.polygon().points(scaled_vertices).color(LINEN);
    }

    fn calculate_scale(&mut self, app_time: f32, start_times: &Vec<Vec<f32>>) {
        let start_time = start_times[self.row][self.col];
        if app_time >= start_time {
            let progress = ((app_time - start_time) / WINDOW_ANIMATION_DURATION).min(1.0);
            // Use bounce ease out for the scale animation
            self.scale = ease::cubic::ease_out(progress, 0.0, 1.0, 1.0);
        }
    }

    fn calculate_vertices(&mut self) {
        let center: Vec2 = self.calculate_center();
        let size: f32 = WINDOW_SIZE;
        // Note: these each make *parallelograms* and not squares.
        if self.side == String::from("left") {
            self.vertices.push(center + vec2(-size, 2.0 * size)); // top left
            self.vertices.push(center + vec2(-size, 0.0)); // bottom left
            self.vertices.push(center + vec2(size, -2.0 * size)); // bottom right
            self.vertices.push(center + vec2(size, 0.0)); // top right
        } else {
            self.vertices.push(center + vec2(-size, 0.0)); // top left
            self.vertices.push(center + vec2(-size, -2.0 * size)); // bottom left
            self.vertices.push(center + vec2(size, 0.0)); // bottom right
            self.vertices.push(center + vec2(size, 2.0 * size));
            // top right
        }

        // Vertices appear like so:
        // 0 \
        // |   3
        // 1   |
        //   \ 2
        // And mirrored for each side of the building.
    }

    fn calculate_center(&mut self) -> Vec2 {
        let window_spacing_horizontal = BASE_SIZE / 4.0;
        let window_spacing_vertical = BUILDING_HEIGHT / (NUM_WINDOW_ROWS as f32 + 0.8);

        // Cascades the windows downwards as they approach the center of the image.
        let iso_stagger = if self.side == String::from("left") {
            -(self.col as f32 * WINDOW_ISO_STAGGER_FACTOR)
        } else {
            self.col as f32 * WINDOW_ISO_STAGGER_FACTOR
        };
        let row_offset = window_spacing_vertical * (self.row as f32 + 1.0) + iso_stagger;
        let col_offset = window_spacing_horizontal * (self.col as f32 + 1.0);

        // Fudging a bit here...
        let start_x = if self.side == String::from("left") {
            -BASE_SIZE - 7.5
        } else {
            -7.5
        };
        let start_y = if self.side == String::from("left") {
            0.0
        } else {
            -BUILDING_HEIGHT / 2.0 + 3.0
        };

        vec2(start_x + col_offset, start_y + row_offset)
    }
}

struct Windows {
    windows_left: Vec<Vec<Window>>,
    windows_right: Vec<Vec<Window>>,
}

impl Windows {
    fn new() -> Self {
        Windows {
            windows_left: Windows::get_windows("left"),
            windows_right: Windows::get_windows("right"),
        }
    }

    pub fn draw(&mut self, draw: &Draw, app_time: f32, start_times: &Vec<Vec<f32>>) {
        for windows in self
            .windows_left
            .iter_mut()
            .chain(self.windows_right.iter_mut())
        {
            for window in windows.iter_mut() {
                window.draw(draw, app_time, start_times);
            }
        }
    }

    fn get_windows(side: &str) -> Vec<Vec<Window>> {
        (0..NUM_WINDOW_ROWS as usize)
            .map(|i| {
                (0..NUM_WINDOW_COLS as usize)
                    .map(|j| Window::new(i, j, side.to_string()))
                    .collect()
            })
            .collect()
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

    // Create flat vector of all window indices
    let mut all_windows: Vec<(usize, usize)> = Vec::new();
    for i in 0..NUM_WINDOW_ROWS {
        for j in 0..NUM_WINDOW_COLS {
            all_windows.push((i as usize, j as usize));
        }
    }

    let mut rng = rand::rngs::StdRng::from_entropy();
    all_windows.shuffle(&mut rng);

    // Create animation start times matrix
    let mut window_animation_start_times =
        vec![vec![0.0; NUM_WINDOW_COLS as usize]; NUM_WINDOW_ROWS as usize];
    for (idx, (row, col)) in all_windows.iter().enumerate() {
        window_animation_start_times[*row][*col] = 1.0 + (idx as f32 * WINDOW_ANIMATION_DELAY);
    }

    Model {
        building_height: 0.0,
        building_animation_progress: 0.0,
        window_animation_start_times,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    model.building_animation_progress = (app.time * BUILDING_ANIMATION_SPEED).min(1.0);

    // Calculate building height based on animation progress
    // Parameters: current time, start value, change in value, duration
    model.building_height =
        ease::cubic::ease_out(model.building_animation_progress, 0.0, BUILDING_HEIGHT, 1.0);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(LINEN);

    Building::new(pt2(0.0, 0.0), model.building_height).draw(&draw);
    if model.building_animation_progress >= 1.0 {
        Windows::new().draw(&draw, app.time, &model.window_animation_start_times);
    }
    watermark(&draw);

    draw.to_frame(app, &frame).unwrap();
}

fn watermark(draw: &Draw) {
    draw.text("1.20")
        .color(rgba(0.0, 0.0, 0.0, 0.5))
        .font_size(24)
        .align_text_bottom()
        .x_y(
            -(OS_WINDOW_WIDTH as f32) / 2.0 + 40.0,
            -(OS_WINDOW_HEIGHT as f32) / 2.0 + 110.0,
        );
}
