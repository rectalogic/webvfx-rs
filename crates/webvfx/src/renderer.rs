// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{path::Path, sync::Arc};

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

// Node ID mapped to a pair of video frame buffers
type VideoNode = (usize, [Arc<Vec<u8>>; 2]);

pub struct WebVfxRenderer<const S: usize> {
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
                if let Ok(Some(node_id)) = document.query_selector(&format!("#webvfx-video{i}"))
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
    use super::*;
    use image::{ImageReader, RgbaImage};
    use testdir::testdir;

    fn read_image(path: &Path) -> Vec<u8> {
        match ImageReader::open(path).unwrap().decode().unwrap() {
            image::DynamicImage::ImageRgba8(image_buffer) => image_buffer.into_vec(),
            _ => panic!("image not rgba8"),
        }
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
        let fail_path = testdir!().join(reference_path.file_name().unwrap());
        if reference_path.exists() {
            if output.as_flat_samples().image_slice().unwrap()
                != read_image(reference_path).as_slice()
            {
                output.save(&fail_path).unwrap();
                panic!("Reference image differs, render saved to {fail_path:?}");
            }
        } else {
            output.save(&fail_path).unwrap();
            panic!("Reference not found, render saved to {fail_path:?}");
        }
    }

    #[test]
    fn test_filter() {
        const WIDTH: u32 = 320;
        const HEIGHT: u32 = 240;
        let testdata = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
        let html_path = testdata.join("filter.html");
        let html = std::fs::read_to_string(&html_path).unwrap();
        let url = Url::from_file_path(html_path.as_path()).unwrap();
        let mut r = WebVfxRenderer::<1>::new(&url, &html, WIDTH, HEIGHT);
        let mut output = RgbaImage::new(WIDTH, HEIGHT);

        render(
            0.0,
            &mut r,
            [&testdata.join("a-320x240.png")],
            &mut output,
            &testdata.join("filter-1.png"),
        );
        render(
            1.0,
            &mut r,
            [&testdata.join("b-320x240.png")],
            &mut output,
            &testdata.join("filter-2.png"),
        );
        render(
            2.0,
            &mut r,
            [&testdata.join("a-320x240.png")],
            &mut output,
            &testdata.join("filter-3.png"),
        );
    }
}
