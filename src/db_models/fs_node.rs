use std::fs;
use rusqlite::types::{FromSqlResult, ValueRef};

///
/// sha1_checksum: 40 chars long
/// links_to: for soft links (symlinks)
/// nlinks: number of hard links to this inode
// macro_rules! fsnode_fields {
//     //( $prefix:expr, $postfix:expr ) => {
//     //    $prefix id $postfix
//     //}
//
//     () => {
//         id
//         node_type
//         sha1_checksum
//         parent_path
//         name
//         size
//         uid
//         gid
//         permissions
//         creation_date
//         modified_date
//         links_to
//         inode
//         nlinks
//     }
// }

// TODO: use diesel for ORM. https://github.com/diesel-rs/diesel
// i64 instead of u64 beacause of some sqlite spec.
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

    pub fn from(value: u32) -> Option<NodeType> {
        match value {
            0 => Some(NodeType::File),
            1 => Some(NodeType::Directory),
            2 => Some(NodeType::Symlink),
            3 => Some(NodeType::Other),
            _=> None,
        }
    }

    pub fn value(&self) -> u32 {
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

impl rusqlite::types::FromSql for NodeType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<NodeType> {
        let value = value.as_i64()?;
        let node_type_opt = NodeType::from(value as u32);
        match node_type_opt {
            Some(node_type) => Ok(node_type),
            None => Err(rusqlite::types::FromSqlError::OutOfRange(value))
        }
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

    pub fn select(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<FsNode>> {
        let mut fs_nodes = Vec::new();
        let mut stmt = conn.prepare("SELECT \
                    id, \
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
                    nlinks \
                    FROM fs_node")?;
        let row_iterator = stmt.query_map(rusqlite::NO_PARAMS, |row| FsNode::map_from_row(row))?;
        for fs_node in row_iterator {
            let fs_node = fs_node?;
            fs_nodes.push(fs_node);
        }
        Ok(fs_nodes)
    }

    pub fn map_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<FsNode> {
        let c = row.column_count();
        Ok(FsNode {
            id: row.get("id")?,
            node_type: row.get("node_type")?,
            sha1_checksum: row.get("sha1_checksum")?,
            parent_path: row.get("parent_path")?,
            name: row.get("name")?,
            size: row.get("size")?,
            uid: row.get("uid")?,
            gid: row.get("gid")?,
            permissions: row.get("permissions")?,
            creation_date: row.get("creation_date")?,
            modified_date: row.get("modified_date")?,
            links_to: row.get("links_to")?,
            inode: row.get("inode")?,
            nlinks: row.get("nlinks")?,
        })
    }

    // pub fn set<T> (&mut self, field: &str, value: T) {
    //     match field {
    //         "id" => { self.id = value as i64 },
    //         "node_type" => { self.node_type = value as NodeType },
    //         "sha1_checksum" => { self.sha1_checksum = value as String },
    //         "parent_path" => { self.parent_path = value as String },
    //         "name" => { self.name = value as String },
    //         "size" => { self.size = value as i64 },
    //         "uid" => { self.uid = value as u32 },
    //         "gid" => { self.gid = value as u32 },
    //         "permissions" => { self.permissions = value as u32 },
    //         "creation_date" => { self.creation_date = value as i64 },
    //         "modified_date" => { self.modified_date = value as i64 },
    //         "links_to" => { self.links_to = value as String },
    //         "inode" => { self.inode = value as i64 },
    //         "nlinks" => { self.nlinks = value as i64 },
    //         _ => { unimplemented!("{}: no setter for the field.", field)}
    //     }
    // }

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


