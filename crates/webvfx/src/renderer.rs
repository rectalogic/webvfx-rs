// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex};

use anyrender::{ImageRenderer, PaintScene};
use anyrender_vello::VelloImageRenderer;
use blitz_dom::{DocumentConfig, IntrinsicSizes, Widget, node::ComputedStyles};
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::{
    net::Url,
    shell::{ColorScheme, Viewport},
};
use peniko::kurbo::{Affine, Rect, Vec2};
use peniko::{Extend, Fill, ImageBrush, ImageSampler};
use style::properties::generated::longhands::object_fit::computed_value::T as ObjectFit;

pub mod net;
pub mod processor;

pub const WEBVFX_SELECTOR_PREFIX: &str = "img.webvfx-video";
pub const WEBVFX_CSS_ANIMATION_PROPERTY: &str = "--webvfx-animation-duration";

/// Shared per-video state. The plugin writes the current frame into `staging`
/// and bumps `generation`; the widget uploads it to its WGPU texture.
#[derive(Debug)]
struct VideoSource {
    width: u32,
    height: u32,
    staging: Vec<u8>,
    generation: u64,
}

/// A Blitz custom widget that draws the current video frame from a WGPU texture.
///
/// Rendering the frame through a widget-owned texture avoids handing the
/// frame buffer to Vello as an `ImageData`, so Vello's image cache never
/// retains the plugin's frame buffers.
struct VideoWidget {
    source: Arc<Mutex<VideoSource>>,
    handle: Option<wgpu_context::DeviceHandle>,
    texture: Option<wgpu::Texture>,
    resource: Option<anyrender::ResourceId>,
    uploaded_generation: Option<u64>,
}

impl VideoWidget {
    fn new(source: Arc<Mutex<VideoSource>>) -> Self {
        Self {
            source,
            handle: None,
            texture: None,
            resource: None,
            uploaded_generation: None,
        }
    }
}

