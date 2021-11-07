#[macro_use]
extern crate glium;

use std::{sync::mpsc, thread};

use nokhwa::{CameraFormat, FrameFormat, Resolution};
use window::render_window;

use crate::frame_grabber::FrameGrabber;

mod frame_grabber;
mod window;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

fn main() {
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(1);

    thread::spawn(move || {
        let mut frame_grabber = FrameGrabber::new(
            0,
            Some(CameraFormat::new(
                Resolution::new(WIDTH, HEIGHT),
                FrameFormat::MJPEG,
                60,
            )),
        );

        loop {
            let frame = frame_grabber.grab_frame();
            let _ = tx.send(frame);
        }
    });

    render_window(WIDTH, HEIGHT, rx);
}
