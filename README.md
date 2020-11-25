# magnetar

### Filesystem indexer &amp; comparator

#### Features:
- Index once; make multiple reports and variants thereof later.
- Generates interactive HTML reports from index results.
- Each index run is saved to its own SQLite database file for easy versioning and archival.
- Can detect changes in following metadata: `nodetype, checksum, size, user, group, permissions, birthdate, modifieddate, linksto, inode, nlinks`.
- Detect duplicates (Work In Progress!)
- The `root-x` options enables you to merge multiple sub-paths into a single pool for comparison on pool vs. pool instead of just a single dir vs. dir.
- Easily compare between two hosts. Just run the indexer locally on both, and `rsync` the index databases back when you want to create the reports.

## Example Usage

This example illustrates the pooling feature.

We will perform indexing and comparison on the following filesystem tree:

```
/tmp
└─── magnetar-demo
    ├── parent-a
    │   ├── fav-song.mp3
    │   └── some-subdir
    │       ├── cool-file.txt
    │       ├── image.jpg
    │       └── image_2.jpg
    ├── parent-b
    │   ├── not-so-cool-song.mp3
    │   └── some-subdir
    │       ├── cool-file.txt
    │       └── image.jpg
    └── parent-c
        └── fav-song.mp3
```

The situation here is that `parent-b` and `parent-c` _combined_ are almost identical to `parent-a`.

We want to find how `parent-a` compares to `parent-b` _and_ `parent-c`, as if they were one directory.

### Indexing

First, create an index database:

```
magnetar idx -o /tmp /tmp/magnetar-demo
```

This will index the directory `/tmp/magnetar-demo`. 
You can give it as many directories as you like (separated by space), as long as they are not sub directories of each other.
The index database will be saved in `/tmp` as instructed by the `-o` flag.

The index database file is saved as `magnetar-xxxx.db`, where `xxxx` is the unix timestamp the database was created. 
In my case, the full path to the database became `/tmp/magnetar-1606312134.db`

We may now use this database file to generate reports.

### Reports

#### Comparison

To compare directories, we load the databases we want to compare:

```
magnetar cmp -u \                        # the -u option tells magnetar to include files that are unchanged as well.
-a /tmp/magnetar-1606312134.db \         # Tells magnetar what index it should regard as source.
-b /tmp/magnetar-1606312134.db \         # Tells magnetar what index it should regard as destination.
--root-a /tmp/magnetar-demo/parent-a \   # We add /tmp/magnetar-demo/parent-a to the pool for our source.
--root-b /tmp/magnetar-demo/parent-b \   # We add /tmp/magnetar-demo/parent-b to the pool for our destination.
--root-b /tmp/magnetar-demo/parent-c \   # We _also_ add /tmp/magnetar-demo/parent-c to the pool for our destination.
> /tmp/comparison.html                   # Redirect the output of the command to the file /tmp/comparison.html. This will be our report.
```

Now we have a comparison report. When reading the report, interpret it as:

> These are the changes that would need to happen, if we were to make our `destination` identical to our `source`.

Running the above sequence of indexing and comparison will yield the following report:

![Generated HTML report](/doc/img/cmp-report.png?raw=true "The generated report")

What we can see from this report:

- The white rows are files/directories that exist in both source and destination, and they are identical. (can be omitted by not giving the `-u` option)
- The blue rows are files/directories that exist in both source and destination, but they are _not_ identical and we can see what has changed.
- The green rows are files/directories that exist in source, but not in destination.
- The red rows are files/directories that don't exist in source, but exist in destination.

#### Find Duplicates

Work in progress. Pull requests are welcomed! The feature is planned to be able to find files that have identical content, but with different names.

### Requirements

Currently only supports Linux, but support for other OSes is planned. Again, PRs are welcomed!
