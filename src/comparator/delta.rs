use crate::db_models::fs_node::FsNode;
use std::panic::resume_unwind;
use std::path::PathBuf;
use crate::comparator::virtual_fs_node::VirtualFsNode;
use std::collections::{HashMap, HashSet};

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
            let field_deltas: Vec<String> = self.field_deltas().into_iter()
                .filter_map(|(fd, s)| if self.field_delta_types.contains(&fd) { Some(s) } else { None })
                .collect();

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

    pub fn field_deltas(&self) -> HashMap<FieldDelta, String> {
        let mut deltas = HashMap::new();

        let aaa = &self.a.as_ref().expect("field_deltas must never be called on a creation or deletion delta").fs_node;
        let bbb = &self.b.as_ref().expect("field_deltas must never be called on a creation or deletion delta").fs_node;

        if aaa.size != bbb.size {
            deltas.insert(FieldDelta::Size, format!("size: {} -> {}", aaa.size, bbb.size));
        }
        if aaa.node_type != bbb.node_type {
            deltas.insert(FieldDelta::NodeType, format!("type: {} -> {}", aaa.node_type, bbb.node_type));
        }
        if aaa.uid != bbb.uid {
            deltas.insert(FieldDelta::User, format!("uid: {} -> {}", aaa.uid, bbb.uid));
        }
        if aaa.gid != bbb.gid {
            deltas.insert(FieldDelta::Group, format!("gid: {} -> {}", aaa.gid, bbb.gid));
        }
        if aaa.permissions != bbb.permissions {
            deltas.insert(FieldDelta::Permissions, format!("perms: {} -> {}", aaa.permissions, bbb.permissions));
        }
        if aaa.sha1_checksum != bbb.sha1_checksum {
            deltas.insert(FieldDelta::Checksum, format!("sha1: {} -> {}", aaa.sha1_checksum, bbb.sha1_checksum));
        }
        if aaa.creation_date != bbb.creation_date {
            deltas.insert(FieldDelta::CreationDate, format!("date created: {} -> {}", aaa.creation_date, bbb.creation_date));
        }
        if aaa.modified_date != bbb.modified_date {
            deltas.insert(FieldDelta::ModifiedDate, format!("date modified: {} -> {}", aaa.modified_date, bbb.modified_date));
        }
        if aaa.links_to != bbb.links_to {
            deltas.insert(FieldDelta::LinksTo, format!("symlink to: {} -> {}", aaa.links_to, bbb.links_to));
        }
        if aaa.inode != bbb.inode {
            deltas.insert(FieldDelta::Inode, format!("inode: {} -> {}", aaa.inode, bbb.inode));
        }
        if aaa.nlinks != bbb.nlinks {
            deltas.insert(FieldDelta::NLinks, format!("hardlink count: {} -> {}", aaa.nlinks, bbb.nlinks));
        }

        deltas
    }
}