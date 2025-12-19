// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use anyrender::{ImageRenderer, PaintScene};
use blitz_dom::{
    DocumentConfig,
    node::{ImageData, RasterImageData, SpecialElementData},
};
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::{
    net::Url,
    shell::{ColorScheme, Viewport},
};
use linebender_resource_handle::Blob;
use smallvec::SmallVec;

pub mod net;
pub mod processor;

cfg_if::cfg_if! {
    if #[cfg(feature = "anyrender_vello")] {
        type AnyRender = anyrender_vello::VelloImageRenderer;
    } else if #[cfg(feature = "anyrender_vello_cpu")] {
        type AnyRender = anyrender_vello_cpu::VelloCpuImageRenderer;
    }else if #[cfg(feature = "anyrender_skia")] {
        type AnyRender = anyrender_skia::SkiaImageRenderer;
    }
}

// Node ID mapped to a pair of video frame buffers
type VideoNode = (SmallVec<[usize; 32]>, [Arc<Vec<u8>>; 2]);

pub const WEBVFX_SELECTOR_PREFIX: &str = "img.webvfx-video";
pub const WEBVFX_CSS_ANIMATION_PROPERTY: &str = "--webvfx-animation-duration";

struct WebVfxRenderer<const S: usize> {
    width: u32,
    height: u32,
    document: HtmlDocument,
    renderer: AnyRender,
    video_nodes: [Option<VideoNode>; S],
    video_node_index: usize,
}

impl<const S: usize> WebVfxRenderer<S> {
    fn new(base_url: &Url, html: &str, animation_duration: &str, width: u32, height: u32) -> Self {
        let css_properties = format!(
            r"
            :root {{
                {WEBVFX_CSS_ANIMATION_PROPERTY}: {animation_duration}
            }}
        "
        );
        let mut document = HtmlDocument::from_html(
            html,
            DocumentConfig {
                base_url: Some(base_url.as_str().into()),
                ua_stylesheets: Some(vec![css_properties]),
                net_provider: Some(Arc::new(net::FileProvider)),
                viewport: Some(Viewport::new(width, height, 1.0, ColorScheme::Light)),
                ..Default::default()
            },
        );
        let video_nodes: [Option<VideoNode>; S] = (0..S)
            .map(|i| {
                if let Ok(node_ids) =
                    document.query_selector_all(&format!("{}{}", WEBVFX_SELECTOR_PREFIX, i + 1))
                    && !node_ids.is_empty()
                {
                    let frame = vec![0u8; (width * height * 4) as usize];
                    let frame_arc = Arc::new(frame.clone());
                    node_ids.iter().copied().for_each(|node_id| {
                        if let Some(node) = document.get_node_mut(node_id)
                            && let Some(element_data) = node.element_data_mut()
                        {
                            element_data.special_data =
                                SpecialElementData::Image(Box::new(ImageData::Raster(
                                    RasterImageData::new(width, height, frame_arc.clone()),
                                )));
                        }
                    });
                    Some((node_ids, [frame_arc, Arc::new(frame)]))
                } else {
                    None
                }
            })
            .collect::<Vec<Option<VideoNode>>>()
            .try_into()
            .unwrap();

        let renderer = AnyRender::new(width, height);
        Self {
            width,
            height,
            document,
            renderer,
            video_nodes,
            video_node_index: 0, // We populated special_data with the 0th image buffer
        }
    }

