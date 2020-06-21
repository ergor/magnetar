use crate::comparator::virtual_fs_node::VirtualFsNode;
use crate::db_models::fs_node::NodeType;
use chrono::TimeZone;
use std::collections::HashSet;
use crate::apperror::AppError;
use crate::util::unix_perms::Permission;

#[derive(Debug)]
pub struct Delta<'a> {
    delta_type: DeltaType,
    delta_trigger_attrs: HashSet<Attribute>,
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
pub enum Attribute {
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

impl Attribute {
    pub fn all() -> HashSet<Attribute> {
        let mut set = HashSet::new();
        set.insert(Attribute::NodeType);
        set.insert(Attribute::Checksum);
        set.insert(Attribute::Size);
        set.insert(Attribute::User);
        set.insert(Attribute::Group);
        set.insert(Attribute::Permissions);
        set.insert(Attribute::CreationDate);
        set.insert(Attribute::ModifiedDate);
        set.insert(Attribute::LinksTo);
        set.insert(Attribute::Inode);
        set.insert(Attribute::NLinks);
        set
    }

    pub fn medium() -> HashSet<Attribute> {
        let mut set = HashSet::new();
        set.insert(Attribute::Checksum);
        set.insert(Attribute::Size);
        set.insert(Attribute::User);
        set.insert(Attribute::Group);
        set.insert(Attribute::Permissions);
        set.insert(Attribute::CreationDate);
        set.insert(Attribute::ModifiedDate);
        set
    }

    pub fn minimum() -> HashSet<Attribute> {
        let mut set = HashSet::new();
        set.insert(Attribute::Checksum);
        set.insert(Attribute::Size);
        set.insert(Attribute::ModifiedDate);
        set
    }

    pub fn from_arg(arg: &str) -> Result<HashSet<Attribute>, AppError> {
        let mut set = HashSet::new();
        for c in arg.chars() {
            let attr = Attribute::from_char(c)?;
            set.insert(attr);
        }
        Ok(set)
    }

    fn from_char(c: char) -> Result<Attribute, AppError> {
        match c {
            't' => Ok(Attribute::NodeType),
            'c' => Ok(Attribute::Checksum),
            's' => Ok(Attribute::Size),
            'u' => Ok(Attribute::User),
            'g' => Ok(Attribute::Group),
            'p' => Ok(Attribute::Permissions),
            'b' => Ok(Attribute::CreationDate),
            'm' => Ok(Attribute::ModifiedDate),
            'l' => Ok(Attribute::LinksTo),
            'i' => Ok(Attribute::Inode),
            'n' => Ok(Attribute::NLinks),
            _ => Err(AppError::WithMessage("'{}' is not a valid attribute change option.".to_string()))
        }
    }
}

impl<'a> Delta<'a> {

    /// ### params
    /// `delta_trigger_attrs`: what field changes shall count as a `DeltaType::Modification`
    pub fn new(a: Option<VirtualFsNode<'a>>, b: Option<VirtualFsNode<'a>>, delta_trigger_attrs: &HashSet<Attribute>) -> Delta<'a> {
        let mut comparison = Delta {
            delta_type: DeltaType::NoChange,
            delta_trigger_attrs: delta_trigger_attrs.clone(),
            a,
            b,
        };
        comparison.delta_type = comparison.delta_type();
        return comparison;
    }

    pub fn delta_type(&self) -> DeltaType {
        if self.a.is_none() && self.b.is_some() {
            return DeltaType::Creation;
        }
        else if self.a.is_some() && self.b.is_none() {
            return DeltaType::Deletion;
        }
        else if let (Some(_), Some(_)) = (&self.a, &self.b) {
            let modified_attrs: Vec<String> = self.modifications();
            return
                if modified_attrs.is_empty() {
                    DeltaType::NoChange
                } else {
                    DeltaType::Modification(modified_attrs)
                };
        }
        unreachable!("comparison: delta_type exhausted");
    }

    pub fn root_path_str(&self) -> &str {
        // order is important, we want b first, because b represents current state.
        if let Some(vnode) = &self.b {
            return vnode.root.as_str();
        }
        if let Some(vnode) = &self.a {
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

    pub fn modifications(&self) -> Vec<String> {
        let mut deltas = Vec::new();

        let aaa = &self.a.as_ref().expect("modified_attributes must never be called on a creation or deletion delta").fs_node;
        let bbb = &self.b.as_ref().expect("modified_attributes must never be called on a creation or deletion delta").fs_node;

        // TODO: this is kinda ugly
        if self.delta_trigger_attrs.contains(&Attribute::Size) && aaa.size != bbb.size {
            deltas.push(format!("size: {} -> {}", aaa.size, bbb.size));
        }
        if self.delta_trigger_attrs.contains(&Attribute::NodeType) && aaa.node_type != bbb.node_type {
            deltas.push(format!("type: {} -> {}", aaa.node_type, bbb.node_type));
        }
        if self.delta_trigger_attrs.contains(&Attribute::User) && aaa.uid != bbb.uid {
            deltas.push(format!("uid: {} -> {}", aaa.uid, bbb.uid));
        }
        if self.delta_trigger_attrs.contains(&Attribute::Group) && aaa.gid != bbb.gid {
            deltas.push(format!("gid: {} -> {}", aaa.gid, bbb.gid));
        }
        if self.delta_trigger_attrs.contains(&Attribute::Permissions) && aaa.permissions != bbb.permissions {
            let aaa = Permission::from_val(aaa.permissions);
            let bbb = Permission::from_val(bbb.permissions);
            deltas.push(format!("perms: {} -> {}", aaa.as_str(), bbb.as_str()));
        }
        if self.delta_trigger_attrs.contains(&Attribute::CreationDate) && aaa.creation_date != bbb.creation_date {
            let time_a = chrono::Local.timestamp(aaa.creation_date, 0);
            let time_b = chrono::Local.timestamp(bbb.creation_date, 0);
            deltas.push(format!("date created: {} -> {}", time_a.to_string(), time_b.to_string()));
        }
        if self.delta_trigger_attrs.contains(&Attribute::ModifiedDate) && aaa.modified_date != bbb.modified_date {
            let time_a = chrono::Local.timestamp(aaa.modified_date, 0);
            let time_b = chrono::Local.timestamp(bbb.modified_date, 0);
            deltas.push(format!("date modified: {} -> {}", time_a.to_string(), time_b.to_string()));
        }
        if self.delta_trigger_attrs.contains(&Attribute::LinksTo) && aaa.links_to != bbb.links_to {
            deltas.push(format!("symlink to: {} -> {}", aaa.links_to, bbb.links_to));
        }
        if self.delta_trigger_attrs.contains(&Attribute::Checksum) && aaa.sha1_checksum != bbb.sha1_checksum {
            deltas.push(format!("sha1: {} -> {}", aaa.sha1_checksum, bbb.sha1_checksum));
        }
        if self.delta_trigger_attrs.contains(&Attribute::Inode) && aaa.inode != bbb.inode {
            deltas.push(format!("inode: {} -> {}", aaa.inode, bbb.inode));
        }
        if self.delta_trigger_attrs.contains(&Attribute::NLinks) && aaa.nlinks != bbb.nlinks {
            deltas.push(format!("hardlink count: {} -> {}", aaa.nlinks, bbb.nlinks));
        }

        deltas
    }
}