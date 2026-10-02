// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CStr;

use super::{PluginInfo, WebVfxPlugin};

pub type Mixer3Plugin = WebVfxPlugin<Mixer3Info, 3>;

pub struct Mixer3Info;

impl PluginInfo for Mixer3Info {
    const NAME: &'static CStr = c"WebVfx mixer3";
    const EXPLANATION: &'static CStr = c"Renders HTML frames with 3 input videos";
}
