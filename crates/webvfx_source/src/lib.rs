// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use webvfx::{SourcePlugin, frei0r_rs2};

frei0r_rs2::plugin!(SourcePlugin);

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::*;
    use webvfx_test_support::{HEIGHT, WIDTH, assert_output, param_cstring, param_string_ptr};

    #[test]
    fn test_source() {
        let plugin = f0r_construct(WIDTH, HEIGHT);
        let html_path = param_cstring("source.html");
        f0r_set_param_value(plugin, param_string_ptr(&html_path), 0);
        let mut output = vec![0u32; (WIDTH * HEIGHT) as usize];
        unsafe { f0r_update(plugin, 0.0, ptr::null::<u32>(), output.as_mut_ptr()) };
        assert_output("source-1.png", &output);
        f0r_destruct(plugin);
    }
}
