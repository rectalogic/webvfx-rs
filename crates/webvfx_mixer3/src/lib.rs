use std::ffi::CStr;

use webvfx::{PluginInfo, WebVfxPlugin, frei0r_rs2};

struct Mixer3PluginInfo;

impl PluginInfo for Mixer3PluginInfo {
    const NAME: &'static CStr = c"WebVfx mixer3";
    const EXPLANATION: &'static CStr = c"Renders HTML frames with 3 input videos";
    const FRAME_COUNT: usize = 3;
}

frei0r_rs2::plugin!(WebVfxPlugin<frei0r_rs2::KindMixer3, Mixer3PluginInfo>);
