use std::ffi::CStr;

use webvfx::{PluginInfo, WebVfxPlugin, frei0r_rs2};

struct FilterPluginInfo;

impl PluginInfo for FilterPluginInfo {
    const NAME: &'static CStr = c"WebVfx filter";
    const EXPLANATION: &'static CStr = c"Renders HTML frames with 1 input video";
    const FRAME_COUNT: usize = 1;
}

frei0r_rs2::plugin!(WebVfxPlugin<frei0r_rs2::KindFilter, FilterPluginInfo>);
