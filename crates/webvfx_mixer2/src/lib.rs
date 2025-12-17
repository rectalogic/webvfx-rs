// Copyright (C) 2025 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use webvfx::{Mixer2Plugin, frei0r_rs2};

frei0r_rs2::plugin!(Mixer2Plugin);

#[cfg(test)]
mod tests {
    use super::*;
    use test_support::{
        HEIGHT, WIDTH, assert_output, param_cstring, param_string_ptr, read_image_u32,
    };

    #[test]
    fn test_mixer2() {
        let plugin = f0r_construct(WIDTH, HEIGHT);
        let html_path = param_cstring("mixer2.html");
        f0r_set_param_value(plugin, param_string_ptr(&html_path), 0);
        let mut output = vec![0u32; (WIDTH * HEIGHT) as usize];
        let inframe1 = read_image_u32("a-320x240.png");
        let inframe2 = read_image_u32("b-320x240.png");
        unsafe {
            f0r_update2(
                plugin,
                0.0,
                inframe1.as_ptr(),
                inframe2.as_ptr(),
                std::ptr::null(),
                output.as_mut_ptr(),
            )
        };
        assert_output("mixer2-1.png", &output);
        f0r_destruct(plugin);
    }
}
