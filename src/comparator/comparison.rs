use crate::db_models::fs_node::FsNode;
use std::panic::resume_unwind;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Comparison<'a> {
    change_type: ChangeType,
    virtual_path: String,
    a: Option<&'a FsNode>,
    b: Option<&'a FsNode>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ChangeType {
    Creation,
    Deletion,
    Modification,
    NoChange
}

impl ChangeType {
    pub fn css_class(change_type: ChangeType) -> &'static str {
        match change_type {
            ChangeType::Creation => "creation",
            ChangeType::Deletion => "deletion",
            ChangeType::Modification => "modification",
            ChangeType::NoChange => "no-change",
        }
    }
}

impl<'a> Comparison<'a> {
    pub fn new(a: Option<&'a FsNode>, b: Option<&'a FsNode>, virtual_path: String) -> Comparison<'a> {
        let mut comparison = Comparison {
            change_type: ChangeType::NoChange,
            virtual_path,
            a,
            b,
        };
        comparison.change_type = comparison.change_type();
        return comparison;
    }

    pub fn change_type(&self) -> ChangeType {
        if self.a.is_none() && self.b.is_some() {
            return ChangeType::Creation;
        }
        else if self.a.is_some() && self.b.is_none() {
            return ChangeType::Deletion;
        }
        else if let (Some(a), Some(b)) = (&self.a, &self.b) {
            return
                if a.sha1_checksum != b.sha1_checksum {
                    ChangeType::Modification
                } else {
                    ChangeType::NoChange
                };
        }
        unreachable!("comparison: change_type exhausted");
    }

    pub fn a(&self) -> String {
        self.a.map_or(String::new(), |node| node.name.clone())
    }

    pub fn set_a(&mut self, fs_node: &'a FsNode) {
        self.a = Some(fs_node);
    }

    pub fn b(&self) -> String {
        self.b.map_or(String::new(), |node| node.name.clone())
    }

    pub fn set_b(&mut self, fs_node: &'a FsNode) {
        self.b = Some(fs_node);
    }

    pub fn virtual_path(&self) -> PathBuf {
        PathBuf::from(&self.virtual_path)
    }

    pub fn virtual_path_clone(&self) -> String {
        self.virtual_path.clone()
    }
}