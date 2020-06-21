
pub enum Permission {
    Socket,           // socket
    Symlink,          // symbolic link
    File,             // regular file
    BlkDev,           // block device
    Dir,              // directory
    CharDev,          // character device
    Fifo,             // FIFO

    SetUid,           // set-user-ID bit (see execve(2))
    SetGid,           // set-group-ID bit
    Sticky,           // sticky bit

    UserRead,         // owner has read permission
    UserWrite,        // owner has write permission
    UserExecute,      // owner has execute permission
    GroupRead,        // group has read permission
    GroupWrite,       // group has write permission
    GroupExecute,     // group has execute permission
    OtherRead,        // others have read permission
    OtherWrite,       // others have write permission
    OtherExecute,     // others have execute permission
}

impl Permission {
    pub fn bit(&self) -> u32 {
        match self {
            Permission::Socket =>       { 0o_140000 },
            Permission::Symlink =>      { 0o_120000 },
            Permission::File =>         { 0o_100000 },
            Permission::BlkDev =>       { 0o_060000 },
            Permission::Dir =>          { 0o_040000 },
            Permission::CharDev =>      { 0o_020000 },
            Permission::Fifo =>         { 0o_010000 },
            Permission::SetUid =>       { 0o___4000 },
            Permission::SetGid =>       { 0o___2000 },
            Permission::Sticky =>       { 0o___1000 },
            Permission::UserRead =>     { 0o___0400 },
            Permission::UserWrite =>    { 0o___0200 },
            Permission::UserExecute =>  { 0o___0100 },
            Permission::GroupRead =>    { 0o___0040 },
            Permission::GroupWrite =>   { 0o___0020 },
            Permission::GroupExecute => { 0o___0010 },
            Permission::OtherRead =>    { 0o___0004 },
            Permission::OtherWrite =>   { 0o___0002 },
            Permission::OtherExecute => { 0o___0001 },
        }
    }

    /// The symbolic representation of the perimission bit.
    /// Return tuple of position and character, with position 0 being start of string (leftmost).
    pub fn repr(&self) -> (usize, char) {
        match self {
            Permission::Socket =>       { (0, 's') },
            Permission::Symlink =>      { (0, 'l') },
            Permission::File =>         { (0, '-') },
            Permission::BlkDev =>       { (0, 'b') },
            Permission::Dir =>          { (0, 'd') },
            Permission::CharDev =>      { (0, 'c') },
            Permission::Fifo =>         { (0, 'p') },
            Permission::SetUid =>       { (3, 'S') }, // lowercase if also executable is set
            Permission::SetGid =>       { (6, 'S') }, // lowercase if also executable is set
            Permission::Sticky =>       { (9, 'T') }, // lowercase if also executable is set
            Permission::UserRead =>     { (1, 'r') },
            Permission::UserWrite =>    { (2, 'w') },
            Permission::UserExecute =>  { (3, 'x') },
            Permission::GroupRead =>    { (4, 'r') },
            Permission::GroupWrite =>   { (5, 'w') },
            Permission::GroupExecute => { (6, 'x') },
            Permission::OtherRead =>    { (7, 'r') },
            Permission::OtherWrite =>   { (8, 'w') },
            Permission::OtherExecute => { (9, 'x') },
        }
    }

    pub fn is_set(&self, permbits: u32) -> bool {
        self.bit() & permbits > 0
    }

    pub fn from_val(permbits: u32) -> String {
        macro_rules! insert_perm {
            ($permission:expr => $string:ident) => {
                if $permission.is_set(permbits) {
                    let (i, c) = $permission.repr();
                    $string = replace_char_at($string, i, c);
                }
            }
        }

        macro_rules! update_perm {
            ($permission:expr => $string:ident, $execute_perm:expr) => {
                if $permission.is_set(permbits) {
                    let (i, c) = $permission.repr();
                    $string = replace_char_at($string, i, if $execute_perm.is_set(permbits) {
                        c.to_ascii_lowercase()
                    } else {
                        c
                    });
                }
            }
        }

        let mut x = "----------".to_string();

        // order is important, as we want to have the basic perms first, then overwrite
        // with special if applicable.

        insert_perm!(Permission::OtherRead => x);
        insert_perm!(Permission::OtherWrite => x);
        insert_perm!(Permission::OtherExecute => x);

        insert_perm!(Permission::GroupRead => x);
        insert_perm!(Permission::GroupWrite => x);
        insert_perm!(Permission::GroupExecute => x);

        insert_perm!(Permission::UserRead => x);
        insert_perm!(Permission::UserWrite => x);
        insert_perm!(Permission::UserExecute => x);

        // SetUid, SetGid and Sticky are special in that they are lowercased conditionally
        update_perm!(Permission::Sticky => x, Permission::OtherExecute);
        update_perm!(Permission::SetGid => x, Permission::GroupExecute);
        update_perm!(Permission::SetUid => x, Permission::UserExecute);

        insert_perm!(Permission::Socket => x);
        insert_perm!(Permission::Symlink => x);
        insert_perm!(Permission::File => x);
        insert_perm!(Permission::BlkDev => x);
        insert_perm!(Permission::Dir => x);
        insert_perm!(Permission::CharDev => x);
        insert_perm!(Permission::Fifo => x);

        x
    }
}

fn replace_char_at(s: String, idx: usize, r: char) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| if i == idx { r } else { c })
        .collect()
}
