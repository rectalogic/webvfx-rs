// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use webvfx::{Mixer3Plugin, frei0r_rs2};

frei0r_rs2::plugin!(Mixer3Plugin);

#[cfg(test)]
mod tests {
    use std::ffi::c_void;

    use super::*;
    use test_support::{HEIGHT, WIDTH, assert_output, param_cstring, read_image_u32};

    #[test]
    fn test_mixer3() {
        let plugin = f0r_construct(WIDTH, HEIGHT);
        let html_path = param_cstring("mixer3.html");
        let html_ptr = html_path.as_ptr();
        let html_param = &raw const html_ptr as *mut c_void;
        f0r_set_param_value(plugin, html_param, 0);
        let mut output = vec![0u32; (WIDTH * HEIGHT) as usize];
        let inframe1 = read_image_u32("a-320x240.png");
        let inframe2 = read_image_u32("b-320x240.png");
        let inframe3 = read_image_u32("c-320x240.png");
        unsafe {
            f0r_update2(
                plugin,
                0.0,
                inframe1.as_ptr(),
                inframe2.as_ptr(),
                inframe3.as_ptr(),
                output.as_mut_ptr(),
            );
        }
        assert_output("mixer3-1.png", &output);
        f0r_destruct(plugin);
    }
}
