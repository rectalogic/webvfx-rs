// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
};

pub struct WebVfxPlugin<K: frei0r_rs2::PluginKind, I: PluginInfo> {
    url: CString,
    width: u32,
    height: u32,
    _phantom_k: PhantomData<K>,
    _phantom_i: PhantomData<I>,
}

impl<K, I> WebVfxPlugin<K, I>
where
    K: frei0r_rs2::PluginKind,
    I: PluginInfo,
{
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            url: c"".to_owned(),
            _phantom_k: PhantomData,
            _phantom_i: PhantomData,
        }
    }

    pub fn update(&mut self, time: f64, inframes: &[&[u32]], outframe: &mut [u32]) {}
}

pub trait PluginInfo {
    const NAME: &'static CStr;
    const EXPLANATION: &'static CStr;
    const FRAME_COUNT: usize;
}

impl<K, I> frei0r_rs2::Plugin for WebVfxPlugin<K, I>
where
    K: frei0r_rs2::PluginKind + Send + 'static,
    I: PluginInfo + Send + 'static,
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
            name: I::NAME,
            author: c"Andrew Wason",
            color_model: frei0r_rs2::ColorModel::RGBA8888,
            major_version: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            minor_version: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            explanation: Some(I::EXPLANATION),
        }
    }

    fn new(width: usize, height: usize) -> Self {
        WebVfxPlugin::new(width as u32, height as u32)
    }
}

impl<I> frei0r_rs2::SourcePlugin for WebVfxPlugin<frei0r_rs2::KindSource, I>
where
    I: PluginInfo + Send + 'static,
{
    fn update_source(&mut self, time: f64, outframe: &mut [u32]) {
        self.update(time, &[], outframe);
    }
}

impl<I> frei0r_rs2::FilterPlugin for WebVfxPlugin<frei0r_rs2::KindFilter, I>
where
    I: PluginInfo + Send + 'static,
{
    fn update_filter(&mut self, time: f64, inframe: &[u32], outframe: &mut [u32]) {
        self.update(time, &[inframe], outframe);
    }
}

impl<I> frei0r_rs2::Mixer2Plugin for WebVfxPlugin<frei0r_rs2::KindMixer2, I>
where
    I: PluginInfo + Send + 'static,
{
    fn update_mixer2(
        &mut self,
        time: f64,
        inframe1: &[u32],
        inframe2: &[u32],
        outframe: &mut [u32],
    ) {
        self.update(time, &[inframe1, inframe2], outframe);
    }
}

impl<I> frei0r_rs2::Mixer3Plugin for WebVfxPlugin<frei0r_rs2::KindMixer3, I>
where
    I: PluginInfo + Send + 'static,
{
    fn update_mixer3(
        &mut self,
        time: f64,
        inframe1: &[u32],
        inframe2: &[u32],
        inframe3: &[u32],
        outframe: &mut [u32],
    ) {
        self.update(time, &[inframe1, inframe2, inframe3], outframe);
    }
}
