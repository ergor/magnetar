use crate::comparator::comparison::{Comparison, ChangeType};
use crate::comparator::comparison;
use crate::db_models::fs_node::{FsNode, NodeType};
use std::collections::{HashMap, BTreeMap, BTreeSet, HashSet};
use std::fs::read;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use crate::comparator::virtual_fs_node::VirtualFsNode;
use crate::error::AppError;

const LI_FILE: &str = r#"<li>
                             <table>
                                 <tr class="${class}">
                                     <td>${file-info-a}</td>
                                     <td>${file-info-b}</td>
                                 </tr>
                             </table>
                         </li>"#;

const LI_DIR: &str = r#"<li class="collapse">
                            <input type="checkbox" id="m${id}"/>
                            <label for="m${id}">
                                ${directory-info}
                            </label>
                            <ul>
                                ${sub-nodes}
                            </ul>
                        </li>"#;


/// Creates a pool where the virtual nodes are sorted by path.
pub fn make_pool(fs_nodes: &Vec<FsNode>, roots: Vec<String>) -> Result<BTreeMap<String, VirtualFsNode>, AppError> {
    #[cfg(feature = "verbose")]
    println!("make_pool: start...");

    let relevant: Vec<(String, &FsNode)> = filter_by_roots(fs_nodes, roots);

    let virtual_nodes: Vec<VirtualFsNode> = relevant.into_iter()
        .map(|tuple| VirtualFsNode::from(tuple))
        .collect();

    // BTreeMap because the implicit ordering by the key (i.e. the virtual path) is important!
    let mut v_node_map: BTreeMap<String, VirtualFsNode> = BTreeMap::new();

    for virtual_node in virtual_nodes {
        if v_node_map.contains_key(&virtual_node.virtual_path) {
            #[cfg(feature = "verbose")]
            println!("make_pool: Err");

            return Err(AppError::WithMessage("compare.rs: make_pool: duplicate virtual path for the given roots."));
        }
        v_node_map.insert(virtual_node.virtual_path.clone(), virtual_node);
    }

    #[cfg(feature = "verbose")]
    v_node_map.iter()
        .for_each(|n| println!("{:?}", n.1));

    #[cfg(feature = "verbose")]
    println!("make_pool: Ok");

    Ok(v_node_map)
}

/// for each pool, the virtual path must be unique.
/// **pool_a** is defined as the old index, and **pool_b** is the new.
pub fn compare<'a>(pool_a: BTreeMap<String, VirtualFsNode<'a>>, pool_b: BTreeMap<String, VirtualFsNode<'a>>) -> Vec<Comparison<'a>> {
    let html = include_str!("report.html");
    let generated = html.replace("${tree-nodes}", "");

    let v_paths_a: BTreeSet<String> = BTreeSet::from_iter(pool_a.keys().cloned());
    let v_paths_b: BTreeSet<String> = BTreeSet::from_iter(pool_b.keys().cloned());


    let mut deletions: BTreeMap<String, Comparison> = BTreeMap::from_iter(
        v_paths_a.difference(&v_paths_b)
        .map(|v_path| (v_path.clone(), Comparison::new(Some(pool_a.get(v_path).unwrap().fs_node), None, v_path.clone())))
    );

    let mut creations: BTreeMap<String, Comparison> = BTreeMap::from_iter(
        v_paths_b.difference(&v_paths_a)
        .map(|v_path| (v_path.clone(), Comparison::new(None, Some(pool_b.get(v_path).unwrap().fs_node), v_path.clone())))
    );

    // TODO: accept args for what shall count as a change
    // the intersection contains both modified and unmodified files
    let mut intersection: BTreeMap<String, Comparison> = BTreeMap::from_iter(
        v_paths_a.intersection(&v_paths_b)
        .map(|v_path| (v_path.clone(), Comparison::new(Some(pool_a.get(v_path).unwrap().fs_node), Some(pool_b.get(v_path).unwrap().fs_node), v_path.clone())))
    );

    let union: BTreeSet<String> = v_paths_a.union(&v_paths_b).cloned().collect();

    assert_eq!(union.len(), deletions.len() + creations.len() + intersection.len());

    let mut result = Vec::new();

    for v_path in union {
        if let Some(comparison) = deletions.remove(&v_path) {
            result.push(comparison);
        }
        else if let Some(comparison) = creations.remove(&v_path) {
            result.push(comparison);
        }
        else if let Some(comparison) = intersection.remove(&v_path) {
            result.push(comparison);
        }
        else {
            unreachable!("compare.rs: compare: if-block exhausted");
        }
    }

    #[cfg(feature = "verbose")]
    result.iter()
        .for_each(|cmp| println!("{:?}", cmp));

    result
}

fn filter_by_roots(fs_nodes: &Vec<FsNode>, roots: Vec<String>) -> Vec<(String, &FsNode)> {
    let roots = BTreeSet::from_iter(roots.iter().cloned());
    let children_in_root: Vec<(String, &FsNode)> = fs_nodes.into_iter()
        .filter_map(|fs_node| find_root(fs_node, &roots))
        .collect();

    return children_in_root;
}

/// Finds what root the supplied FsNode is in, and wraps it
/// in a an optional tuple (root, fsnode). `filter_map` friendly.
fn find_root<'a, 'b>(fs_node: &'a FsNode, parents: &'b BTreeSet<String>) -> Option<(String, &'a FsNode)> {
    for parent in parents {
        if fs_node.parent_path.starts_with(parent) {
            return Some((parent.clone(), fs_node));
        }
    }
    return None;
}

fn file(fs_node: FsNode) -> String {
    let file_info = format!("{}", fs_node.name);
    LI_FILE.replace("${file-info}", file_info.as_str())
}

fn dir(fs_node: FsNode, children: &[FsNode]) -> String {
    for child in children {
        if let NodeType::Directory = child.node_type {

        }
    }
    let dir_info = format!("{}", fs_node.name);
    LI_DIR.replace("${directory-info}", dir_info.as_str())
}