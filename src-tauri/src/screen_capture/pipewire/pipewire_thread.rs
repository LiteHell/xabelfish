// Copyright The pipewire-rs Contributors.
// SPDX-License-Identifier: MIT

//! This file is a rustic interpretation of the the [PipeWire Tutorial 5][tut]
//!
//! tut: https://docs.pipewire.org/page_tutorial5.html

use std::{
    sync::{Arc, RwLock},
    time::{self, SystemTime},
};

use image::{Pixel, RgbImage, RgbaImage};
use pipewire::{self as pw, constants::ID_ANY, spa::param::video::VideoFormat};
use portal_screencast::{ActiveScreenCast, PortalError, SourceType};
use pw::{properties::properties, spa};

use spa::pod::Pod;

use crate::screen_capture::Capture;

// Simple struct to tell pipewire to terminate
pub struct PipewireTerminate {}

struct UserData {
    format: spa::param::video::VideoInfoRaw,
}

#[derive(Debug, Clone, Copy)]
pub struct PipewireVideoInfo {
    pub format: VideoFormat,
    pub width: u32,
    pub height: u32,
    empty: bool,
}

#[derive(Debug, Clone)]
pub struct PipewireVideoData {
    pub stride: i32,
    pub offset: u32,
    pub size: u32,
    pub data: Vec<u8>,
    pub timestamp: SystemTime,
    empty: bool,
}

impl PipewireVideoData {
    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn empty() -> Self {
        PipewireVideoData {
            stride: 0,
            offset: 0,
            size: 0,
            data: vec![],
            timestamp: SystemTime::now(),
            empty: true,
        }
    }
}

impl PipewireVideoInfo {
    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn empty() -> Self {
        PipewireVideoInfo {
            format: VideoFormat::RGBA,
            width: 0,
            height: 0,
            empty: true,
        }
    }
}

pub fn pipewire_thread(
    screen: ActiveScreenCast,
    video_info_lock: Arc<RwLock<PipewireVideoInfo>>,
    video_data_lock: Arc<RwLock<PipewireVideoData>>,
    pw_receiver: pipewire::channel::Receiver<PipewireTerminate>,
) {
    pw::init();
    let mainloop =
        pw::main_loop::MainLoop::new(None).expect("Failed to initialize pipewire main loop");
    let context = pw::context::Context::new(&mainloop).expect("Failed to initialize context");
    let core = context
        .connect(None)
        .expect("Failed to connect pipewire core");

    let data = UserData {
        format: Default::default(),
    };

    let _receiver = pw_receiver.attach(mainloop.loop_(), {
        let mainloop = mainloop.clone();
        move |_| mainloop.quit()
    });

    let stream = pw::stream::Stream::new(
        &core,
        "kabelfish-screen-capture",
        properties! {
            *pw::keys::MEDIA_TYPE => "Video",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Screen",
        },
    )
    .expect("Failed to create pipewire stream");

    let _listener = stream
        .add_local_listener_with_user_data(data)
        .state_changed(|_, _, old, new| {
            println!("State changed: {:?} -> {:?}", old, new);
        })
        .param_changed(move |_, user_data, id, param| {
            let Some(param) = param else {
                return;
            };
            if id != pw::spa::param::ParamType::Format.as_raw() {
                return;
            }

            let (media_type, media_subtype) =
                match pw::spa::param::format_utils::parse_format(param) {
                    Ok(v) => v,
                    Err(_) => return,
                };

            if media_type != pw::spa::param::format::MediaType::Video
                || media_subtype != pw::spa::param::format::MediaSubtype::Raw
            {
                return;
            }

            user_data
                .format
                .parse(param)
                .expect("Failed to parse param changed to VideoInfoRaw");

            println!("format: {:#?}", user_data.format.format());
            (*(video_info_lock.write().unwrap())) = PipewireVideoInfo {
                format: user_data.format.format(),
                width: user_data.format.size().width,
                height: user_data.format.size().height,
                empty: false,
            };
        })
        .process(move |stream, _| {
            match stream.dequeue_buffer() {
                None => println!("out of buffers"),
                Some(mut buffer) => {
                    let datas = buffer.datas_mut();
                    if datas.is_empty() {
                        return;
                    }

                    // copy frame data to screen
                    let data = &mut datas[0];
                    let stride = data.chunk().stride();

                    let timestamp = time::SystemTime::now();
                    *(video_data_lock.write().unwrap()) = PipewireVideoData {
                        stride: stride,
                        offset: data.chunk().offset(),
                        size: data.chunk().size(),
                        data: data.data().unwrap().to_vec(),
                        timestamp: timestamp,
                        empty: false,
                    };
                }
            }
        })
        .register()
        .expect("Failed to register listeners");

    println!("Created stream {:#?}", stream);

    let obj = pw::spa::pod::object!(
        pw::spa::utils::SpaTypes::ObjectParamFormat,
        pw::spa::param::ParamType::EnumFormat,
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::MediaType,
            Id,
            pw::spa::param::format::MediaType::Video
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::MediaSubtype,
            Id,
            pw::spa::param::format::MediaSubtype::Raw
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::VideoFormat,
            Choice,
            Enum,
            Id,
            pw::spa::param::video::VideoFormat::RGB,
            pw::spa::param::video::VideoFormat::RGBA,
            pw::spa::param::video::VideoFormat::RGBx,
            pw::spa::param::video::VideoFormat::BGRx,
            pw::spa::param::video::VideoFormat::YUY2,
            pw::spa::param::video::VideoFormat::I420,
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::VideoFramerate,
            Choice,
            Range,
            Fraction,
            pw::spa::utils::Fraction { num: 25, denom: 1 },
            pw::spa::utils::Fraction { num: 0, denom: 1 },
            pw::spa::utils::Fraction {
                num: 1000,
                denom: 1
            }
        ),
    );

    let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pw::spa::pod::Value::Object(obj),
    )
    .unwrap()
    .0
    .into_inner();

    let mut params = [Pod::from_bytes(&values).unwrap()];

    stream
        .connect(
            spa::utils::Direction::Input,
            Some(screen.streams().last().unwrap().pipewire_node()),
            pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
            &mut params,
        )
        .expect("Failed to connect stream");

    mainloop.run();
}