impl Widget for VideoWidget {
    fn can_create_surfaces(&mut self, render_ctx: &mut dyn anyrender::RenderContext) {
        let Some(handle) = render_ctx
            .renderer_specific_context()
            .and_then(|ctx| ctx.downcast::<wgpu_context::DeviceHandle>().ok())
            .map(|handle| *handle)
        else {
            return;
        };

        let (width, height) = {
            let source = self.source.lock().unwrap();
            (source.width, source.height)
        };
        let texture = handle.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("webvfx video frame"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        if let Ok(resource) = render_ctx.try_register_custom_resource(Box::new(texture.clone())) {
            self.resource = Some(resource);
        }
        self.texture = Some(texture);
        self.handle = Some(handle);
        self.uploaded_generation = None;
    }

    fn destroy_surfaces(&mut self) {
        self.texture = None;
        self.resource = None;
        self.handle = None;
        self.uploaded_generation = None;
    }

    fn requires_redraw(&self) -> bool {
        true
    }

    #[allow(clippy::cast_precision_loss)]
    fn intrinsic_sizes(&self) -> IntrinsicSizes {
        let source = self.source.lock().unwrap();
        let (width, height) = (source.width as f32, source.height as f32);
        IntrinsicSizes {
            width: Some(width),
            height: Some(height),
            ratio: Some(width / height),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn paint(
        &mut self,
        _render_ctx: &mut dyn anyrender::RenderContext,
        styles: &ComputedStyles,
        width: u32,
        height: u32,
        _scale: f64,
    ) -> anyrender::Scene {
        let mut scene = anyrender::Scene::new();
        let (Some(texture), Some(handle), Some(resource)) =
            (self.texture.as_ref(), self.handle.as_ref(), self.resource)
        else {
            return scene;
        };

        let (vf_width, vf_height) = {
            let source = self.source.lock().unwrap();
            if self.uploaded_generation != Some(source.generation) {
                handle.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &source.staging,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(source.width * 4),
                        rows_per_image: Some(source.height),
                    },
                    wgpu::Extent3d {
                        width: source.width,
                        height: source.height,
                        depth_or_array_layers: 1,
                    },
                );
                self.uploaded_generation = Some(source.generation);
            }
            (source.width as f32, source.height as f32)
        };

        if width == 0 || height == 0 || vf_width == 0.0 || vf_height == 0.0 {
            return scene;
        }

        // Match Blitz's `draw_image` object-fit/object-position behaviour.
        let container = (width as f32, height as f32);
        let object = (vf_width, vf_height);
        let paint_size = compute_object_fit(container, object, styles.clone_object_fit());
        let offset = (
            (container.0 - paint_size.0) / 2.0,
            (container.1 - paint_size.1) / 2.0,
        );
        let transform = Affine::scale_non_uniform(
            f64::from(paint_size.0) / f64::from(object.0),
            f64::from(paint_size.1) / f64::from(object.1),
        )
        .then_translate(Vec2::new(f64::from(offset.0), f64::from(offset.1)));

        let brush = ImageBrush {
            image: resource,
            sampler: ImageSampler {
                x_extend: Extend::Repeat,
                y_extend: Extend::Repeat,
                quality: peniko::ImageQuality::Medium,
                alpha: 1.0,
            },
        };
        scene.fill(
            Fill::NonZero,
            transform,
            &anyrender::Paint::Resource(brush),
            None,
            &Rect::new(0.0, 0.0, f64::from(object.0), f64::from(object.1)),
        );
        scene
    }
}

/// Mirrors `blitz_paint`'s object-fit sizing (CSS Images 3).
fn compute_object_fit(container: (f32, f32), object: (f32, f32), fit: ObjectFit) -> (f32, f32) {
    let contain = (container.0 / object.0).min(container.1 / object.1);
    match fit {
        ObjectFit::None => object,
        ObjectFit::Fill => container,
        ObjectFit::Cover => {
            let ratio = (container.0 / object.0).max(container.1 / object.1);
            (object.0 * ratio, object.1 * ratio)
        }
        ObjectFit::Contain => (object.0 * contain, object.1 * contain),
        ObjectFit::ScaleDown => {
            let scaled = (object.0 * contain, object.1 * contain);
            if object.0 < scaled.0 { object } else { scaled }
        }
    }
}

struct WebVfxRenderer<const S: usize> {
    width: u32,
    height: u32,
    document: HtmlDocument,
    renderer: VelloImageRenderer,
    video_sources: [Option<Arc<Mutex<VideoSource>>>; S],
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
                net_provider: Some(Arc::new(net::SyncNetProvider::new())),
                viewport: Some(Viewport::new(width, height, 1.0, ColorScheme::Light)),
                ..Default::default()
            },
        );
        let video_sources: [Option<Arc<Mutex<VideoSource>>>; S] = (0..S)
            .map(|i| {
                let Ok(node_ids) =
                    document.query_selector_all(&format!("{}{}", WEBVFX_SELECTOR_PREFIX, i + 1))
                else {
                    return None;
                };
                if node_ids.is_empty() {
                    return None;
                }
                let source = Arc::new(Mutex::new(VideoSource {
                    width,
                    height,
                    staging: vec![0u8; (width * height * 4) as usize],
                    generation: 0,
                }));
                node_ids.iter().copied().for_each(|node_id| {
                    document.set_custom_widget(node_id, Box::new(VideoWidget::new(source.clone())));
                });
                Some(source)
            })
            .collect::<Vec<Option<Arc<Mutex<VideoSource>>>>>()
            .try_into()
            .unwrap();

        let renderer = VelloImageRenderer::new(width, height);
        Self {
            width,
            height,
            document,
            renderer,
            video_sources,
        }
    }

    fn update(&mut self, time: f64, inframes: [&[u8]; S], outframe: &mut [u8]) {
        self.video_sources
            .iter()
            .zip(inframes)
            .filter_map(|(source, inframe)| source.as_ref().map(|source| (source, inframe)))
            .for_each(|(source, inframe)| {
                let mut source = source.lock().unwrap();
                source.staging.copy_from_slice(inframe);
                source.generation = source.generation.wrapping_add(1);
            });
        self.document.resolve(time);
        self.renderer.render(
            |scene| {
                scene.reset();
                paint_scene(
                    scene,
                    &mut self.document,
                    1.0,
                    self.width,
                    self.height,
                    0,
                    0,
                );
            },
            outframe,
        );
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::process_template;

    use super::*;
    use test_support::{HEIGHT, RgbaImage, WIDTH, assert_reference, read_image, testdata};

    fn init_renderer<const S: usize>(
        html_file: &str,
        json_file: Option<&str>,
    ) -> (WebVfxRenderer<S>, RgbaImage) {
        let (url, html) = process_template(
            testdata!().join(html_file),
            json_file.map(|f| testdata!().join(f)),
        )
        .unwrap();
        let renderer = WebVfxRenderer::<S>::new(&url, &html, "5s", WIDTH, HEIGHT);
        let output = RgbaImage::new(WIDTH, HEIGHT);
        (renderer, output)
    }

    fn render<const S: usize>(
        time: f64,
        renderer: &mut WebVfxRenderer<S>,
        inframe_paths: [&Path; S],
        output: &mut RgbaImage,
        reference_file: &str,
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
        assert_reference(reference_file, output);
    }

    #[test]
    fn object_fit_contain_fills_box() {
        assert_eq!(
            compute_object_fit((200.0, 200.0), (320.0, 240.0), ObjectFit::Contain),
            (200.0, 150.0)
        );
        assert_eq!(
            compute_object_fit((200.0, 200.0), (320.0, 240.0), ObjectFit::Fill),
            (200.0, 200.0)
        );
        assert_eq!(
            compute_object_fit((200.0, 200.0), (320.0, 240.0), ObjectFit::None),
            (320.0, 240.0)
        );
    }

    #[test]
    fn test_source() {
        let (mut r, mut output) = init_renderer::<0>("source.html", None);
        render(0.0, &mut r, [], &mut output, "source-1.png");
    }

    #[test]
    fn test_source_template() {
        let (mut r, mut output) =
            init_renderer::<0>("source-template.html", Some("source-template.json"));
        render(0.0, &mut r, [], &mut output, "source-template-1.png");
    }

    #[test]
    fn test_filter() {
        let (mut r, mut output) = init_renderer::<1>("filter.html", None);
        render(
            0.0,
            &mut r,
            [&testdata!().join("a-320x240.png")],
            &mut output,
            "filter-1.png",
        );
        render(
            1.0,
            &mut r,
            [&testdata!().join("b-320x240.png")],
            &mut output,
            "filter-2.png",
        );
        render(
            2.0,
            &mut r,
            [&testdata!().join("a-320x240.png")],
            &mut output,
            "filter-3.png",
        );
    }

    #[test]
    fn test_mixer2() {
        let (mut r, mut output) = init_renderer::<2>("mixer2.html", None);
        render(
            0.0,
            &mut r,
            [
                &testdata!().join("a-320x240.png"),
                &testdata!().join("b-320x240.png"),
            ],
            &mut output,
            "mixer2-1.png",
        );
        render(
            1.0,
            &mut r,
            [
                &testdata!().join("b-320x240.png"),
                &testdata!().join("a-320x240.png"),
            ],
            &mut output,
            "mixer2-2.png",
        );
        render(
            2.0,
            &mut r,
            [
                &testdata!().join("a-320x240.png"),
                &testdata!().join("b-320x240.png"),
            ],
            &mut output,
            "mixer2-3.png",
        );
    }

    fn test_mixer3_base(html_file: &str, reference_files: [&str; 3]) {
        let (mut r, mut output) = init_renderer::<3>(html_file, None);
        render(
            0.0,
            &mut r,
            [
                &testdata!().join("a-320x240.png"),
                &testdata!().join("b-320x240.png"),
                &testdata!().join("c-320x240.png"),
            ],
            &mut output,
            reference_files[0],
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
            reference_files[1],
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
            reference_files[2],
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
