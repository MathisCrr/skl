use crate::types::SklError;
use std::{fs, path::{Path, PathBuf}};

pub fn normalize_repo_id(input: &str) -> String {
    if input.starts_with("https://") || input.starts_with("http://") {
        let parts: Vec<&str> = input
            .trim_end_matches('/')
            .trim_end_matches(".git")
            .trim_end_matches('/')
            .split('/')
            .collect();
        format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        input.to_string()
    }
}

pub fn find_files(dir: &Path, filename: &str) -> Result<Vec<PathBuf>, SklError> {
    let mut files = Vec::new();

    if !dir.is_dir() {
        return Ok(files);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if path.join(filename).exists() {
                files.push(path);
            } else {
                files.extend(find_files(&path, filename)?);
            }
        }
    }

    Ok(files)
}

pub fn copy_dir(src: &Path, dest: &Path) -> Result<(), SklError> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}
