use nannou::prelude::*;
use rand::Rng;

const PIXEL_GRID_WIDTH: usize = 200;
const PIXEL_GRID_HEIGHT: usize = 200;
const DISPLAY_WINDOW_WIDTH: u32 = 800;
const DISPLAY_WINDOW_HEIGHT: u32 = 800;
const NUM_SORTS_PER_FRAME: usize = 5000000;

#[derive(Copy, Clone, Debug)]
struct Pixel {
    color: Rgb8,
    idx: usize,
}

impl Ord for Pixel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}

impl PartialOrd for Pixel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Pixel {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl Eq for Pixel {}

struct Model {
    finished: bool,
    sorter: Box<BubbleSort<Pixel>>,
}

impl Model {
    fn new(current: Vec<Pixel>) -> Self {
        Model {
            finished: false,
            sorter: Box::new(BubbleSort::new(current.into_iter())),
        }
    }
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
    let mut colors = vec![Rgb8::new(0, 0, 0); PIXEL_GRID_WIDTH * PIXEL_GRID_HEIGHT];
    for y in 0..PIXEL_GRID_HEIGHT {
        for x in 0..PIXEL_GRID_WIDTH {
            let r = lerp(0.0, 255.0, y as f32 / PIXEL_GRID_HEIGHT as f32) as u8;
            let g = lerp(
                0.0,
                255.0,
                (x + y) as f32 / (PIXEL_GRID_WIDTH + PIXEL_GRID_HEIGHT) as f32,
            ) as u8;
            let b = lerp(255.0, 0.0, y as f32 / PIXEL_GRID_HEIGHT as f32) as u8;
            colors[y * PIXEL_GRID_WIDTH + x] = Rgb8::new(r, g, b);
        }
    }

    // Create target indices (sorted order)
    let mut current_indices: Vec<usize> = (0..colors.len()).collect();

    // Create randomized current state
    let mut rng = rand::thread_rng();
    for i in (1..colors.len()).rev() {
        let j = rng.gen_range(0..=i);
        colors.swap(i, j);
        current_indices.swap(i, j);
    }

    Model::new(
        colors
            .iter()
            .zip(current_indices.iter())
            .map(|(color, &idx)| Pixel { color: *color, idx })
            .collect(),
    )
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

pub struct BubbleSort<T>
where
    T: Ord + Clone,
{
    items: Vec<T>,
    did_swap: bool,
    index: usize,
    done: bool,
}

impl<T: Ord + Clone> BubbleSort<T> {
    pub fn new<I: Iterator<Item = T>>(iter: I) -> Self {
        BubbleSort {
            items: iter.collect(),
            did_swap: false,
            index: 0,
            done: false,
        }
    }
}

impl<T: Ord + Clone> Iterator for BubbleSort<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.items.is_empty() || self.done {
            return None;
        }

        if self.index >= self.items.len() - 1 {
            if !self.did_swap {
                self.done = true;
                return Some(self.items.clone());
            }
            self.index = 0;
            self.did_swap = false;
        }

        if self.items[self.index] > self.items[self.index + 1] {
            self.items.swap(self.index, self.index + 1);
            self.did_swap = true;
        }
        self.index += 1;

        Some(self.items.clone())
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if model.finished {
        return;
    }

    for _ in 0..NUM_SORTS_PER_FRAME {
        if model.sorter.next().is_none() {
            model.finished = true;
            break;
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    let pixel_size = DISPLAY_WINDOW_WIDTH as f32 / PIXEL_GRID_WIDTH as f32;

    // Draw current state
    for y in 0..PIXEL_GRID_HEIGHT {
        for x in 0..PIXEL_GRID_WIDTH {
            let idx = y * PIXEL_GRID_WIDTH + x;
            let color = model.sorter.items[idx].color;
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
