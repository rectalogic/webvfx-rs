// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use blitz_traits::net::{Bytes, NetHandler, NetProvider, Request};
use data_url::DataUrl;

pub(super) struct FileProvider;

impl FileProvider {
    fn fetch_inner(&self, request: Request) -> anyhow::Result<Bytes> {
        match request.url.scheme() {
            "data" => {
                let data_url = DataUrl::process(request.url.as_str())?;
                let decoded = data_url.decode_to_vec()?;
                Ok(Bytes::from(decoded.0))
            }
            "file" => {
                let file_content = std::fs::read(request.url.path())?;
                Ok(Bytes::from(file_content))
            }
            scheme => Err(anyhow::anyhow!("WebVfx: Unsupported url scheme {scheme}")),
        }
    }
}

impl NetProvider for FileProvider {
    fn fetch(&self, _doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let url = request.url.to_string();
        match self.fetch_inner(request) {
            Err(e) => eprintln!("Failed to fetch url {}: {e:?}", url),
            Ok(bytes) => handler.bytes(url, bytes),
        }
    }
}
