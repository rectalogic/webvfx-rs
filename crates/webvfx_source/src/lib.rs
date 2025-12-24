// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use webvfx::{SourcePlugin, frei0r_rs2};

frei0r_rs2::plugin!(SourcePlugin);

#[cfg(test)]
mod tests {
    use std::{ffi::c_void, ptr};

    use super::*;
    use test_support::{HEIGHT, WIDTH, assert_output, param_cstring};

    #[test]
    fn test_source() {
        let plugin = f0r_construct(WIDTH, HEIGHT);
        let html_path = param_cstring("source.html");
        let html_ptr = html_path.as_ptr();
        let html_param = &raw const html_ptr as *mut c_void;
        f0r_set_param_value(plugin, html_param, 0);
        let mut output = vec![0u32; (WIDTH * HEIGHT) as usize];
        unsafe { f0r_update(plugin, 0.0, ptr::null::<u32>(), output.as_mut_ptr()) };
        assert_output("source-1.png", &output);
        f0r_destruct(plugin);
    }
}
