// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CStr;

use webvfx::{PluginInfo, WebVfxPlugin, frei0r_rs2};

struct SourcePluginInfo;

impl PluginInfo for SourcePluginInfo {
    const NAME: &'static CStr = c"WebVfx source";
    const EXPLANATION: &'static CStr = c"Renders HTML frames with no input videos";
    const FRAME_COUNT: usize = 0;
}

frei0r_rs2::plugin!(WebVfxPlugin<frei0r_rs2::KindSource, SourcePluginInfo>);
