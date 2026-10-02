// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CStr;

use super::{PluginInfo, WebVfxPlugin};

pub type Mixer2Plugin = WebVfxPlugin<Mixer2Info, 2>;

pub struct Mixer2Info;

impl PluginInfo for Mixer2Info {
    const NAME: &'static CStr = c"WebVfx mixer2";
    const EXPLANATION: &'static CStr = c"Renders HTML frames with 2 input videos";
}
