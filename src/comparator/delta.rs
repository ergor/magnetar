use crate::db_models::fs_node::{FsNode, NodeType};
use std::panic::resume_unwind;
use std::path::PathBuf;
use crate::comparator::virtual_fs_node::VirtualFsNode;
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use chrono::TimeZone;

#[derive(Debug)]
pub struct Delta<'a> {
    delta_type: DeltaType,
    field_delta_types: HashSet<FieldDelta>,
    a: Option<VirtualFsNode<'a>>,
    b: Option<VirtualFsNode<'a>>,
}

#[derive(Eq, PartialEq, Debug)]
pub enum DeltaType {
    Creation,
    Deletion,
    Modification(Vec<String>),
    NoChange
}

impl DeltaType {
    pub fn css_class(&self) -> &'static str {
        match &self {
            DeltaType::Creation => "creation",
            DeltaType::Deletion => "deletion",
            DeltaType::Modification(_) => "modification",
            DeltaType::NoChange => "no-change",
        }
    }
}

/// Indicates what field in the FsNodes was changed.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum FieldDelta {
    NodeType,
    Checksum,
    Size,
    User,
    Group,
    Permissions,
    CreationDate,
    ModifiedDate,
    LinksTo,
    Inode,
    NLinks
}

impl FieldDelta {
    pub fn all() -> HashSet<FieldDelta> {
        let mut set = HashSet::new();
        set.insert(FieldDelta::NodeType);
        set.insert(FieldDelta::Checksum);
        set.insert(FieldDelta::Size);
        set.insert(FieldDelta::User);
        set.insert(FieldDelta::Group);
        set.insert(FieldDelta::Permissions);
        set.insert(FieldDelta::CreationDate);
        set.insert(FieldDelta::ModifiedDate);
        set.insert(FieldDelta::LinksTo);
        set.insert(FieldDelta::Inode);
        set.insert(FieldDelta::NLinks);
        set
    }
}

impl<'a> Delta<'a> {

