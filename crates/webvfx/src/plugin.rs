// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
};

pub mod filter;
pub mod mixer2;
pub mod mixer3;
pub mod source;

pub struct WebVfxPlugin<K: frei0r_rs2::PluginKind, const S: usize> {
    url: CString,
    width: u32,
    height: u32,
    _phantom: PhantomData<K>,
}

impl<K, const S: usize> WebVfxPlugin<K, S>
where
    K: frei0r_rs2::PluginKind,
{
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            url: c"".to_owned(),
            _phantom: PhantomData,
        }
    }

    fn update(&mut self, time: f64, inframes: [&[u32]; S], outframe: &mut [u32]) {
        //XXX do we need FRAME_COUNT? can just hardcode HTML ids - webvfx-video1, webvfx-video2 etc.
        // // XXX may need it to parameterize the Job we send over - e.g. [(*u8, len); FRAME_COUNT]
    }
}

trait PluginInfo {
    const NAME: &'static CStr;
    const EXPLANATION: &'static CStr;
}

impl<K, const S: usize> frei0r_rs2::Plugin for WebVfxPlugin<K, S>
where
    K: frei0r_rs2::PluginKind + PluginInfo + Send + 'static,
{
    type Kind = K;

    const PARAMS: &'static [frei0r_rs2::ParamInfo<Self>] = &[frei0r_rs2::ParamInfo::new_string(
        c"url",
        c"Web page URL",
        |plugin| plugin.url.as_c_str(),
        |plugin, value| plugin.url = value.to_owned(),
    )];

    fn info() -> frei0r_rs2::PluginInfo {
        frei0r_rs2::PluginInfo {
            name: K::NAME,
            author: c"Andrew Wason",
            color_model: frei0r_rs2::ColorModel::RGBA8888,
            major_version: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            minor_version: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            explanation: Some(K::EXPLANATION),
        }
    }

    fn new(width: usize, height: usize) -> Self {
        WebVfxPlugin::new(width as u32, height as u32)
    }
}
