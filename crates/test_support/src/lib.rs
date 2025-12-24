use std::path::Path;

use image::ImageReader;
pub use image::RgbaImage;
use testdir::testdir;

use std::ffi::CString;

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 240;

pub const TEST_ROOT: &str = env!("CARGO_MANIFEST_DIR");

#[macro_export]
macro_rules! testdata {
    () => {
        std::path::Path::new($crate::TEST_ROOT).join("testdata")
    };
}

pub fn read_image(path: &Path) -> Vec<u8> {
    match ImageReader::open(path).unwrap().decode().unwrap() {
        image::DynamicImage::ImageRgba8(image_buffer) => image_buffer.into_vec(),
        _ => panic!("image not rgba8"),
    }
}

pub fn assert_reference(reference_file: &str, output: &RgbaImage) {
    cfg_if::cfg_if! {
        if #[cfg(feature = "anyrender_vello_cpu")] {
            let reference_dir = "output/anyrender_vello_cpu";
        }
        else if #[cfg(feature = "anyrender_skia")] {
            let reference_dir = "output/anyrender_skia";
        }
        else if #[cfg(feature = "anyrender_vello")] {
            let reference_dir = "output/anyrender_vello";
        }
    }
    let reference_file = testdata!().join(reference_dir).join(reference_file);
    let fail_path = testdir!().join(reference_file.file_name().unwrap());
    if reference_file.exists() {
        if output.as_flat_samples().image_slice().unwrap() != read_image(&reference_file).as_slice()
        {
            output.save(&fail_path).unwrap();
            panic!(
                "Reference image differs, render saved to {}",
                fail_path.display()
            );
        }
    } else {
        output.save(&fail_path).unwrap();
        panic!(
            "Reference '{}' not found, render saved to '{}'",
            reference_file.display(),
            fail_path.display()
        );
    }
}

pub fn param_cstring(filename: &str) -> CString {
    CString::new(testdata!().join(filename).to_str().unwrap()).unwrap()
}

pub fn assert_output(reference_file: &str, output: &Vec<u32>) {
    let output_slice = output.as_slice();
    let bytes = unsafe {
        std::slice::from_raw_parts(
            output_slice.as_ptr().cast::<u8>(),
            size_of_val(output_slice),
        )
    };
    let output = RgbaImage::from_raw(WIDTH, HEIGHT, bytes.into()).unwrap();
    assert_reference(reference_file, &output);
}

pub fn read_image_u32(filename: &str) -> Vec<u32> {
    let bytes = read_image(&testdata!().join(filename));
    assert!((bytes.as_ptr() as usize).is_multiple_of(std::mem::align_of::<u32>()));
    #[allow(clippy::ptr_as_ptr, clippy::cast_ptr_alignment)]
    let u32_slice =
        unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u32, bytes.len() / 4) };
    u32_slice.into()
}
