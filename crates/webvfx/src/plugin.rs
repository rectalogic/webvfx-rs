// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    path::{self, Path},
};

use crate::renderer::processor::RenderProcessor;

pub mod filter;
pub mod mixer2;
pub mod mixer3;
pub mod source;

pub struct WebVfxPlugin<K: frei0r_rs2::PluginKind, const S: usize> {
    html_path: CString,
    width: u32,
    height: u32,
    processor: Option<anyhow::Result<RenderProcessor<S>>>,
    _phantom: PhantomData<K>,
}

impl<K, const S: usize> WebVfxPlugin<K, S>
where
    K: frei0r_rs2::PluginKind,
{
    fn new(width: u32, height: u32) -> Self {
        Self {
            html_path: c"".to_owned(),
            width,
            height,
            processor: None,
            _phantom: PhantomData,
        }
    }

    fn update(&mut self, time: f64, inframes: [&[u32]; S], outframe: &mut [u32]) {
        if self.processor.is_none() {
            match self.html_path.to_str() {
                Ok(html_path) => match path::absolute(Path::new(html_path)) {
                    Ok(absolute_path) => {
                        let processor =
                            RenderProcessor::<S>::new(absolute_path, self.width, self.height);
                        if let Err(ref e) = processor {
                            eprintln!("WebVfx: failed to create renderer: {e:?}");
                        }
                        self.processor = Some(processor);
                    }
                    Err(e) => {
                        eprintln!("WebVfx: invalid absolute path '{html_path}': {e:?}");
                        self.processor = Some(Err(e.into()));
                        return;
                    }
                },
                Err(e) => {
                    eprintln!("WebVfx: invalid path `{:?}`", self.html_path);
                    self.processor = Some(Err(e.into()));
                    return;
                }
            }
        }
        let processor = match self.processor {
            Some(Ok(ref processor)) => processor,
            Some(Err(_)) => return,
            None => unreachable!(),
        };
        if let Err(e) = processor.update(time, inframes, outframe) {
            eprintln!("WebVfx: failed to render frame: {e:?}");
        }
    }
}

trait PluginInfo {
    const NAME: &'static CStr;
    const EXPLANATION: &'static CStr;
}

impl<K, const S: usize> frei0r_rs2::Plugin for WebVfxPlugin<K, S>
where
    K: frei0r_rs2::PluginKind + PluginInfo + Send + 'static,
{
    type Kind = K;

    const PARAMS: &'static [frei0r_rs2::ParamInfo<Self>] = &[frei0r_rs2::ParamInfo::new_string(
        c"html_path",
        c"Web page file path",
        |plugin| plugin.html_path.as_c_str(),
        |plugin, value| value.clone_into(&mut plugin.html_path),
    )];

    fn info() -> frei0r_rs2::PluginInfo {
        frei0r_rs2::PluginInfo {
            name: K::NAME,
            author: c"Andrew Wason",
            color_model: frei0r_rs2::ColorModel::RGBA8888,
            major_version: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            minor_version: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            explanation: Some(K::EXPLANATION),
        }
    }

    fn new(width: usize, height: usize) -> Self {
        WebVfxPlugin::new(width as u32, height as u32)
    }
}
