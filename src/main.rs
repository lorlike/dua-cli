use anyhow::Result;
use byte_unit::Byte;
use std::{env, fmt::format, fs, os::unix::fs::MetadataExt, path::PathBuf};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut path = String::new();
    if args.len() == 1 {
        path = String::from(".");
    } else {
        path = String::from(&args[1]);
    }
    let dir_list = get_pathlist(&path)?;

    for i in dir_list {
        let filename = i.file_name().unwrap().to_string_lossy();
        let size: u64 = i.metadata().unwrap().size().into();
        let size = Byte::from_u64(size).get_appropriate_unit(byte_unit::UnitType::Binary);
        let size = format!("{size:.2}");
        let width = 10;
        println!(" {:<width$} {}", filename, size);
    }
    Ok(())
}

// 获取指定目录的子条目数组
fn get_pathlist(path: &str) -> Result<Vec<PathBuf>> {
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
