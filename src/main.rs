#![feature(clamp)]
use nannou::prelude::*;

mod serial;
mod debugobjects;

use serial::SerialConnector;
use debugobjects::{Scope, DebugProcessor};

const HZ:f32 = 1.0;
const AMP:f32 = 100.0;

const BAUD:u32 = 230_400;
const PORT:&str = "/dev/serial/by-id/usb-FTDI_FT232R_USB_UART_00000000-if00-port0";

struct Model {
    scope: Scope,
    serial: SerialConnector,
}

fn model(_app: &App) -> Model {
    let scope = Scope::new(&vec!["Test".to_string()]);
    let serial = SerialConnector::new(PORT, BAUD).expect("serial port failed");
    assert!(scope.name() == "Test");
    Model { scope, serial }
}

fn update(app: &App, model: &mut Model, _update: Update)
{
    let t = app.time;
    let value = (t * HZ * TAU).sin() * AMP;
    for line in model.serial.receiver.try_iter() {
	print!("{}", line);
    }
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
