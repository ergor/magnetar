-- exported from magnetar server v. 0.0.1-SNAPSHOT.
-- removed unapplicable tables.

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS "user" (
	"id"	bigint NOT NULL,
	"password_hash"	varchar(255),
	"user_name"	varchar(255),
	PRIMARY KEY("id")
);
CREATE TABLE IF NOT EXISTS "unix_user" (
	"id"	bigint NOT NULL,
	"name"	TEXT,
	"uid"	integer NOT NULL,
	"host_id"	bigint,
	PRIMARY KEY("id")
);
CREATE TABLE IF NOT EXISTS "unix_group" (
	"id"	bigint NOT NULL,
	"gid"	integer NOT NULL,
	"name"	TEXT,
	"host_id"	bigint,
	PRIMARY KEY("id")
);
CREATE TABLE IF NOT EXISTS "sshkey" (
	"id"	bigint NOT NULL,
	"user_id"	bigint,
	PRIMARY KEY("id")
);
CREATE TABLE IF NOT EXISTS "indexing_run" (
	"id"	bigint NOT NULL,
	"timestamp"	datetime,
	"host_id"	bigint,
	"parent_run_id"	bigint,
	PRIMARY KEY("id")
);
CREATE TABLE IF NOT EXISTS "host_address" (
	"id"	bigint NOT NULL,
	"ip_address"	TEXT,
	"host_id"	bigint,
	PRIMARY KEY("id")
);
CREATE TABLE IF NOT EXISTS "host" (
	"id"	bigint NOT NULL,
	"display_name"	TEXT,
	"fqdn"	TEXT,
	PRIMARY KEY("id")
);
CREATE TABLE IF NOT EXISTS "fs_node" (
	"id"	INTEGER PRIMARY KEY AUTOINCREMENT,
	"node_type"	INTEGER,
	"name"	TEXT,
	"size"	INTEGER NOT NULL,
	"uid"	INTEGER NOT NULL,
	"gid"	INTEGER NOT NULL,
	"permissions"	INTEGER NOT NULL,
	"creation_date"	INTEGER,
	"modified_date"	INTEGER,
	"parent_path"	TEXT,
	"sha1_checksum"	TEXT,
	"links_to"	TEXT,
	"inode" INTEGER ,
	"nlinks" INTEGER ,
	"parent_id"	INTEGER
);

COMMIT;