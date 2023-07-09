use anyhow::{Context, Result};
use clap::Parser;
use std::fs::{read_dir, remove_dir, rename};
use std::io;
use std::path::{Path, PathBuf};
use unicode_normalization::{is_nfc, UnicodeNormalization};

#[derive(Parser)]
#[command(version, author, about)]
struct Args {
    /// Path to the root directory to process.
    #[arg(value_name = "PATH", required = true)]
    paths: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args: Args = Args::parse();

    for path in &args.paths {
        scan(path)?;
    }

    Ok(())
}

fn scan(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    for entry in
        read_dir(path).with_context(|| format!("Failed to process `{}`", path.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            scan(&path).with_context(|| format!("Failed to process `{}`", path.display()))?;
            process_dir(&path)
                .with_context(|| format!("Failed to process `{}`", path.display()))?;
        } else {
            process_file(&path)
                .with_context(|| format!("Failed to process `{}`", path.display()))?;
        }
    }

    Ok(())
}

fn process_file(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let file_name = path.file_name().unwrap().to_str().unwrap();
    if !is_nfc(file_name) {
        let normalized_path = path.with_file_name(file_name.nfc().collect::<String>());
        rename(path, normalized_path)?;
    }

    Ok(())
}

fn process_dir(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    let file_name = path.file_name().unwrap().to_str().unwrap();
    if !is_nfc(file_name) {
        let normalized_path = path.with_file_name(file_name.nfc().collect::<String>());
        if normalized_path.exists() {
            merge_dir(path, normalized_path)?;
        } else {
            rename(path, normalized_path)?;
        }
    }

    Ok(())
}

fn merge_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    for entry in read_dir(from)? {
        let entry = entry?;
        let to = to.join(entry.file_name());
        rename(entry.path(), to)?;
    }

    remove_dir(from)
}
