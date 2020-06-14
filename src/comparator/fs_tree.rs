use std::collections::{BTreeMap, HashMap};
use crate::comparator::comparison::Comparison;
use std::path::PathBuf;
use std::cell::RefCell;
use std::ops::DerefMut;

const LI_FILE: &str = r#"<li>
                             <table>
                                 <tr class="${class}">
                                     <td>${file-info-a}</td>
                                     <td>${file-info-b}</td>
                                 </tr>
                             </table>
                         </li>"#;

const LI_DIR: &str = r#"<li class="collapse">
                            <input type="checkbox" id="${id}"/>
                            <label for="${id}">
                                ${directory-info}
                            </label>
                            <ul>
                                ${sub-nodes}
                            </ul>
                        </li>"#;

/// Defines root as the empty string
const ROOT_KEY: &str = "";

enum Node<'a> {
    Root,
    Node(&'a Comparison<'a>)
}

pub fn make_tree(comparisons: BTreeMap<String, Comparison<'_>>) {

    // virtual path -> nodeid
    let mut node_lookup: HashMap<String, indextree::NodeId> = HashMap::new();

    let mut arena: indextree::Arena<Node> = indextree::Arena::new();
    let root_id = arena.new_node(Node::Root);

    node_lookup.insert(String::from(ROOT_KEY), root_id);

    let arena_refcell = RefCell::new(arena);
    let node_lookup_refcell = RefCell::new(node_lookup);

    for (_, comparison) in &comparisons {
        recursive_insert(&arena_refcell, comparison, &comparisons, &node_lookup_refcell);
    }
}

fn recursive_insert<'a> (
        arena_refcell: &'a RefCell<indextree::Arena<Node<'a>>>,
        comparison: &'a Comparison<'a>,
        comparison_lookup: &'a BTreeMap<String, Comparison<'a>>,
        node_lookup_refcell: &'a RefCell<HashMap<String, indextree::NodeId>>) {

    let mut vpath = comparison.virtual_path();
    //let file_name = vpath.file_name().unwrap();
    vpath.pop();

    let vpath_parent = String::from(vpath.to_str().unwrap());


    if let None = node_lookup_refcell.borrow().get(&vpath_parent) {
        recursive_insert(arena_refcell, comparison_lookup.get(&vpath_parent).unwrap(), comparison_lookup, node_lookup_refcell);
    }

    if let Some(parent_node_id) = node_lookup_refcell.borrow().get(&vpath_parent) {
        let new_node_id = arena_refcell.borrow_mut().new_node(Node::Node(comparison));
        node_lookup_refcell.borrow_mut().insert(comparison.virtual_path_clone(), new_node_id.clone());
        parent_node_id.append(new_node_id, arena_refcell.borrow_mut().deref_mut());
    } else {
        unreachable!("recursive_insert: there was no parent node")
    }
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