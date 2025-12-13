// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CStr;

use super::{PluginInfo, WebVfxPlugin};

pub type SourcePlugin = WebVfxPlugin<frei0r_rs2::KindSource, 0>;

impl PluginInfo for frei0r_rs2::KindSource {
    const NAME: &'static CStr = c"WebVfx mixer3";
    const EXPLANATION: &'static CStr = c"Renders HTML frames with 3 input videos";
}

impl frei0r_rs2::SourcePlugin for SourcePlugin {
    fn update_source(&mut self, time: f64, outframe: &mut [u32]) {
        self.update(time, [], outframe);
    }
}
