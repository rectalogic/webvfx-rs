use image::ImageReader;
pub use image::RgbaImage;
use std::{
    env::consts,
    error::Error,
    ffi::CString,
    fmt,
    path::{Path, PathBuf},
};
use testdir::testdir;

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 240;

pub const TEST_ROOT: &str = env!("CARGO_MANIFEST_DIR");

#[derive(Debug)]
pub struct ImageDiffError {
    reference_file: PathBuf,
    failed_file: PathBuf,
    diff_file: PathBuf,
    similarity: f64,
}

impl fmt::Display for ImageDiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Reference image `{}` differs, render saved to `{}`, diff saved to `{}`. Similarity {}",
            self.reference_file.display(),
            self.failed_file.display(),
            self.diff_file.display(),
            self.similarity
        )
    }
}

impl Error for ImageDiffError {}

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

#[expect(clippy::missing_errors_doc)]
pub fn assert_reference(reference_file: &str, output: &RgbaImage) -> Result<(), ImageDiffError> {
    let reference_file = testdata!()
        .join("output")
        .join(consts::OS)
        .join(reference_file);
    let failed_file = testdir!(ModuleScope).join(reference_file.file_name().unwrap());
    if reference_file.exists() {
        let mut diff_file = testdir!(ModuleScope).join(reference_file.file_name().unwrap());
        diff_file.set_extension("diff.png");
        let reference_image = image::open(&reference_file).unwrap().into_rgba8();
        let compare_result = image_compare::rgba_hybrid_compare(&reference_image, output).unwrap();
        if compare_result.score < 1.0 {
            output.save(&failed_file).unwrap();
            compare_result
                .image
                .to_color_map()
                .save(&diff_file)
                .unwrap();
            return Err(ImageDiffError {
                reference_file,
                failed_file,
                diff_file,
                similarity: compare_result.score,
            });
        }
    } else {
        output.save(&failed_file).unwrap();
        panic!(
            "Reference '{}' not found, render saved to '{}'",
            reference_file.display(),
            failed_file.display()
        );
    }
    Ok(())
}

pub fn param_cstring(filename: &str) -> CString {
    CString::new(testdata!().join(filename).to_str().unwrap()).unwrap()
}

#[expect(clippy::missing_errors_doc)]
pub fn assert_output(reference_file: &str, output: &Vec<u32>) -> Result<(), ImageDiffError> {
    let output_slice = output.as_slice();
    let bytes = unsafe {
        std::slice::from_raw_parts(
            output_slice.as_ptr().cast::<u8>(),
            size_of_val(output_slice),
        )
    };
    let output = RgbaImage::from_raw(WIDTH, HEIGHT, bytes.into()).unwrap();
    assert_reference(reference_file, &output)
}

pub fn read_image_u32(filename: &str) -> Vec<u32> {
    let bytes = read_image(&testdata!().join(filename));
    assert!((bytes.as_ptr() as usize).is_multiple_of(std::mem::align_of::<u32>()));
    #[allow(clippy::ptr_as_ptr, clippy::cast_ptr_alignment)]
    let u32_slice =
        unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u32, bytes.len() / 4) };
    u32_slice.into()
}
