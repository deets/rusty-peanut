#![feature(clamp)]
use nannou::prelude::*;

mod serial;
mod debugobjects;

use debugobjects::{Scope, DebugProcessor};

const HZ:f32 = 1.0;
const AMP:f32 = 100.0;

struct Model {
    scope: Scope
}

fn model(_app: &App) -> Model {
    let scope = Scope::new(&vec!["Test".to_string()]);
    assert!(scope.name() == "Test");
    Model { scope }
}

fn update(app: &App, model: &mut Model, _update: Update)
{
    let t = app.time;
    let value = (t * HZ * TAU).sin() * AMP;
    model.scope.feed(vec![value]);
}


fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(BLACK);
    model.scope.draw(&draw);
    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn main() {
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .run();
}
