use crate::apperror::AppError;
use crate::comparator::delta::{Delta, Attribute};
use crate::comparator::virtual_fs_node::VirtualFsNode;
use crate::db_models::fs_node::FsNode;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::iter::FromIterator;


pub type VFsNodeMap<'a> = BTreeMap<String, VirtualFsNode<'a>>;
pub type DeltaMap<'a> = BTreeMap<String, Delta<'a>>;

/// Creates a pool where the virtual nodes are sorted by path.
pub fn make_pool(fs_nodes: &Vec<FsNode>, roots: Vec<String>) -> Result<VFsNodeMap<'_>, AppError> {

    log::debug!("make_pool: start...");

    let relevant: Vec<(String, &FsNode)> = filter_by_roots(fs_nodes, roots);
    log::debug!("FsNodes supplied: {}. FsNodes in given roots: {}", fs_nodes.len(), relevant.len());

    let virtual_nodes: Vec<VirtualFsNode<'_>> = relevant.into_iter()
        .map(|tuple| VirtualFsNode::from(tuple))
        .collect();

    // BTreeMap because the implicit ordering by the key (i.e. the virtual path) is important!
    let mut v_node_map: VFsNodeMap<'_> = BTreeMap::new();

    log::debug!("validating virtual paths...");
    for virtual_node in virtual_nodes {
        let v_path = &virtual_node.virtual_path;
        if v_node_map.contains_key(v_path) {
            let error = AppError::WithMessage(format!("duplicate virtual path for the given roots: '{}'", v_path));
            log::error!("error: {}", error);
            return Err(error);
        }
        v_node_map.insert(v_path.clone(), virtual_node);
    }
    log::debug!("all virtual paths OK.");

    v_node_map.iter()
        .for_each(|(_, v_node)| log::trace!("{:?}", v_node));

    log::debug!("pool created with {} virtual paths", v_node_map.len());

    Ok(v_node_map)
}

/// for each pool, the virtual path must be unique.
/// **pool_a** is defined as the old index, and **pool_b** is the new.
pub fn compare<'a>(mut pool_a: VFsNodeMap<'a>, mut pool_b: VFsNodeMap<'a>, attr_types: &HashSet<Attribute>) -> Vec<Delta<'a>> {

    let v_paths_a_set: BTreeSet<String> = BTreeSet::from_iter(pool_a.keys().cloned());
    let v_paths_b_set: BTreeSet<String> = BTreeSet::from_iter(pool_b.keys().cloned());


    let mut deletions: DeltaMap<'_> = BTreeMap::from_iter(
        v_paths_a_set.difference(&v_paths_b_set)
        .map(|v_path| (v_path.clone(), Delta::new(Some(pool_a.remove(v_path).unwrap()), None, attr_types)))
    );

    log::debug!("compare: found {} deletions", deletions.len());

    let mut creations: DeltaMap<'_> = BTreeMap::from_iter(
        v_paths_b_set.difference(&v_paths_a_set)
        .map(|v_path| (v_path.clone(), Delta::new(None, Some(pool_b.remove(v_path).unwrap()), attr_types)))
    );

    log::debug!("compare: found {} creations", creations.len());

    // the intersection contains both modified and unmodified files
    let mut intersection: DeltaMap<'_> = BTreeMap::from_iter(
        v_paths_a_set.intersection(&v_paths_b_set)
        .map(|v_path| (v_path.clone(), Delta::new(Some(pool_a.remove(v_path).unwrap()), Some(pool_b.remove(v_path).unwrap()), attr_types)))
    );

    log::debug!("compare: found {} intersections", intersection.len());

    let union: BTreeSet<String> = v_paths_a_set.union(&v_paths_b_set).cloned().collect();

    assert_eq!(union.len(), deletions.len() + creations.len() + intersection.len());

    let mut result: Vec<Delta<'a>> = Vec::new();

    for v_path in union {
        if let Some(delta) = deletions.remove(&v_path) {
            result.push(delta);
        }
        else if let Some(delta) = creations.remove(&v_path) {
            result.push(delta);
        }
        else if let Some(delta) = intersection.remove(&v_path) {
            result.push(delta);
        }
        else {
            unreachable!("compare.rs: compare: if-block exhausted");
        }
    }

    result.iter()
        .for_each(|cmp| log::trace!("{:?}", cmp));

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
