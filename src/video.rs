use nokhwa::{Camera, CameraFormat, FrameFormat, Resolution};

pub struct VideoDevice {
    device: Camera,
}

impl VideoDevice {
    pub fn new() -> Self {
        let device = Camera::new(
            0,
            Some(CameraFormat::new_from(1280, 720, FrameFormat::MJPEG, 60)),
        )
        .unwrap();

        Self { device }
    }

    pub fn resolution(&self) -> (u32, u32) {
        let Resolution { width_x, height_y } = self.device.resolution();
        (width_x, height_y)
    }

    pub fn open(&mut self) {
        self.device.open_stream().unwrap();
    }

    pub fn grab_frame(&mut self) -> Vec<u8> {
        self.device.frame().unwrap().into_raw()
    }
}
