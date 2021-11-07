use nokhwa::{query_devices, Camera, CameraFormat, CaptureAPIBackend, NokhwaError};

pub struct FrameGrabber {
    device: Camera,
    current_device_index: usize,
    current_format: Option<CameraFormat>,
}

impl FrameGrabber {
    pub fn new(index: usize, format: Option<CameraFormat>) -> Self {
        let device = Camera::new(index, format).unwrap();

        Self {
            device,
            current_device_index: index,
            current_format: format,
        }
    }

    pub fn list_devices() -> Result<Vec<(usize, String)>, NokhwaError> {
        query_devices(CaptureAPIBackend::Auto).and_then(|devices| {
            Ok(devices
                .into_iter()
                .map(|device| (device.index(), device.human_name()))
                .collect())
        })
    }

    pub fn change_device(&mut self, index: usize) {
        self.device.set_index(index);
    }

    // todo: make hashmap of formats as keys and resolution+framerates as values
    pub fn list_supported_modes(&mut self) -> Result<Vec<CameraFormat>, NokhwaError> {
        let fcc = self.device.compatible_fourcc()?;

        let modes = fcc
            .iter()
            .filter_map(|frame_format| {
                let resolutions = self
                    .device
                    .compatible_list_by_resolution(*frame_format)
                    .ok()?;

                let modes_for_format: Vec<CameraFormat> = resolutions
                    .into_iter()
                    .flat_map(|(resolution, frame_rates)| {
                        let modes_for_resolution: Vec<CameraFormat> = frame_rates
                            .into_iter()
                            .map(|frame_rate| {
                                CameraFormat::new(resolution, *frame_format, frame_rate)
                            })
                            .collect();

                        modes_for_resolution
                    })
                    .collect();

                Some(modes_for_format)
            })
            .flatten()
            .collect();

        Ok(modes)
    }

    pub fn change_mode(&self) {}

    pub fn open_stream(&mut self) {
        self.device.open_stream().unwrap();
    }

    pub fn grab_frame(&mut self) -> Vec<u8> {
        self.device.frame().unwrap().into_raw()
    }
}
