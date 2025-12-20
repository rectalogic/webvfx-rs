// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
    path::{self, Path, PathBuf},
    process::exit,
    sync::Arc,
};

use argh::FromArgs;
use blitz_dom::{DocumentConfig, qual_name};
use blitz_html::HtmlDocument;
use blitz_shell::{
    BlitzApplication, BlitzShellEvent, Window, WindowConfig, create_default_event_loop,
};
use blitz_traits::net::Url;
use webvfx::{
    SyncNetProvider, WEBVFX_CSS_ANIMATION_PROPERTY, WEBVFX_SELECTOR_PREFIX, process_template,
};
use winit::dpi::LogicalSize;

#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help", "help"))]
#[allow(clippy::doc_markdown)]
/// WebVfx HTML viewer
struct Args {
    #[argh(option, default = "640")]
    /// width of browser window
    width: usize,

    #[argh(option, default = "360")]
    /// height of browser window
    height: usize,

    #[argh(option, short = 'd', default = r#"String::from("5s")"#)]
    /// CSS animation duration (specify s or ms suffix)
    animation_duration: String,

    #[argh(option)]
    /// image paths to insert into HTML
    image: Vec<String>,

    #[argh(option)]
    /// JSON path to template data
    json: Option<String>,

    #[argh(positional)]
    /// path to HTML file
    html: String,
}

fn main() {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt::init();

    let args: Args = argh::from_env();

    let (url, html) = match process_template(&args.html, args.json) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
    };

    let mut document = HtmlDocument::from_html(
        &html,
        DocumentConfig {
            base_url: Some(url.into()),
            ua_stylesheets: Some(vec![format!(
                r"
                :root {{
                    {WEBVFX_CSS_ANIMATION_PROPERTY}: {}
                }}
                ",
                args.animation_duration
            )]),
            net_provider: Some(Arc::new(SyncNetProvider::new())),
            ..Default::default()
        },
    );

    for (index, image) in args.image.iter().enumerate() {
        let selector = format!("{}{}", WEBVFX_SELECTOR_PREFIX, index + 1);
        if let Ok(node_ids) = document.query_selector_all(&selector)
            && !node_ids.is_empty()
        {
            if let Ok((url, _)) = path_url(image) {
                node_ids.iter().copied().for_each(|node_id| {
                    document
                        .mutate()
                        .set_attribute(node_id, qual_name!("src"), url.as_str());
                });
            } else {
                eprintln!("Invalid image path '{image}', ignoring");
            }
        } else {
            eprintln!("Selector {selector} not found in document");
        }
    }

    let renderer = anyrender_vello::VelloWindowRenderer::new();
    let window = WindowConfig::with_attributes(
        Box::new(document) as _,
        renderer,
        #[allow(clippy::cast_precision_loss)]
        Window::default_attributes()
            .with_inner_size(LogicalSize::new(args.width as f64, args.height as f64))
            .with_title("WebVfx Viewer"),
    );

    let event_loop = create_default_event_loop::<BlitzShellEvent>();
    let mut application = BlitzApplication::new(event_loop.create_proxy());
    application.add_window(window);

    event_loop.run_app(&mut application).unwrap();
}

fn path_url(path: &str) -> anyhow::Result<(Url, PathBuf)> {
    let path = path::absolute(Path::new(path))?;
    let url = Url::from_file_path(&path)
        .map_err(|()| anyhow::anyhow!("HTML file path must be absolute"))?;
    Ok((url, path))
}
