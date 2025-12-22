use anyhow::Context;
use glob::glob;
use std::{
    env::{self, Args},
    fs,
    path::PathBuf,
};

fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    let task = args.nth(1);
    match task.as_deref() {
        Some("package") => package(args)?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:

package [output-dir]       package built plugins for frei0r
"
    )
}

fn package(mut args: Args) -> anyhow::Result<()> {
    let target_dir =
        PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()));
    let output_dir = if let Some(output) = args.next() {
        let output = PathBuf::from(output);
        fs::create_dir_all(&output)
            .with_context(|| format!("Failed to create directory '{}'", output.display()))?;
        Some(output)
    } else {
        None
    };
    let pattern = match env::consts::OS {
        "macos" => target_dir.join("*/libwebvfx_*.dylib"),
        "linux" => target_dir.join("*/libwebvfx_*.so"),
        _ => return Ok(()),
    };
    let pattern = pattern.to_str().ok_or(anyhow::anyhow!("Invalid path"))?;
    for entry in glob(pattern)? {
        let path = entry?;
        let path_ext = path.with_extension("so");
        let filename = path_ext
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_prefix("lib")
            .unwrap();
        let new_path = if let Some(ref output) = output_dir {
            output.join(filename)
        } else {
            path.with_file_name(filename)
        };
        let _ = fs::remove_file(&new_path); // Ignore error if nonexistent
        fs::hard_link(&path, &new_path)
            .with_context(|| format!("Link failed {} -> {}", path.display(), new_path.display()))?;
    }

    Ok(())
}
