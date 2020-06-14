use crate::db_models::fs_node::FsNode;
use std::panic::resume_unwind;
use std::path::PathBuf;
use crate::comparator::virtual_fs_node::VirtualFsNode;

#[derive(Debug)]
pub struct Delta<'a> {
    delta_type: DeltaType,
    a: Option<VirtualFsNode<'a>>,
    b: Option<VirtualFsNode<'a>>,
}

#[derive(Eq, PartialEq, Debug)]
pub enum DeltaType {
    Creation,
    Deletion,
    Modification,
    NoChange
}

impl DeltaType {
    pub fn css_class(change_type: DeltaType) -> &'static str {
        match change_type {
            DeltaType::Creation => "creation",
            DeltaType::Deletion => "deletion",
            DeltaType::Modification => "modification",
            DeltaType::NoChange => "no-change",
        }
    }
}

impl<'a> Delta<'a> {
    pub fn new(a: Option<VirtualFsNode<'a>>, b: Option<VirtualFsNode<'a>>) -> Delta<'a> {
        let mut comparison = Delta {
            delta_type: DeltaType::NoChange,
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
        else if let (Some(a), Some(b)) = (&self.a, &self.b) {
            return
                if a.fs_node.sha1_checksum != b.fs_node.sha1_checksum {
                    DeltaType::Modification
                } else {
                    DeltaType::NoChange
                };
        }
        unreachable!("comparison: delta_type exhausted");
    }
}