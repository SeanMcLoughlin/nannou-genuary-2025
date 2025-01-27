use nannou::prelude::*;

struct Model {
    squares: Vec<Square>,
    time: u64,
}

struct Square {
    position: Point2,
    size: f32,
    phase: u8,
}

impl Square {
    fn new(x: f32, y: f32, size: f32) -> Self {
        Square {
            position: pt2(x, y),
            size,
            phase: 0,
        }
    }

    fn update(&mut self, time: u64) {
        // Systematic phase progression
        self.phase = ((time / 30) % 4) as u8;
    }

    fn draw(&self, draw: &Draw) {
        let color = match self.phase {
            0 => BLUE,
            1 => GREEN,
            2 => RED,
            3 => PURPLE,
            _ => BLACK,
        };

        // Size oscillation based on phase
        let scale = match self.phase {
            0 => 1.0,
            1 => 0.8,
            2 => 0.6,
            3 => 0.4,
            _ => 1.0,
        };

        draw.rect()
            .xy(self.position)
            .w_h(self.size * scale, self.size * scale)
            .color(color);
    }
}

fn model(app: &App) -> Model {
    app.new_window().size(800, 800).view(view).build().unwrap();

    // Create a 5x5 grid of squares
    let mut squares = Vec::new();
    let square_size = 100.0;
    let spacing = 120.0;
    let offset = -240.0; // Center the grid

    for i in 0..5 {
        for j in 0..5 {
            let x = offset + (i as f32 * spacing);
            let y = offset + (j as f32 * spacing);
            squares.push(Square::new(x, y, square_size));
        }
    }

    Model { squares, time: 0 }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.time += 1;

    // Update each square with a different timing offset based on position
    for (idx, square) in model.squares.iter_mut().enumerate() {
        let row = idx / 5;
        let col = idx % 5;
        let offset = (row + col) as u64 * 15; // Diagonal wave pattern
        square.update(model.time + offset);
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(LINEN);

    for square in &model.squares {
        square.draw(&draw);
    }

    watermark(&draw);
    draw.to_frame(app, &frame).unwrap();
}

fn watermark(draw: &Draw) {
    draw.text("1.27")
        .color(rgba(0.0, 0.0, 0.0, 0.5))
        .font_size(24)
        .align_text_bottom()
        .x_y(-(800.0 as f32) / 2.0 + 40.0, -(800.0 as f32) / 2.0 + 110.0);
}

fn main() {
    nannou::app(model).update(update).run();
}
