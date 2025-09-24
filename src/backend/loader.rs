use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use super::project::{Node, NodeKind};

/// Build a tree from disk with sane limits.
/// - `max_depth`: stop recursing after this depth
/// - `max_entries`: global cap on visited entries
pub fn load_tree_from(root: &Path, max_depth: usize, max_entries: usize) -> Result<Node> {
    let mut counter = 0usize;
    walk(root, 0, max_depth, &mut counter, max_entries)
}

fn walk(
    path: &Path,
    depth: usize,
    max_depth: usize,
    counter: &mut usize,
    max_entries: usize,
) -> Result<Node> {
    if *counter >= max_entries {
        return Ok(Node {
            id: norm(path),
            name: format!("{} (truncated)", last_name(path)),
            kind: NodeKind::Folder,
            children: vec![],
        });
    }

    let meta = fs::symlink_metadata(path)?;
    let name = last_name(path);
    if meta.is_dir() {
        let mut children = Vec::new();
        if depth < max_depth {
            let entries = match fs::read_dir(path) {
                Ok(rd) => rd,
                Err(_) => {
                    return Ok(Node {
                        id: norm(path),
                        name,
                        kind: NodeKind::Folder,
                        children,
                    });
                }
            };
            for entry in entries {
                if *counter >= max_entries {
                    break;
                }
                if let Ok(entry) = entry {
                    let p: PathBuf = entry.path();
                    if let Some(fname) = p.file_name().and_then(|s| s.to_str()) {
                        if fname.starts_with('.') {
                            continue; // skip hidden
                        }
                    }
                    *counter += 1;
                    if let Ok(node) = walk(&p, depth + 1, max_depth, counter, max_entries) {
                        children.push(node);
                    }
                }
            }
            // Sort: folders first, then files; then case-insensitive name
            children.sort_by(|a, b| {
                use std::cmp::Ordering::*;
                match (&a.kind, &b.kind) {
                    (NodeKind::Folder, NodeKind::File) => Less,
                    (NodeKind::File, NodeKind::Folder) => Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            });
        }
        Ok(Node {
            id: norm(path),
            name,
            kind: NodeKind::Folder,
            children,
        })
    } else {
        Ok(Node {
            id: norm(path),
            name,
            kind: NodeKind::File,
            children: vec![],
        })
    }
}

fn last_name(path: &Path) -> String {
    // Return an owned String (no borrowing of a temporary)
    match path.file_name() {
        Some(os) => os.to_string_lossy().to_string(),
        None => path.display().to_string(),
    }
}

fn norm(path: &Path) -> String {
    // Path as forward-slash string (nicer in UI)
    path.to_string_lossy().replace('\\', "/")
}
