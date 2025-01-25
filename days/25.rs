extern crate time;
extern crate travelling_salesman;
use nannou::prelude::*;

const OS_WINDOW_WIDTH: u32 = 800;
const OS_WINDOW_HEIGHT: u32 = 800;
const NUM_COORDS: usize = 50;
const SOLUTION_VIEW_TIME: f32 = 0.5;
const COORDS_ANIMATION_SPEED: f32 = 0.05;
const EDGES_ANIMATION_SPEED: f32 = 0.4;
const MAX_TSP_SOLUTION_TIME_MILLISECONDS: i64 = 200;

#[derive(Clone)]
enum ModelState {
    DrawingEdges,    // Draw the solution connecting all points
    ViewingSolution, // Pause to view the complete solution
    MovingCoords,    // Move the coordinates to a new random location
}

struct ModelAnimationProgress {
    coord_animation_progress: Vec<f32>,
    edge_animation_progress: f32,
    solution_view_progress: f32,
}

struct Model {
    coords: Vec<Point2>,        // Current coordinates
    target_coords: Vec<Point2>, // Random target coordinates to move to
    animations: ModelAnimationProgress,
    state: ModelState,
    current_tour: Vec<usize>, // Current TSP solution
    tour_length: f64,         // Length of current tour
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

    // Initialize all points at the center
    let mut coords = Vec::new();
    let mut target_coords = Vec::new();
    let mut coord_animation_progress = Vec::new();

    for _ in 0..NUM_COORDS {
        coords.push(pt2(0.0, 0.0));
        target_coords.push(random_point());
        coord_animation_progress.push(0.0);
    }

    Model {
        coords,
        target_coords,
        animations: ModelAnimationProgress {
            coord_animation_progress,
            edge_animation_progress: 0.0,
            solution_view_progress: 0.0,
        },
        state: ModelState::MovingCoords,
        current_tour: Vec::new(),
        tour_length: 0.0,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    match model.state {
        ModelState::MovingCoords => update_moving_coords(model),
        ModelState::DrawingEdges => update_drawing_edges(model),
        ModelState::ViewingSolution => update_viewing_solution(model, update),
    }
}

fn update_moving_coords(model: &mut Model) {
    let mut all_arrived = true;

    for i in 0..NUM_COORDS {
        model.animations.coord_animation_progress[i] += COORDS_ANIMATION_SPEED;
        if model.animations.coord_animation_progress[i] > 1.0 {
            model.animations.coord_animation_progress[i] = 1.0;
            model.coords[i] = model.target_coords[i];
        } else {
            all_arrived = false;
            // Interpolate between current and target position
            let t = model.animations.coord_animation_progress[i];
            model.coords[i] = pt2(
                lerp(model.coords[i].x, model.target_coords[i].x, t),
                lerp(model.coords[i].y, model.target_coords[i].y, t),
            );
        }
    }

    if all_arrived {
        // Convert coordinates to the format expected by the TSP solver
        let points: Vec<(f64, f64)> = model
            .coords
            .iter()
            .map(|p| {
                (
                    (p.x + OS_WINDOW_WIDTH as f32 / 2.0) as f64,
                    (p.y + OS_WINDOW_HEIGHT as f32 / 2.0) as f64,
                )
            })
            .collect();

        // Solve TSP
        let tour = travelling_salesman::simulated_annealing::solve(
            &points,
            time::Duration::milliseconds(MAX_TSP_SOLUTION_TIME_MILLISECONDS),
        );

        model.current_tour = tour.route;
        model.tour_length = tour.distance;
        model.state = ModelState::DrawingEdges;
        model.animations.edge_animation_progress = 0.0;
    }
}

fn update_drawing_edges(model: &mut Model) {
    model.animations.edge_animation_progress += EDGES_ANIMATION_SPEED;
    if model.animations.edge_animation_progress >= NUM_COORDS as f32 {
        model.animations.edge_animation_progress = NUM_COORDS as f32;
        model.animations.solution_view_progress = 0.0;
        model.state = ModelState::ViewingSolution;
    }
}

fn update_viewing_solution(model: &mut Model, update: Update) {
    model.animations.solution_view_progress += update.since_last.as_secs_f32();
    if model.animations.solution_view_progress >= SOLUTION_VIEW_TIME {
        // Generate new random target coordinates
        for i in 0..NUM_COORDS {
            model.target_coords[i] = random_point();
            model.animations.coord_animation_progress[i] = 0.0;
        }
        model.animations.edge_animation_progress = 0.0;
        model.state = ModelState::MovingCoords;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(LINEN);

    // Draw points
    for coord in &model.coords {
        draw.ellipse().xy(*coord).radius(5.0).color(BLACK);
    }
    // In the view function, replace the edge drawing code with this:
    if matches!(
        model.state,
        ModelState::DrawingEdges | ModelState::ViewingSolution
    ) {
        let progress = model.animations.edge_animation_progress;
        if progress > 0.0 {
            let num_edges = progress.floor() as usize;
            let partial_progress = progress.fract();

            // Draw complete edges
            for i in 0..num_edges.min(NUM_COORDS) {
                let start = model.coords[model.current_tour[i]];
                let end = model.coords[model.current_tour[(i + 1) % NUM_COORDS]];
                draw.line()
                    .start(start)
                    .end(end)
                    .weight(2.0)
                    .color(rgba(0.0, 0.0, 0.0, 0.5));
            }

            // Draw partial edge if in DrawingEdges state
            if matches!(model.state, ModelState::DrawingEdges) && partial_progress > 0.0 {
                let start = model.coords[model.current_tour[num_edges % NUM_COORDS]];
                let end = model.coords[model.current_tour[(num_edges + 1) % NUM_COORDS]];

                let actual_end = pt2(
                    lerp(start.x, end.x, partial_progress),
                    lerp(start.y, end.y, partial_progress),
                );

                draw.line()
                    .start(start)
                    .end(actual_end)
                    .weight(2.0)
                    .color(rgba(0.0, 0.0, 0.0, 0.5));
            }
        }
    }

    watermark(&draw);
    tour_length_watermark(model, &draw);

    draw.to_frame(app, &frame).unwrap();
}

fn watermark(draw: &Draw) {
    draw.text("1.25")
        .color(rgba(0.0, 0.0, 0.0, 0.5))
        .font_size(24)
        .align_text_bottom()
        .x_y(
            -(OS_WINDOW_WIDTH as f32) / 2.0 + 40.0,
            -(OS_WINDOW_HEIGHT as f32) / 2.0 + 110.0,
        );
}

fn tour_length_watermark(model: &Model, draw: &Draw) {
    if model.tour_length > 0.0 {
        draw.text(&format!("{:.1}", model.tour_length))
            .color(rgba(0.0, 0.0, 0.0, 0.5))
            .font_size(24)
            .align_text_bottom()
            .x_y(
                OS_WINDOW_WIDTH as f32 / 2.0 - 50.0,
                -(OS_WINDOW_HEIGHT as f32) / 2.0 + 110.0,
            );
    }
}

fn random_point() -> Point2 {
    let x = random_range(
        -(OS_WINDOW_WIDTH as f32) / 3.0,
        OS_WINDOW_WIDTH as f32 / 3.0,
    );
    let y = random_range(
        -(OS_WINDOW_HEIGHT as f32) / 3.0,
        OS_WINDOW_HEIGHT as f32 / 3.0,
    );
    pt2(x, y)
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}
