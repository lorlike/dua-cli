use anyhow::{Context, Result};
use byte_unit::Byte;
use filesize::PathExt;
use owo_colors::OwoColorize;
use std::{
    collections::HashSet,
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

mod args;
mod ui;
pub use args::{Args, Command};
pub use ui::App;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Item {
    pub size: u64,
    pub name: String,
    pub is_dir: bool,
}
#[derive(Default)]
struct InodeFilter {
    seen: HashSet<(u64, u64)>,
}

impl InodeFilter {
    fn should_count(&mut self, metadata: &fs::Metadata) -> bool {
        let dev_ino = (metadata.dev(), metadata.ino());
        self.seen.insert(dev_ino)
    }
}

pub fn aggregate(paths: Vec<PathBuf>) -> Result<Vec<Item>> {
    let mut items = Vec::new();
    for dir in paths {
        let mut inode_filter = InodeFilter::default();
        let filename = dir
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| dir.to_string_lossy().to_string());
        let size = compute_path_size(&dir, &mut inode_filter)?;
        items.push(Item {
            size,
            name: filename,
            is_dir: dir.is_dir(),
        });
    }
    Ok(items)
}
// sort and output with colors
pub fn output_with_color(mut items: Vec<Item>) {
    let mut total_size = 0u64;
    items.sort();
    for item in items {
        let Item {
            size,
            name: filename,
            is_dir,
        } = item;
        total_size += size;
        let size = Byte::from_u64(size).get_appropriate_unit(byte_unit::UnitType::Binary);
        let size = format!("{size:.2}");
        let width = 10;
        if is_dir {
            let filename = filename.cyan();
            println!(" {:>width$} {}", size.green(), filename);
        } else {
            println!(" {:>width$} {}", size.green(), filename);
        }
    }
    let size = Byte::from_u64(total_size).get_appropriate_unit(byte_unit::UnitType::Binary);
    let size = format!("{size:.2}");
    let width = 10;
    println!(" {:>width$} total", size.green());
}
// 获取指定目录的子条目数组
pub fn get_path_list(path: &str) -> Result<Vec<PathBuf>> {
    let mut dir_list = Vec::new();
    let dir = fs::read_dir(path)?;
    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        if let Ok(meta) = path.symlink_metadata()
            && meta.is_symlink()
        {
            continue;
        }
        dir_list.push(path);
    }
    Ok(dir_list)
}
fn compute_size_once(path: &Path, inode_filter: &mut InodeFilter) -> Result<u64> {
    let metadata = path.metadata()?;
    if inode_filter.should_count(&metadata) {
        path.size_on_disk_fast(&metadata)
            .context("failed to read file metadata")
    } else {
        Ok(0)
    }
}
fn compute_path_size(dir: &Path, inode_filter: &mut InodeFilter) -> Result<u64> {
    let mut total_size = compute_size_once(dir, inode_filter)?;
    if !dir.is_dir() {
        return Ok(total_size);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            total_size += compute_path_size(&path, inode_filter)?;
        } else {
            total_size += compute_size_once(&path, inode_filter)?;
        }
    }
    Ok(total_size)
}
