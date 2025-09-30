use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

use super::project::{Node, NodeKind};

pub fn load_tree_from(root: &Path, max_depth: usize, max_nodes: usize) -> Result<Node> {
    let root = root.canonicalize()?;
    let mut count = 0;
    let node = load_dir(&root, 0, max_depth, max_nodes, &mut count)?;
    Ok(node)
}

fn load_dir(
    dir: &Path,
    depth: usize,
    max_depth: usize,
    max_nodes: usize,
    count: &mut usize,
) -> Result<Node> {
    *count += 1;
    if *count > max_nodes {
        return Ok(make_leaf(dir));
    }

    let mut children: Vec<Node> = Vec::new();
    if depth < max_depth {
        let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        entries.sort();

        for p in entries {
            if p.is_dir() {
                children.push(load_dir(&p, depth + 1, max_depth, max_nodes, count)?);
            } else {
                let ext = p
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                if ext.eq_ignore_ascii_case("map") {
                    continue; // legacy format hidden from explorer
                }

                let id = p
                    .canonicalize()
                    .unwrap_or(p.clone())
                    .to_string_lossy()
                    .to_string();

                let name = p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                children.push(Node {
                    id,
                    name,
                    kind: NodeKind::File {
                        path: p.to_string_lossy().to_string(),
                        ext,
                    },
                });
            }
        }
    }

    // 🔧 Make folder ID canonical too (stability for open_folders keys)
    let id = dir
        .canonicalize()
        .unwrap_or(dir.to_path_buf())
        .to_string_lossy()
        .to_string();

    let name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(dir.to_string_lossy().as_ref())
        .to_string();

    Ok(Node {
        id,
        name,
        kind: NodeKind::Folder { children },
    })
}

fn make_leaf(path: &Path) -> Node {
    let id = path
        .canonicalize()
        .unwrap_or(path.to_path_buf())
        .to_string_lossy()
        .to_string();
    Node {
        id,
        name: path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string(),
        kind: NodeKind::Folder { children: vec![] },
    }
}
