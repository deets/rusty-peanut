#![feature(clamp)]
use nannou::prelude::*;

use log::{debug};

mod serial;
mod debugobjects;

use serial::SerialConnector;
use debugobjects::{Scope, DebugProcessor, DebugLine};

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

fn update(_app: &App, model: &mut Model, _update: Update)
{
    for line in model.serial.receiver.try_iter() {
	match DebugLine::from_str(&line) {
	    Ok(debug_line) => {
		debug!("feeding Scope tokens {:?}", debug_line.tokens);
		model.scope.feed(debug_line.tokens);
	    }
	    _ => {}
	}
    }
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
    //env_logger::init();
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .run();
}
