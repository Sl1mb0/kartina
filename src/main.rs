use nannou::prelude::*;
use nix::unistd::{fork, ForkResult};

fn main() {
    match fork() {
        Ok(ForkResult::Parent { child, .. }) => { 
            play::play("./song/Can I Take A Picture With You.mp3").unwrap();
        }
        Ok(ForkResult::Child) => {
            nannou::app(model)
                .update(update)
                .simple_window(view)
                .run();
        },
        Err(_) => println!("Fork failed."),
    }
}

struct Model {}

fn model(_app: &App) -> Model {
    Model {} 
}

// Handle events related to the window and update the model if necessary
fn update(_app: &App, _model: &mut Model, _update: Update) {
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, _model: &Model, frame: Frame) {
    // prepare to draw
    let draw = app.draw();

    // Generate sine wave stuff based on time of app
    let sine = app.time.sin();
    let fastersine = (app.time * 2.0).sin();

    // Get boundary of window to constrain movements of circle
    let boundary = app.window_rect();

    // Map sine wave functions to ranges between window boundaries
    let x = map_range(sine, -1.0, 1.0, boundary.left(), boundary.right());
    let y = map_range(fastersine, -1.0, 1.0, boundary.bottom(), boundary.top());

    // color background
    frame.clear(BLACK);
    
    // draw a green ellipse
    draw.ellipse().color(GREEN).x_y(x, y);

    draw.to_frame(app, &frame).unwrap();
    // shapes::drawCube(point, WHITE, frame); <- this is how I want the code to look
    // shapes::drawTriangle(point, GREEN, frame);
}
