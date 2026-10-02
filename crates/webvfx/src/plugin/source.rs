// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CStr;

use super::{PluginInfo, WebVfxPlugin};

pub type SourcePlugin = WebVfxPlugin<SourceInfo, 0>;

pub struct SourceInfo;

impl PluginInfo for SourceInfo {
    const NAME: &'static CStr = c"WebVfx mixer3";
    const EXPLANATION: &'static CStr = c"Renders HTML frames with 3 input videos";
}
