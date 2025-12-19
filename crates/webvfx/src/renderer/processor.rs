// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    path::{self, Path},
    sync::mpsc::{Receiver, Sender, channel},
    thread::{self, JoinHandle},
};

use super::WebVfxRenderer;
use anyhow::Context;
use blitz_traits::net::Url;

struct RenderJob<const S: usize> {
    time: f64,
    inputs: [(*const u8, usize); S],
    output: (*mut u8, usize),
}

// SAFETY: The caller guarantees:
// 1. input and output references remain valid until done_tx signals completion
// 2. No other thread accesses output during processing
unsafe impl<const S: usize> Send for RenderJob<S> {}

impl<const S: usize> RenderJob<S> {
    fn new(time: f64, inputs: [&[u32]; S], output: &mut [u32]) -> Self {
        let inputs: [(*const u8, usize); S] = inputs
            .into_iter()
            .map(|input| (input.as_ptr().cast::<u8>(), size_of_val(input)))
            .collect::<Vec<(*const u8, usize)>>()
            .try_into()
            .unwrap();
        Self {
            time,
            inputs,
            output: (output.as_mut_ptr().cast::<u8>(), size_of_val(output)),
        }
    }
}

pub struct RenderProcessor<const S: usize> {
    job_tx: Option<Sender<RenderJob<S>>>,
    job_done_rx: Receiver<()>,
    worker: Option<JoinHandle<()>>,
}

impl<const S: usize> RenderProcessor<S> {
    pub fn new(
        html_path: impl AsRef<Path>,
        json_path: Option<impl AsRef<Path>>,
        animation_duration: &str,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Self> {
        let (url, html) = process_template(html_path, json_path)?;

        let (job_tx, job_rx) = channel::<RenderJob<S>>();
        let (job_done_tx, job_done_rx) = channel::<()>();

        let animation_duration = String::from(animation_duration);
        let worker = thread::spawn(move || {
            let mut renderer =
                WebVfxRenderer::<S>::new(&url, &html, &animation_duration, width, height);
            while let Ok(job) = job_rx.recv() {
                let inputs: [&[u8]; S] = job
                    .inputs
                    .into_iter()
                    .map(|(input_ptr, input_len)| unsafe {
                        std::slice::from_raw_parts(input_ptr, input_len)
                    })
                    .collect::<Vec<&[u8]>>()
                    .try_into()
                    .unwrap();
                let output = unsafe { std::slice::from_raw_parts_mut(job.output.0, job.output.1) };

                renderer.update(job.time, inputs, output);

                if job_done_tx.send(()).is_err() {
                    return;
                }
            }
        });

        Ok(Self {
            job_tx: Some(job_tx),
            job_done_rx,
            worker: Some(worker),
        })
    }

    pub fn update(&self, time: f64, inputs: [&[u32]; S], output: &mut [u32]) -> anyhow::Result<()> {
        let job = RenderJob::new(time, inputs, output);
        self.job_tx
            .as_ref()
            .unwrap()
            .send(job)
            .map_err(|e| anyhow::anyhow!("Worker thread exited: {e:?}"))?;
        self.job_done_rx.recv()?;
        Ok(())
    }
}

impl<const S: usize> Drop for RenderProcessor<S> {
    fn drop(&mut self) {
        drop(self.job_tx.take().unwrap());
        let worker = self.worker.take().unwrap();
        if let Err(e) = worker.join() {
            eprintln!("WebVfx: worker failed to exit: {e:?}");
        }
    }
}

#[allow(clippy::missing_errors_doc)]
pub fn process_template(
    html_path: impl AsRef<Path>,
    json_path: Option<impl AsRef<Path>>,
) -> anyhow::Result<(Url, String)> {
    let html_path = html_path.as_ref();
    let absolute_html_path = path::absolute(Path::new(html_path)).with_context(|| {
        format!(
            "WebVfx: failed to make HTML path '{}' absolute",
            html_path.display()
        )
    })?;
    let html = std::fs::read_to_string(&absolute_html_path).with_context(|| {
        format!(
            "WebVfx: failed to read HTML path '{}'",
            absolute_html_path.display()
        )
    })?;
    let url = Url::from_file_path(&absolute_html_path).map_err(|()| {
        anyhow::anyhow!(
            "WebVfx: failed to convert HTML path '{}' to file: URL",
            absolute_html_path.display()
        )
    })?;
    if let Some(json_path) = json_path {
        let json_path = json_path.as_ref();
        let json_str = std::fs::read_to_string(json_path).with_context(|| {
            format!("WebVfx: failed to read JSON path '{}'", json_path.display())
        })?;
        let json_value: serde_json::Value = serde_json::from_str(&json_str).with_context(|| {
            format!(
                "WebVfx: failed to parse JSON path '{}'",
                json_path.display()
            )
        })?;
        let context = tera::Context::from_value(json_value).with_context(|| {
            format!(
                "WebVfx: failed to configure template context from JSON '{}'",
                json_path.display()
            )
        })?;
        Ok((
            url,
            tera::Tera::one_off(&html, &context, true)
                .context("WebVfx: Failed to render template")?,
        ))
    } else {
        Ok((url, html))
    }
}
