
use crate::db_models::fs_node::FsNode;
use std::path::PathBuf;

/// A "virtual" FsNode is a wrapper that adds the concept of a _root path_.
/// When comparing files, we add them to a virtual pool, and the root path
/// specifies what folder we want to source files into the pool from.
#[derive(Debug)]
pub struct VirtualFsNode<'fsnode> {

    /// The root of this virtual node; a substring of the full path.
    pub root: String,

    /// The virtual path is defined as the full path, minus the root.
    pub virtual_path: String,

    ///
    pub fs_node: &'fsnode FsNode,
}

impl<'a> From<(String, &'a FsNode)> for VirtualFsNode<'a> {
    fn from(tuple: (String, &'a FsNode)) -> Self {
        let (root_string, fs_node) = tuple;

        let mut full_path = PathBuf::from(&fs_node.parent_path);
        full_path.push(&fs_node.name);

        let root = PathBuf::from(&root_string);
        let mut virtual_path = full_path.clone();
        let virtual_path = virtual_path.strip_prefix(&root)
            .expect("root path was not prefix of the full path");

        VirtualFsNode {
            fs_node,
            root: root_string,
            virtual_path: String::from(virtual_path.to_str().unwrap())
        }
    }
}