use std::ffi::CStr;

use webvfx::{PluginInfo, WebVfxPlugin, frei0r_rs2};

struct Mixer2PluginInfo;

impl PluginInfo for Mixer2PluginInfo {
    const NAME: &'static CStr = c"WebVfx mixer2";
    const EXPLANATION: &'static CStr = c"Renders HTML frames with 2 input videos";
    const FRAME_COUNT: usize = 2;
}

frei0r_rs2::plugin!(WebVfxPlugin<frei0r_rs2::KindMixer2, Mixer2PluginInfo>);
