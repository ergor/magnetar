use std::fmt;
use rusqlite::types::{FromSqlResult, ValueRef};
use std::fmt::Display;
use std::path::Path;
use crate::apperror::AppError;
use crate::errorwrapper::ErrorWrapper;

///
/// sha1_checksum: 40 chars long
/// links_to: for soft links (symlinks)
/// nlinks: number of hard links to this inode
/// TODO: use diesel for ORM. https://github.com/diesel-rs/diesel
/// i64 instead of u64 beacause of some sqlite spec.
#[derive(Default, Debug, Clone)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NodeType {
    File,
    Directory,
    Symlink,
    Other,
    Error
}

impl NodeType {

    pub fn from(value: u32) -> Option<NodeType> {
        match value {
            0 => Some(NodeType::File),
            1 => Some(NodeType::Directory),
            2 => Some(NodeType::Symlink),
            3 => Some(NodeType::Other),
            4 => Some(NodeType::Error),
            _=> None,
        }
    }

    pub fn value(&self) -> u32 {
        match self {
            NodeType::File => 0,
            NodeType::Directory => 1,
            NodeType::Symlink => 2,
            NodeType::Other => 3,
            NodeType::Error => 4,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            NodeType::File => {""},
            NodeType::Directory => {"D"},
            NodeType::Symlink => {"L"},
            NodeType::Other => {"-"},
            NodeType::Error => {"ERR"},
        }
    }
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::File
    }
}

impl Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match &self {
            NodeType::File => { "file" },
            NodeType::Directory => { "dir" },
            NodeType::Symlink => { "symlink" },
            NodeType::Other => { "(other)" },
            NodeType::Error => { "(none (read error))" },
        };
        write!(f, "{}", val)
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

    pub fn select_n(db_path: &str) -> crate::ConvertibleResult<Vec<FsNode>> {
        log::debug!("fetching fs_nodes from '{}'", db_path);

        let index_db_path = Path::new(db_path);
        if !index_db_path.exists() {
            let error = AppError::WithMessage(
                format!("database '{}' not found.", index_db_path.to_string_lossy())
            );
            log::error!("{}", error);
            return Err(ErrorWrapper::AppError(error))
        }

        let mut fs_nodes = Vec::new();
        { // open for db work
            let conn = rusqlite::Connection::open(index_db_path)?;
            log::debug!("{}: database connection opened", index_db_path.to_string_lossy().as_ref());
            std::mem::drop(fs_nodes);
            fs_nodes = FsNode::select(&conn)?;
            log::debug!("{}: retrieved {} rows.", index_db_path.to_string_lossy().as_ref(), fs_nodes.len());
        } // drops all db connections
        log::debug!("{}: database connection closed", index_db_path.to_string_lossy().as_ref());

        Ok(fs_nodes)
    }
}


