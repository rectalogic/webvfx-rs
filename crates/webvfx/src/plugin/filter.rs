// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CStr;

use super::{PluginInfo, WebVfxPlugin};

pub type FilterPlugin = WebVfxPlugin<FilterInfo, 1>;

pub struct FilterInfo;

impl PluginInfo for FilterInfo {
    const NAME: &'static CStr = c"WebVfx filter";
    const EXPLANATION: &'static CStr = c"Renders HTML frames with 1 input video";
}
