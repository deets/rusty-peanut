#![feature(clamp)]
use nannou::prelude::*;

mod serial;
mod debugobjects;

use serial::SerialConnector;
use debugobjects::{DebugObjects};

const BAUD:u32 = 230_400;
const PORT:&str = "/dev/serial/by-id/usb-FTDI_FT232R_USB_UART_00000000-if00-port0";

struct Model {
    views: DebugObjects,
    serial: SerialConnector,
}

fn model(_app: &App) -> Model {
    let views = DebugObjects::new();
    let serial = SerialConnector::new(PORT, BAUD).expect("serial port failed");
    Model { views , serial }
}

fn update(_app: &App, model: &mut Model, _update: Update)
{
    for line in model.serial.receiver.try_iter() {
	//println!("{}", line);
	model.views.feed(&line);
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(BLACK);
    model.views.draw(&draw);
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
