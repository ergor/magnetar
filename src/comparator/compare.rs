use crate::comparator::comparison::Comparison;
use crate::comparator::comparison;
use crate::db_models::fs_node::{FsNode, NodeType};
use std::collections::{HashMap, BTreeMap, BTreeSet, HashSet};
use std::fs::read;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use crate::comparator::virtual_fs_node::VirtualFsNode;

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
pub fn make_pool(fs_nodes: &Vec<FsNode>, roots: Vec<String>) -> Result<Vec<VirtualFsNode>, &'static str> {
    #[cfg(feature = "verbose")]
    println!("make_pool: start...");

    let relevant: Vec<(String, &FsNode)> = filter_by_roots(fs_nodes, roots);

    // TODO: sort
    let virtual_nodes: Vec<VirtualFsNode> = relevant.into_iter()
        .map(|tuple| VirtualFsNode::from(tuple))
        .collect();

    let mut virtual_paths: HashSet<String> = HashSet::new();

    for virtual_node in &virtual_nodes {

        #[cfg(feature = "verbose")]
        println!("{:?}", virtual_node);

        if virtual_paths.contains(&virtual_node.virtual_path) {

            #[cfg(feature = "verbose")]
            println!("make_pool: Err");

            return Err("compare: make_pool: duplicate virtual path for the given roots.");
        }
        virtual_paths.insert(virtual_node.virtual_path.clone());
    }

    #[cfg(feature = "verbose")]
    println!("make_pool: Ok");

    Ok(virtual_nodes)
}

/// for each pool, the virtual path must be unique.
pub fn compare(pool_a: Vec<VirtualFsNode>, pool_b: Vec<VirtualFsNode>) {
    let html = include_str!("report.html");
    let generated = html.replace("${tree-nodes}", "");

    // let mut tree: indextree::Arena<Comparison> = indextree::Arena::new();
    //
    // // TODO: sort by parent_path before inserting here
    // let mut parents_a: BTreeMap<String, Vec<&FsNode>> = BTreeMap::new();
    //
    // // TODO: sort by parent_path before inserting here
    // let mut parents_b: BTreeMap<String, Vec<&FsNode>> = BTreeMap::new();

    //populate(&mut tree, &a, true);
    //populate(&mut tree, &b, false);

    //println!("{}", generated);
}

// fn wrap_fs_nodes(fs_nodes: &Vec<FsNode>) -> Vec<VirtualFsNode> {
//     let mut wrapped_nodes = Vec::new();
//
//     for fs_node in fs_nodes {
//         let mut full_path = PathBuf::from(&fs_node.parent_path);
//         full_path.push(&fs_node.name);
//         wrapped_nodes.push(VirtualFsNode {
//             full_path: String::from(full_path.to_str().unwrap()),
//             fs_node
//         });
//     }
//
//     return wrapped_nodes;
// }

// fn hashmap<'a>(wrapped_nodes: &'a Vec<VirtualFsNode>) -> HashMap<String, &'a VirtualFsNode<'a>> {
//     let mut map: HashMap<String, &'a VirtualFsNode<'a>> = HashMap::new();
//
//     for wrapped_node in wrapped_nodes {
//         map.insert(wrapped_node.full_path.clone(), wrapped_node);
//     }
//
//     return map;
// }

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

// fn populate<'a>(tree: &mut HashMap<String, Vec<Comparison<'a>>>, fs_nodes: &'a Vec<FsNode>, is_a: bool) {
//
//     fn set(comparison: &mut Comparison, fs_node: &FsNode) {
//         if is_a {
//             comparison.set_a(fs_node);
//         } else {
//             comparison.set_b(fs_node);
//         }
//     }
//
//     for fs_node in fs_nodes {
//         let parent_path = &fs_node.parent_path;
//         let entry = tree.get_mut(parent_path);
//         match entry {
//             None => {
//                 let mut comparison = Comparison::new();
//                 set(&mut comparison, fs_node);
//
//                 let mut comparisons = Vec::new();
//                 comparisons.push(comparison);
//
//                 tree.insert(parent_path.clone(), comparisons);
//             },
//             Some(comparisons) => {
//                 comparisons.push(fs_node);
//             }
//         }
//     }
// }

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