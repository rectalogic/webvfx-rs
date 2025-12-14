// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use anyrender::ImageRenderer;
use anyrender_vello::VelloImageRenderer;
use blitz::{
    dom::DocumentConfig,
    html::HtmlDocument,
    traits::shell::{ColorScheme, Viewport},
};

// Node ID mapped to a pair of video frame buffers
type VideoNode = (usize, (Arc<Vec<u8>>, Arc<Vec<u8>>));

pub struct WebVfxRender<const S: usize> {
    width: u32,
    height: u32,
    document: HtmlDocument,
    renderer: VelloImageRenderer,
    video_nodes: [Option<VideoNode>; S],
}

impl<const S: usize> WebVfxRender<S> {
    fn new(url: &str, width: u32, height: u32) -> Self {
        let document = HtmlDocument::from_html(
            "", //XXX need to load url - also need NetProvider
            DocumentConfig {
                base_url: Some(url.into()),
                viewport: Some(Viewport::new(width, height, 1.0, ColorScheme::Light)),
                ..Default::default()
            },
        );
        let video_nodes: [Option<VideoNode>; S] = (0..S)
            .map(|i| {
                if let Ok(Some(node_id)) = document.query_selector(&format!("#webvfx-video{i}"))
                    && let Some(node) = document.get_node(node_id)
                    && node
                        .element_data()
                        .and_then(|e| e.raster_image_data())
                        .is_some()
                {
                    let frame = vec![0u8; (width * height * 4) as usize];
                    Some((node_id, (Arc::new(frame.clone()), Arc::new(frame))))
                } else {
                    None
                }
            })
            .collect::<Vec<Option<VideoNode>>>()
            .try_into()
            .unwrap();

        let renderer = VelloImageRenderer::new(width, height);
        Self {
            width,
            height,
            document,
            renderer,
            video_nodes,
        }
    }

    fn update(&mut self, time: f64, inframes: [&[u32]; S], outframe: &mut [u32]) {}
}
