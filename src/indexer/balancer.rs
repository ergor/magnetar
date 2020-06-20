use std::fs::{DirEntry, ReadDir, Metadata};
use std::cell::RefCell;
use std::time::Instant;
use std::{fs, io, fmt};
use std::borrow::BorrowMut;
use std::fmt::{Display, Formatter};

pub struct Work {
    pub thread: usize,
    pub size: u64,
    pub entries: Vec<DirEntry>
}

impl Work {
    pub fn update(&mut self, dir_entry: DirEntry, metadata: &Metadata) {
        self.size += metadata.len();
        self.entries.push(dir_entry);
    }
}

impl Display for Work {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "workload: thread {}: {} KiB in {} files", self.thread, self.size, self.entries.len())
    }
}

/// Walks all the given directory trees, and creates lists of subtrees
/// that are as equal as possible in byte size, with the intent
/// of giving each subtree to their own thread as a measure of load balancing.
pub fn split_workload(directories: &Vec<String>, cpu_count: usize) -> Vec<Work> {

    let mut workload: Vec<Work> = Vec::new();

    for i in 0..cpu_count {
        workload.push(Work {
            thread: i,
            size: 0,
            entries: Vec::new()
        });
    }

    for dir in directories {
        scan_fstree(dir.as_str(), &mut workload).expect("scan_fstree returned Err");
    }

    workload
}

fn scan_fstree(top_level_path: &str, workload: &mut Vec<Work>) -> io::Result<()> {
    let mut dir_iter_stack: Vec<RefCell<ReadDir>> = Vec::new();
    let mut visit_log_stack: Vec<String> = Vec::new(); // for logging purposes

    let start_time = Instant::now();
    log::debug!("scan_fstree: '{}': start...", top_level_path);

    let top_level_entries_iter = fs::read_dir(top_level_path)?;
    dir_iter_stack.push(RefCell::new(top_level_entries_iter));
    visit_log_stack.push(top_level_path.to_string());

    let mut least_loaded_thread = least_loaded(workload);

    while !dir_iter_stack.is_empty() {
        let current_dir_iter = dir_iter_stack.last().unwrap(); // we know it's Some, because of loop condition
        let next_child = current_dir_iter.borrow_mut().next();

        if next_child.is_some() {
            let child: DirEntry = next_child.unwrap()?;

            let child_path = child.path();
            let child_path = child_path.to_str().unwrap_or("(unwrap error)");
            log::trace!("'{}': reading metadata...", child_path);

            let metadata = child.metadata()?;

            // dirs have a small size. this stops from quickly alternating least loaded initially.
            if !metadata.is_dir() {
                least_loaded_thread = least_loaded(workload);
                log::trace!("'{}': set thread {} as assignee", child_path, least_loaded_thread.thread);
            }

            least_loaded_thread.update(child, &metadata);

            log::trace!("'{}': assigned {:.2} K of work to thread {}. (total: {:.2} K)",
                    child_path,
                    metadata.len() as f32 /1024.0,
                    least_loaded_thread.thread,
                    least_loaded_thread.size as f32 /1024.0);

            if metadata.is_dir() {
                log::debug!("'{}': start descent...", child_path);
                dir_iter_stack.push(
                    RefCell::new(fs::read_dir(child_path)?)
                );
                visit_log_stack.push(child_path.to_string());
            }
        }
        else {
            let visited_path = visit_log_stack.pop().unwrap_or_else(|| "(unwrap error)".to_string());
            log::debug!("'{}': dir scan done.", visited_path);
            dir_iter_stack.pop();
        }
    }

    log::debug!("scan_fstree: '{}': done. time elapsed: {} ms.", top_level_path, start_time.elapsed().as_millis());
    Ok(())
}

fn least_loaded(workload: &mut Vec<Work>) -> &mut Work {
    workload.iter_mut()
        .min_by(|x, y| x.size.cmp(&y.size))
        .unwrap()
}