use std::fs;

// TODO: why isnt u64 implemented for ToSql?
#[derive(Default, Debug)]
pub struct FsNode {
    pub id: i64,
    pub node_type: NodeType,
    pub sha1_checksum: String, // 40 chars
    pub parent_path: String,
    pub name: String,
    pub size: i64,
    pub uid: u32,
    pub gid: u32,
    pub permissions: u32,
    pub creation_date: i64,
    pub modified_date: i64,
    pub links_to: String, // for soft links (symlinks)
    pub inode: i64,
    pub nlinks: i64, // number of hard links to this inode
    //pub parent_id: i64, // fk: FsNode::id
}

#[derive(Debug)]
pub enum NodeType {
    File,
    Directory,
    Symlink,
    Other
}

impl NodeType {
    pub fn value(&self) -> i32 {
        match self {
            NodeType::File => 0,
            NodeType::Directory => 1,
            NodeType::Symlink => 2,
            NodeType::Other => 3,
        }
    }
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::File
    }
}

impl FsNode {

    pub fn new() -> FsNode {
        FsNode::default()
    }

    pub fn insert (&self, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO fs_node ( \
                    node_type, \
                    sha1_checksum, \
                    parent_path, \
                    name, \
                    size, \
                    uid, \
                    gid, \
                    permissions, \
                    creation_date, \
                    modified_date, \
                    links_to, \
                    inode, \
                    nlinks) \
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                self.node_type.value(),
                self.sha1_checksum,
                self.parent_path,
                self.name,
                self.size,
                self.uid,
                self.gid,
                self.permissions,
                self.creation_date,
                self.modified_date,
                self.links_to,
                self.inode,
                self.nlinks
            ]
        )?;
        Ok(())
    }

    // pub fn getOrNew(conn: rusqlite::Connection, node: fs::DirEntry) -> crate::Result<FsNode> {
    //     let mut full_path = node.path();
    //     assert!(full_path.is_absolute());
    //
    //     let path = full_path.parent()
    //         .map_or(Some(""), |p| p.to_str());
    //     let path = match path {
    //         Some(s) => s,
    //         None => return Err(crate::error::Error::NoneError),
    //     };
    //
    //     let name = node.file_name();
    //     let name = match name.to_str() {
    //         Some(s) => s,
    //         None => return Err(crate::error::Error::NoneError),
    //     };
    //
    //     let mut cols_vect = Vec::new();
    //     let mut stmt = conn.prepare("SELECT * FROM fs_node WHERE name = ? AND path = ?")?;
    //     let mut rows = stmt.query(&[name, path])?;
    //
    //     while let Some(row) = rows.next()? {
    //         cols_vect.push(rows.columns());
    //     }
    //
    //     assert_eq!(1, cols_vect.len());
    //
    //     Ok(FsNode::new())
    // }
}