    /// ### params
    /// `field_delta_types`: what field changes shall count as a `DeltaType::Modification`
    pub fn new(a: Option<VirtualFsNode<'a>>, b: Option<VirtualFsNode<'a>>, field_delta_types: &HashSet<FieldDelta>) -> Delta<'a> {
        let mut comparison = Delta {
            delta_type: DeltaType::NoChange,
            field_delta_types: field_delta_types.clone(),
            a,
            b,
        };
        comparison.delta_type = comparison.delta_type();
        return comparison;
    }

    /// ### params
    /// `field_delta_types`: what field changes shall count as a `DeltaType::Modification`
    pub fn delta_type(&self) -> DeltaType {
        if self.a.is_none() && self.b.is_some() {
            return DeltaType::Creation;
        }
        else if self.a.is_some() && self.b.is_none() {
            return DeltaType::Deletion;
        }
        else if let (Some(a), Some(b)) = (&self.a, &self.b) {
            let field_deltas: Vec<String> = self.field_deltas();
            return
                if field_deltas.is_empty() {
                    DeltaType::NoChange
                } else {
                    DeltaType::Modification(field_deltas)
                };
        }
        unreachable!("comparison: delta_type exhausted");
    }

    pub fn root_path_str(&self) -> &str {
        if let Some(vnode) = &self.a {
            return vnode.root.as_str();
        }
        if let Some(vnode) = &self.b {
            return vnode.root.as_str();
        }
        unreachable!("both vfsnodes were None");
    }

    pub fn virtual_path_str(&self) -> &str {
        if let Some(vnode) = &self.a {
            return vnode.virtual_path.as_str();
        }
        if let Some(vnode) = &self.b {
            return vnode.virtual_path.as_str();
        }
        unreachable!("both vfsnodes were None");
    }

    pub fn file_type(&self) -> &str {
        fn to_symbol(node_type: &NodeType) -> &str {
            match node_type {
                NodeType::File => {""},
                NodeType::Directory => {"D"},
                NodeType::Symlink => {"L"},
                NodeType::Other => {"O"},
            }
        }
        match self.delta_type {
            DeltaType::Creation => { to_symbol(&self.b.as_ref().unwrap().fs_node.node_type) },
            DeltaType::Deletion => { to_symbol(&self.a.as_ref().unwrap().fs_node.node_type) },
            DeltaType::Modification(_) => { to_symbol(&self.b.as_ref().unwrap().fs_node.node_type) },
            DeltaType::NoChange => { to_symbol(&self.a.as_ref().unwrap().fs_node.node_type) },
        }
    }

    pub fn delta_info(&self) -> String {
        match &self.delta_type {
            DeltaType::NoChange => { String::new() },
            DeltaType::Creation => { format!("[created]") },
            DeltaType::Deletion => { format!("[deleted]") },
            DeltaType::Modification(changes) => {
                String::from(changes.join(", "))
            },
        }
    }

    pub fn field_deltas(&self) -> Vec<String> {
        let mut deltas = Vec::new();

        let aaa = &self.a.as_ref().expect("field_deltas must never be called on a creation or deletion delta").fs_node;
        let bbb = &self.b.as_ref().expect("field_deltas must never be called on a creation or deletion delta").fs_node;

        // TODO: this is kinda ugly
        if self.field_delta_types.contains(&FieldDelta::Size) && aaa.size != bbb.size {
            deltas.push(format!("size: {} -> {}", aaa.size, bbb.size));
        }
        if self.field_delta_types.contains(&FieldDelta::NodeType) && aaa.node_type != bbb.node_type {
            deltas.push(format!("type: {} -> {}", aaa.node_type, bbb.node_type));
        }
        if self.field_delta_types.contains(&FieldDelta::User) && aaa.uid != bbb.uid {
            deltas.push(format!("uid: {} -> {}", aaa.uid, bbb.uid));
        }
        if self.field_delta_types.contains(&FieldDelta::Group) && aaa.gid != bbb.gid {
            deltas.push(format!("gid: {} -> {}", aaa.gid, bbb.gid));
        }
        if self.field_delta_types.contains(&FieldDelta::Permissions) && aaa.permissions != bbb.permissions {
            deltas.push(format!("perms: {} -> {}", aaa.permissions, bbb.permissions));
        }
        if self.field_delta_types.contains(&FieldDelta::CreationDate) && aaa.creation_date != bbb.creation_date {
            let time_a = chrono::Local.timestamp(aaa.creation_date, 0);
            let time_b = chrono::Local.timestamp(bbb.creation_date, 0);
            deltas.push(format!("date created: {} -> {}", time_a.to_string(), time_b.to_string()));
        }
        if self.field_delta_types.contains(&FieldDelta::ModifiedDate) && aaa.modified_date != bbb.modified_date {
            let time_a = chrono::Local.timestamp(aaa.modified_date, 0);
            let time_b = chrono::Local.timestamp(bbb.modified_date, 0);
            deltas.push(format!("date modified: {} -> {}", time_a.to_string(), time_b.to_string()));
        }
        if self.field_delta_types.contains(&FieldDelta::LinksTo) && aaa.links_to != bbb.links_to {
            deltas.push(format!("symlink to: {} -> {}", aaa.links_to, bbb.links_to));
        }
        if self.field_delta_types.contains(&FieldDelta::Checksum) && aaa.sha1_checksum != bbb.sha1_checksum {
            deltas.push(format!("sha1: {} -> {}", aaa.sha1_checksum, bbb.sha1_checksum));
        }
        if self.field_delta_types.contains(&FieldDelta::Inode) && aaa.inode != bbb.inode {
            deltas.push(format!("inode: {} -> {}", aaa.inode, bbb.inode));
        }
        if self.field_delta_types.contains(&FieldDelta::NLinks) && aaa.nlinks != bbb.nlinks {
            deltas.push(format!("hardlink count: {} -> {}", aaa.nlinks, bbb.nlinks));
        }

        deltas
    }
}