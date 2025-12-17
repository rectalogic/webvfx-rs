// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use anyrender::{ImageRenderer, PaintScene};
use anyrender_vello::VelloImageRenderer;
use blitz::{
    dom::{
        DocumentConfig, local_name,
        node::{ImageData, RasterImageData, SpecialElementData},
    },
    html::HtmlDocument,
    paint::paint_scene,
    traits::{
        net::Url,
        shell::{ColorScheme, Viewport},
    },
};

pub mod processor;

// Node ID mapped to a pair of video frame buffers
type VideoNode = (usize, [Arc<Vec<u8>>; 2]);

struct WebVfxRenderer<const S: usize> {
    width: u32,
    height: u32,
    document: HtmlDocument,
    renderer: VelloImageRenderer,
    video_nodes: [Option<VideoNode>; S],
    video_node_index: usize,
}

impl<const S: usize> WebVfxRenderer<S> {
    fn new(base_url: &Url, html: &str, width: u32, height: u32) -> Self {
        let mut document = HtmlDocument::from_html(
            html,
            DocumentConfig {
                //XXX  also need NetProvider
                base_url: Some(base_url.as_str().into()),
                viewport: Some(Viewport::new(width, height, 1.0, ColorScheme::Light)),
                ..Default::default()
            },
        );
        let video_nodes: [Option<VideoNode>; S] = (0..S)
            .map(|i| {
                if let Ok(Some(node_id)) =
                    document.query_selector(&format!("#webvfx-video{}", i + 1))
                    && let Some(node) = document.get_node_mut(node_id)
                    && let Some(element_data) = node.element_data_mut()
                    && element_data.name.local == local_name!("img")
                {
                    let frame = vec![0u8; (width * height * 4) as usize];
                    let frame_arc = Arc::new(frame.clone());
                    element_data.special_data = SpecialElementData::Image(Box::new(
                        ImageData::Raster(RasterImageData::new(width, height, frame_arc.clone())),
                    ));
                    Some((node_id, [frame_arc, Arc::new(frame)]))
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
            video_node_index: 0, // We populated special_data with the 0th image buffer
        }
    }

    fn update(&mut self, time: f64, inframes: [&[u8]; S], outframe: &mut [u8]) {
        self.video_node_index = (self.video_node_index + 1) % 2;
        self.video_nodes
            .iter_mut()
            .zip(inframes)
            .flat_map(|(video_node, inframe)| {
                video_node.as_mut().map(|video_node| (video_node, inframe))
            })
            .for_each(|((video_node_id, frames), inframe)| {
                Arc::get_mut(&mut frames[self.video_node_index])
                    .unwrap()
                    .copy_from_slice(inframe);
                // Safe to unwrap since we verified all this when contructing
                let raster_data = self
                    .document
                    .get_node_mut(*video_node_id)
                    .unwrap()
                    .element_data_mut()
                    .unwrap()
                    .raster_image_data_mut()
                    .unwrap();
                raster_data.data = frames[self.video_node_index].clone();
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
    use image::RgbaImage;
    use test_support::{HEIGHT, WIDTH, assert_reference, read_image, testdata};

    fn init_renderer<const S: usize>(html_file: &str) -> (WebVfxRenderer<S>, RgbaImage) {
        let html_path = testdata!().join(html_file);
        let html = std::fs::read_to_string(&html_path).unwrap();
        let url = Url::from_file_path(html_path.as_path()).unwrap();
        let renderer = WebVfxRenderer::<S>::new(&url, &html, WIDTH, HEIGHT);
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
            .map(|f| f.as_slice())
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

    #[test]
    fn test_mixer3() {
        let (mut r, mut output) = init_renderer::<3>("mixer3.html");
        render(
            0.0,
            &mut r,
            [
                &testdata!().join("a-320x240.png"),
                &testdata!().join("b-320x240.png"),
                &testdata!().join("c-320x240.png"),
            ],
            &mut output,
            &testdata!().join("mixer3-1.png"),
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
            &testdata!().join("mixer3-2.png"),
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
            &testdata!().join("mixer3-3.png"),
        );
    }
}
