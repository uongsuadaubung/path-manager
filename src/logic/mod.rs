use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathScope {
    User,
    System,
}

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

pub fn add_path(scope: PathScope, new_path: String) -> Result<bool> {
    let mut paths = read_current_paths(scope)?;
    if paths.iter().any(|p| is_same_path(p, &new_path)) {
        return Ok(false);
    }
    paths.push(new_path);
    write_paths(scope, paths)?;
    Ok(true)
}

pub fn remove_path(scope: PathScope, index: usize) -> Result<()> {
    let mut paths = read_current_paths(scope)?;
    if index == 0 || index > paths.len() {
        return Err(anyhow::anyhow!("Số thứ tự không hợp lệ."));
    }
    paths.remove(index - 1);
    write_paths(scope, paths)
}

pub fn set_path(scope: PathScope, index: usize, new_path: String) -> Result<String> {
    let mut paths = read_current_paths(scope)?;
    if index == 0 || index > paths.len() {
        return Err(anyhow::anyhow!("Số thứ tự không hợp lệ."));
    }
    let old = std::mem::replace(&mut paths[index - 1], new_path);
    write_paths(scope, paths)?;
    Ok(old)
}

pub fn dedupe_paths(scope: PathScope) -> Result<usize> {
    let paths = read_current_paths(scope)?;
    let mut unique = Vec::new();
    let mut count = 0;

    let system_paths = if scope == PathScope::User {
        read_current_paths(PathScope::System).unwrap_or_default()
    } else {
        Vec::new()
    };

    for p in paths {
        let is_internal_duplicate = unique.iter().any(|existing: &String| is_same_path(existing, &p));
        let is_system_duplicate = system_paths.iter().any(|sp| is_same_path(sp, &p));

        if !is_internal_duplicate && !is_system_duplicate {
            unique.push(p);
        } else {
            count += 1;
        }
    }

    if count > 0 {
        write_paths(scope, unique)?;
    }
    Ok(count)
}

pub fn merge_paths(scope: PathScope, imported_paths: Vec<String>) -> Result<usize> {
    let mut current = read_current_paths(scope)?;
    let mut count = 0;
    for p in imported_paths {
        if !current.iter().any(|existing| is_same_path(existing, &p)) {
            current.push(p);
            count += 1;
        }
    }
    if count > 0 {
        write_paths(scope, current)?;
    }
    Ok(count)
}
