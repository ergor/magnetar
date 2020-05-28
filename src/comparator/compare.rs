use crate::comparator::comparison::Comparison;
use crate::comparator::comparison;
use crate::db_models::fs_node::{FsNode, NodeType};
use std::collections::{HashMap, BTreeMap, BTreeSet};
use std::fs::read;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

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

struct NodeWrapper<'fsnode> {
    full_path: String,
    fs_node: &'fsnode FsNode
}

pub fn compare(a: Vec<FsNode>, b: Vec<FsNode>, roots_a: Vec<String>, roots_b: Vec<String>) {
    let html = include_str!("report.html");
    let generated = html.replace("${tree-nodes}", "");

    let relevant_a = filter_by_roots(a, roots_a);
    let relevant_b = filter_by_roots(b, roots_b);

    let a_wrapped = wrap_fs_nodes(&relevant_a);
    let b_wrapped = wrap_fs_nodes(&relevant_b);

    let a_map = hashmap(&a_wrapped);
    let b_map = hashmap(&b_wrapped);



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

fn wrap_fs_nodes<'a>(fs_nodes: &'a Vec<FsNode>) -> Vec<NodeWrapper<'a>> {
    let mut wrapped_nodes = Vec::new();

    for fs_node in fs_nodes {
        let mut full_path = PathBuf::from(&fs_node.parent_path);
        full_path.push(&fs_node.name);
        wrapped_nodes.push(NodeWrapper {
            full_path: String::from(full_path.to_str().unwrap()),
            fs_node
        });
    }

    return wrapped_nodes;
}

fn hashmap<'a>(wrapped_nodes: &'a Vec<NodeWrapper>) -> HashMap<String, &'a NodeWrapper<'a>> {
    let mut map: HashMap<String, &'a NodeWrapper<'a>> = HashMap::new();

    for wrapped_node in wrapped_nodes {
        map.insert(wrapped_node.full_path.clone(), wrapped_node);
    }

    return map;
}

fn filter_by_roots(fs_nodes: Vec<FsNode>, roots: Vec<String>) -> Vec<FsNode> {
    let roots = BTreeSet::from_iter(roots.iter().cloned());
    let children_in_root: Vec<FsNode> = fs_nodes.into_iter()
        .filter(|fs_node| is_child_of(fs_node, &roots))
        .collect();

    return children_in_root;
}

fn is_child_of(fs_node: &FsNode, parents: &BTreeSet<String>) -> bool {
    for parent in parents {
        if fs_node.parent_path.starts_with(parent) {
            return true;
        }
    }
    return false;
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