// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
    path::{self, Path, PathBuf},
    process::exit,
    sync::Arc,
};

use argh::FromArgs;
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use blitz_shell::{
    BlitzApplication, BlitzShellEvent, Window, WindowConfig, create_default_event_loop,
};
use blitz_traits::net::Url;
use webvfx::FileProvider;
use winit::dpi::LogicalSize;

#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help", "help"))]
/// WebVfx HTML viewer
struct Args {
    #[argh(option, default = "640")]
    /// width of browser window
    width: usize,
    #[argh(option, default = "360")]
    /// height of browser window
    height: usize,
    #[argh(positional)]
    /// path to HTML file
    file: String,
}

fn main() {
    let args: Args = argh::from_env();

    let (url, html) = match path_url(&args.file) {
        Ok((url, path)) => match std::fs::read_to_string(&path) {
            Ok(html) => (url, html),
            Err(e) => {
                eprintln!("Unable to read {} :{e}", path.display());
                exit(1);
            }
        },
        Err(e) => {
            eprintln!("Invalid file: {e}");
            exit(1);
        }
    };

    let doc = HtmlDocument::from_html(
        &html,
        DocumentConfig {
            base_url: Some(url.into()),
            // ua_stylesheets: Some(cfg.stylesheets),
            net_provider: Some(Arc::new(FileProvider)),
            ..Default::default()
        },
    );
    //XXX accept list of images on cli and insert them into webvfx-imageX elements
    let renderer = anyrender_vello::VelloWindowRenderer::new();
    let window = WindowConfig::with_attributes(
        Box::new(doc) as _,
        renderer,
        Window::default_attributes()
            .with_inner_size(LogicalSize::new(args.width as f64, args.height as f64)),
    );

    let event_loop = create_default_event_loop::<BlitzShellEvent>();
    let mut application = BlitzApplication::new(event_loop.create_proxy());
    application.add_window(window);

    event_loop.run_app(&mut application).unwrap()
}

fn path_url(path: &str) -> anyhow::Result<(Url, PathBuf)> {
    let path = path::absolute(Path::new(path))?;
    let url =
        Url::from_file_path(&path).map_err(|_| anyhow::anyhow!("File path must be absolute"))?;
    Ok((url, path))
}