    fn update(&mut self, time: f64, inframes: [&[u8]; S], outframe: &mut [u8]) {
        self.video_node_index = (self.video_node_index + 1) % 2;
        self.video_nodes
            .iter_mut()
            .zip(inframes)
            .filter_map(|(video_node, inframe)| {
                video_node.as_mut().map(|video_node| (video_node, inframe))
            })
            .for_each(|((video_node_ids, frames), inframe)| {
                Arc::get_mut(&mut frames[self.video_node_index])
                    .unwrap()
                    .copy_from_slice(inframe);
                video_node_ids.iter().copied().for_each(|node_id| {
                    // Safe to unwrap since we verified all this when contructing
                    let raster_data = self
                        .document
                        .get_node_mut(node_id)
                        .unwrap()
                        .element_data_mut()
                        .unwrap()
                        .raster_image_data_mut()
                        .unwrap();
                    raster_data.data = Blob::new(frames[self.video_node_index].clone());
                });
            });
        self.document.resolve(time);
        self.renderer.render(
            |scene| {
                scene.reset();
                paint_scene(scene, &self.document, 1.0, self.width, self.height);
            },
            outframe,
        );
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use test_support::{HEIGHT, RgbaImage, WIDTH, assert_reference, read_image, testdata};

    fn init_renderer<const S: usize>(html_file: &str) -> (WebVfxRenderer<S>, RgbaImage) {
        let html_path = testdata!().join(html_file);
        let html = std::fs::read_to_string(&html_path).unwrap();
        let url = Url::from_file_path(html_path.as_path()).unwrap();
        let renderer = WebVfxRenderer::<S>::new(&url, &html, "5s", WIDTH, HEIGHT);
        let output = RgbaImage::new(WIDTH, HEIGHT);
        (renderer, output)
    }

    fn render<const S: usize>(
        time: f64,
        renderer: &mut WebVfxRenderer<S>,
        inframe_paths: [&Path; S],
        output: &mut RgbaImage,
        reference_path: &Path,
    ) {
        let inframes = inframe_paths.map(read_image);
        let inframe_refs: [&[u8]; S] = inframes
            .iter()
            .map(std::vec::Vec::as_slice)
            .collect::<Vec<&[u8]>>()
            .try_into()
            .unwrap();
        renderer.update(
            time,
            inframe_refs,
            output.as_flat_samples_mut().image_mut_slice().unwrap(),
        );
        assert_reference(reference_path, output);
    }

    #[test]
    fn test_source() {
        let (mut r, mut output) = init_renderer::<0>("source.html");
        render(
            0.0,
            &mut r,
            [],
            &mut output,
            &testdata!().join("source-1.png"),
        );
    }

    #[test]
    fn test_filter() {
        let (mut r, mut output) = init_renderer::<1>("filter.html");
        render(
            0.0,
            &mut r,
            [&testdata!().join("a-320x240.png")],
            &mut output,
            &testdata!().join("filter-1.png"),
        );
        render(
            1.0,
            &mut r,
            [&testdata!().join("b-320x240.png")],
            &mut output,
            &testdata!().join("filter-2.png"),
        );
        render(
            2.0,
            &mut r,
            [&testdata!().join("a-320x240.png")],
            &mut output,
            &testdata!().join("filter-3.png"),
        );
    }

    #[test]
    fn test_mixer2() {
        let (mut r, mut output) = init_renderer::<2>("mixer2.html");
        render(
            0.0,
            &mut r,
            [
                &testdata!().join("a-320x240.png"),
                &testdata!().join("b-320x240.png"),
            ],
            &mut output,
            &testdata!().join("mixer2-1.png"),
        );
        render(
            1.0,
            &mut r,
            [
                &testdata!().join("b-320x240.png"),
                &testdata!().join("a-320x240.png"),
            ],
            &mut output,
            &testdata!().join("mixer2-2.png"),
        );
        render(
            2.0,
            &mut r,
            [
                &testdata!().join("a-320x240.png"),
                &testdata!().join("b-320x240.png"),
            ],
            &mut output,
            &testdata!().join("mixer2-3.png"),
        );
    }

    fn test_mixer3_base(html_file: &str, reference_paths: [&str; 3]) {
        let (mut r, mut output) = init_renderer::<3>(html_file);
        render(
            0.0,
            &mut r,
            [
                &testdata!().join("a-320x240.png"),
                &testdata!().join("b-320x240.png"),
                &testdata!().join("c-320x240.png"),
            ],
            &mut output,
            &testdata!().join(reference_paths[0]),
        );
        render(
            1.0,
            &mut r,
            [
                &testdata!().join("c-320x240.png"),
                &testdata!().join("a-320x240.png"),
                &testdata!().join("b-320x240.png"),
            ],
            &mut output,
            &testdata!().join(reference_paths[1]),
        );
        render(
            3.0,
            &mut r,
            [
                &testdata!().join("b-320x240.png"),
                &testdata!().join("c-320x240.png"),
                &testdata!().join("a-320x240.png"),
            ],
            &mut output,
            &testdata!().join(reference_paths[2]),
        );
    }

    #[test]
    fn test_mixer3() {
        test_mixer3_base(
            "mixer3.html",
            ["mixer3-1.png", "mixer3-2.png", "mixer3-3.png"],
        );
    }

    #[test]
    fn test_mixer3_dupe() {
        test_mixer3_base(
            "mixer3-dupe.html",
            [
                "mixer3-dupe-1.png",
                "mixer3-dupe-2.png",
                "mixer3-dupe-3.png",
            ],
        );
    }
}
