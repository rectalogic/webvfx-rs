// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Context;
use blitz_traits::net::{Body, Bytes, NetHandler, NetProvider, Request};
use data_url::DataUrl;
use reqwest::blocking::Response;

#[derive(Default)]
pub struct SyncNetProvider {
    client: reqwest::blocking::Client,
}

impl SyncNetProvider {
    pub fn new() -> Self {
        Self::default()
    }

    fn fetch_inner(&self, request: Request) -> anyhow::Result<Bytes> {
        match request.url.scheme() {
            "data" => {
                let data_url = DataUrl::process(request.url.as_str())?;
                let decoded = data_url.decode_to_vec()?;
                Ok(Bytes::from(decoded.0))
            }
            "file" => {
                let file_content = std::fs::read(
                    request
                        .url
                        .to_file_path()
                        .map_err(|()| anyhow::anyhow!("cannot convert URL to path"))?,
                )?;

                Ok(Bytes::from(file_content))
            }
            _ => {
                let url = request.url.to_string();
                let mut builder = self
                    .client
                    .request(request.method, request.url)
                    .headers(request.headers)
                    .header("Content-Type", request.content_type.as_str());
                if let Body::Bytes(bytes) = request.body {
                    builder = builder.body(bytes);
                }
                Ok(builder
                    .send()
                    .and_then(Response::bytes)
                    .with_context(|| format!("WebVfx: failed to request URL '{url}'"))?)
            }
        }
    }
}

impl NetProvider for SyncNetProvider {
    fn fetch(&self, _doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let url = request.url.to_string();
        match self.fetch_inner(request) {
            Err(e) => eprintln!("WebVfx: failed to fetch url {url}: {e:?}"),
            Ok(bytes) => handler.bytes(url, bytes),
        }
    }
}
