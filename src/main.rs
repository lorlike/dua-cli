use anyhow::{Context, Result};
use byte_unit::Byte;
use filesize::PathExt;
use std::{
    collections::HashSet,
    env, fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Item {
    size: u64,
    name: String,
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
        let mut inode_filter = InodeFilter::default();
        let filename = dir.file_name().unwrap().to_string_lossy();
        let size = compute_path_size(&dir, &mut inode_filter)?;
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
