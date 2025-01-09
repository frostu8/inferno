use color_eyre::{Section, SectionExt};
use eyre::{Report, WrapErr};

use std::env;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Report> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();

    let webpack_dir = manifest_dir.join("web");

    let webpack_out_dir = manifest_dir.join("site");

    for entry in read_dir(&webpack_dir)?.flatten() {
        if entry.path().file_name() == Some(OsStr::new("node_modules")) {
            continue;
        }

        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    // change working directory
    env::set_current_dir(&webpack_dir)?;

    // run rollup command
    let handle = Command::new("npx")
        .args(["rollup", "-c"])
        .env("WEBPACK_OUT_DIR", webpack_out_dir)
        .spawn()?;
    let output = handle
        .wait_with_output()
        .wrap_err("rollup command failed")?;
    if !output.status.success() {
        let output = String::from_utf8_lossy(&output.stdout).into_owned();
        return Err(Report::msg("rollup command failed").section(output.header("Rollup output")));
    }

    Ok(())
}
