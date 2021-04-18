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
    model.scope.feed(value);
}


fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let win = app.window_rect();
    let t = app.time;
    let draw = app.draw();

    // Clear the background to black.
    draw.background().color(BLACK);

    // Create an iterator yielding triangles for drawing a sine wave.
    let tris = model.scope.values.iter().zip(model.scope.values.iter().skip(1)).enumerate()
	.flat_map(|(i, (left, right))| {
	    let l_x = (i * 5) as f32 / 10.0;
	    let r_x = ((i + 1) * 5) as f32 / 10.0;
            let a = pt2(l_x, *left);
            let b = pt2(r_x, *right);
            let c = pt2(r_x, 0.0);
            let d = pt2(l_x, 0.0);
            geom::Quad([a, b, c, d]).triangles_iter()
	})
        .map(|tri| {
            // Color the vertices based on their amplitude.
            tri.map_vertices(|v| {
                let y_fract = map_range(v.y.abs(), 0.0, win.top(), 0.0, 1.0);
                let color = srgba(y_fract, 1.0 - y_fract, 1.0 - y_fract, 1.0);
                (v, color)
            })
        });

    // Draw the mesh!
    draw.mesh().tris_colored(tris);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn main() {
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .run();
}
