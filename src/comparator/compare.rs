use crate::comparator::comparison::Comparison;
use crate::comparator::comparison;
use crate::db_models::fs_node::{FsNode, NodeType};
use std::collections::{HashMap, BTreeMap};

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

pub fn compare(a: Vec<FsNode>, b: Vec<FsNode>) {
    let html = include_str!("report.html");
    let generated = html.replace("${tree-nodes}", "");


    let mut tree: indextree::Arena<Comparison> = indextree::Arena::new();

    // TODO: sort by parent_path before inserting here
    let mut parents_a: BTreeMap<String, Vec<&FsNode>> = BTreeMap::new();

    // TODO: sort by parent_path before inserting here
    let mut parents_b: BTreeMap<String, Vec<&FsNode>> = BTreeMap::new();

    //populate(&mut tree, &a, true);
    //populate(&mut tree, &b, false);

    //println!("{}", generated);
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