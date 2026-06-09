use anyhow::{Context, Result};
use byte_unit::Byte;
use filesize::PathExt;
use std::{
    env,
    fmt::format,
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Item {
    size: u64,
    name: String,
}
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut path = String::new();
    if args.len() == 1 {
        path = String::from(".");
    } else {
        path = String::from(&args[1]);
    }
    // aggregate
    let dir_list = get_path_list(&path)?;
    let mut items = Vec::new();
    for dir in dir_list {
        let filename = dir.file_name().unwrap().to_string_lossy();
        let size = compute_dir_size(&dir)?;
        items.push(Item {
            size,
            name: filename.to_string(),
        });
    }
    // sort and output
    items.sort();
    for item in items {
        let Item {
            size,
            name: filename,
        } = item;
        let size = Byte::from_u64(size).get_appropriate_unit(byte_unit::UnitType::Binary);
        let size = format!("{size:.2}");
        let width = 10;
        println!(" {:>width$} {}", size, filename);
    }
    Ok(())
}

// 获取指定目录的子条目数组
fn get_path_list(path: &str) -> Result<Vec<PathBuf>> {
    let mut dit_list = Vec::new();
    let dir = fs::read_dir(path)?;
    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        if let Ok(meta) = path.symlink_metadata()
            && meta.is_symlink()
        {
            continue;
        }
        dit_list.push(path);
    }
    Ok(dit_list)
}
fn compute_dir_size(dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    if !dir.is_dir() {
        let metadata = dir.metadata()?;
        return dir
            .size_on_disk_fast(&metadata)
            .context("failed to read file metadata");
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            total_size += compute_dir_size(&path)?;
        } else {
            let metadata = path.metadata()?;
            total_size += path.size_on_disk_fast(&metadata)?;
        }
    }
    Ok(total_size)
}
