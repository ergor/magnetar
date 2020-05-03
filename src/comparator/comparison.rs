use crate::db_models::fs_node::FsNode;

pub struct Comparison<'a> {
    a: Option<&'a FsNode>,
    b: Option<&'a FsNode>
}

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
    pub fn new(a: Option<&'a FsNode>, b: Option<&'a FsNode>) -> Comparison<'a> {
        Comparison {
            a,b
        }
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

    pub fn b(&self) -> String {
        self.b.map_or(String::new(), |node| node.name.clone())
    }
}