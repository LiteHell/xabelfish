// Copyright The pipewire-rs Contributors.
// SPDX-License-Identifier: MIT

//! This file is a rustic interpretation of the the [PipeWire Tutorial 5][tut]
//!
//! tut: https://docs.pipewire.org/page_tutorial5.html

use std::{
    sync::{mpsc, Arc, Once, RwLock},
    thread::{self, JoinHandle},
    time::SystemTime,
};

use image::{codecs::gif, Pixel, Rgb, RgbImage, Rgba, RgbaImage};
use pipewire::{self as pw, constants::ID_ANY, spa::param::video::VideoFormat};
use portal_screencast::{ActiveScreenCast, PortalError, SourceType};
use pw::{properties::properties, spa};

use spa::pod::Pod;

use crate::screen_capture::{
    pipewire::pipewire_thread::{self, PipewireTerminate, PipewireVideoData, PipewireVideoInfo},
    Capture,
};

pub fn select_window() -> Result<ActiveScreenCast, PortalError> {
    let screen_cast = portal_screencast::ScreenCast::new()?;
    let selected = screen_cast.start(None)?;
    Ok(selected)
}

// https://gstreamer.freedesktop.org/documentation/additional/design/mediatype-video-raw.html?gi-language=c
fn normalize_to_rgba(video_format: VideoFormat, data: &mut [u8]) -> (u8, u8, u8, u8) {
    match video_format {
        VideoFormat::RGB => (data[0], data[1], data[2], 255),
        VideoFormat::RGBA => (data[0], data[1], data[2], data[3]),
        VideoFormat::RGBx => (data[0], data[1], data[2], 255),
        VideoFormat::BGRx => (data[2], data[1], data[0], 255),
        VideoFormat::YUY2 => todo!(),
        VideoFormat::I420 => todo!(),
        _ => panic!(
            "Unsupported normalization to rgb from {}",
            video_format.as_raw()
        ),
    }
}

fn get_pixel_size(video_format: VideoFormat) -> u8 {
    match video_format {
        VideoFormat::RGB => 3,
        VideoFormat::YUY2 => 3,
        VideoFormat::I420 => 3,
        VideoFormat::RGBA => 4,
        VideoFormat::RGBx => 4,
        VideoFormat::BGRx => 4,
        _ => panic!(
            "Unsupported normalization to rgb from {}",
            video_format.as_raw()
        ),
    }
}

pub struct PipeWireScreenCapture {
    screen_info: Option<Arc<RwLock<PipewireVideoInfo>>>,
    screen_data: Option<Arc<RwLock<PipewireVideoData>>>,
    pw_sender: Option<Arc<RwLock<pipewire::channel::Sender<PipewireTerminate>>>>,
}

impl Drop for PipeWireScreenCapture {
    fn drop(&mut self) {
        if let Some(sender) = &self.pw_sender {
            sender.write().unwrap().send(PipewireTerminate {});
        }
    }
}

impl Capture for PipeWireScreenCapture {
    fn new() -> Self {
        Self {
            screen_data: None,
            screen_info: None,
            pw_sender: None,
        }
    }

    fn start_capture(&mut self) -> Result<(), ()> {
        let screen_info: Arc<RwLock<PipewireVideoInfo>> =
            Arc::new(RwLock::new(PipewireVideoInfo::empty()));
        let screen_data: Arc<RwLock<PipewireVideoData>> =
            Arc::new(RwLock::new(PipewireVideoData::empty()));
        {
            let data_cloned = screen_data.clone();
            let info_cloned = screen_info.clone();
            self.screen_data = Some(data_cloned);
            self.screen_info = Some(info_cloned);
        }
        let _handle = {
            let info_cloned = screen_info.clone();
            let data_cloned = screen_data.clone();
            let (pw_sender, pw_receiver) = pipewire::channel::channel();
            self.pw_sender = Some(Arc::new(RwLock::new(pw_sender.clone())));

            let screen_share = select_window().or_else(|_| Err(()))?;
            thread::spawn(move || {
                let pipewire_thread = {
                    thread::spawn(move || {
                        pipewire_thread::pipewire_thread(
                            screen_share,
                            info_cloned,
                            data_cloned,
                            pw_receiver,
                        );
                    })
                };

                let _ = pipewire_thread.join();
            })
        };

        Ok(())
    }

    fn get_captured_image(&mut self) -> Option<RgbaImage> {
        if self.screen_data.is_none() || self.screen_info.is_none() {
            None
        } else {
            let data = self.screen_data.as_ref().unwrap().read().unwrap().clone();
            let info = self.screen_info.as_ref().unwrap().read().unwrap().clone();

            if (data.is_empty() || info.is_empty()) {
                return None;
            }

            let mut buffer = data.data.clone();

            let height = info.height;
            let width = info.width;
            let pixel_size = get_pixel_size(info.format);
            let mut image = RgbaImage::new(width, height);

            for y in 0..height {
                let line_start = (data.offset as i32 + data.stride * y as i32) as u32;
                for x in 0..width {
                    let pixel_offset = (line_start + pixel_size as u32 * x) as usize;
                    let pixels =
                        normalize_to_rgba(info.format, &mut buffer.as_mut_slice()[pixel_offset..]);
                    let pixels_slice = [pixels.0, pixels.1, pixels.2, pixels.3];

                    image.put_pixel(x, y, Rgba::from(pixels_slice));
                }
            }

            println!(
                "Image data timestamp: {}",
                data.timestamp
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );
            Some(image)
        }
    }
}
