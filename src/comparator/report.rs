use crate::comparator::delta::{Delta, DeltaType};
use std::io::Write;
use std::io;
use std::path::PathBuf;
use crate::db_models::fs_node::NodeType;

const TR: &str =
    r#"<tr class="${class}">
        <td>${root}</td>
        <td class="slim">${ftype}</td>
        <td${vpath_attrs}>${vpath}</td>
        <td>${info}</td>
    </tr>"#;

const SPAN: &str = "<span class=\"path\">{}/</span>";

pub struct ReportSummary {
    pub db_a_name: String,
    pub db_b_name: String,
    pub roots_a: Vec<String>,
    pub roots_b: Vec<String>,
}

/// Prints the generated report to the given stream.
pub fn write(mut out_stream: impl Write, deltas: Vec<Delta<'_>>, keep_unchanged: bool, collapse: bool, summary: ReportSummary) -> io::Result<()> {
    let template = include_str!("report.html");

    let output_html = summary.into_html(template);

    let deltas = process_unchanged(deltas, keep_unchanged);
    let deltas = process_collapse(deltas, collapse);

    let rows = make_rows(&deltas, collapse);

    let mut output_html = output_html.replace("${rows}", rows.as_str());
    output_html = output_html.replace("${keep-unchanged}", if keep_unchanged {""} else {"hidden"});

    let bytes_written = out_stream.write(output_html.as_bytes())?;

    log::debug!("wrote {} bytes to output stream.", bytes_written);

    Ok(())
}

fn make_rows(deltas: &Vec<Delta<'_>>, pre_collapsed: bool) -> String {
    let mut rows = String::new();

    for delta in deltas {
        let is_delta_precollapsed = pre_collapsed && delta.delta_type().is_created_or_deleted();
        let mut row = TR.replace("${class}", delta.delta_type().css_class());
        row = row.replace("${root}", delta.root_path_str());
        row = row.replace("${ftype}", delta.file_type().to_str());
        row = row.replace("${vpath_attrs}",
                          if !is_delta_precollapsed && delta.file_type().is_dir() {
                              " class=\"dir\" onclick=\"onclickCollapse(this)\""
                          } else {
                              ""
                          });

        let v_path_str = delta.virtual_path_str();
        let v_path_buf = PathBuf::from(v_path_str);
        let v_path = match v_path_buf.parent() {
            None => {
                format!("{}", v_path_str)
            },
            Some(parent) => {
                let parent = parent.to_str().unwrap();
                if parent.is_empty() {
                    format!("{}", v_path_str)
                } else {
                    let parent_span = SPAN.replace("{}", parent);
                    let name = v_path_buf.file_name().unwrap().to_str().unwrap();
                    format!("{}{}", parent_span, name)
                }
            },
        };
        let v_path = format!("{}{}", v_path.as_str(), if is_delta_precollapsed && delta.file_type().is_dir() {" [...]"} else {""});
        row = row.replace("${vpath}", v_path.as_str());

        let delta_info = delta.delta_info();
        row = row.replace("${info}", delta_info.as_str());

        rows.push_str(row.as_str());
        rows.push_str("\n");
    }

    rows
}

/// If `keep_unchanged` is set, returns the original list. Otherwise, returns the
/// list without deltas of type unchanged.
fn process_unchanged(deltas: Vec<Delta<'_>>, keep_unchanged: bool) -> Vec<Delta<'_>> {
    if keep_unchanged {
        deltas
    } else {
        deltas
            .into_iter()
            .filter(|delta| !delta.delta_type().is_unchanged())
            .collect()
    }
}

/// IMPORTANT: this function assumes the input `deltas` are ordered by path.
/// If `collapse` is set, collapses the result such that created or deleted
/// directories only show up as 1 row with how many nodes there are under that node.
fn process_collapse(deltas: Vec<Delta<'_>>, collapse: bool) -> Vec<Delta<'_>> {
    if !collapse {
        return deltas;
    }

    log::debug!("collapsing created or deleted directories...");
    let start_count = deltas.len();
    let mut current_parent = None;
    let mut filtered = Vec::new();

    for delta in deltas {
        if delta.delta_type().is_created_or_deleted() {
            let vpath = PathBuf::from(delta.virtual_path_str());

            if delta.file_type().is_dir() {
                let new_parent = vpath.clone();
                match &current_parent {
                    None => {
                        log::trace!("new parent: '{}'", new_parent.to_string_lossy());
                        current_parent = Some(new_parent);
                    },
                    Some(parent) => {
                        if !new_parent.starts_with(parent) {
                            log::trace!("new parent: '{}'", new_parent.to_string_lossy());
                            current_parent = Some(new_parent);
                        }
                    },
                }
            }

            if let Some(parent) = &current_parent {
                if vpath != *parent && vpath.starts_with(parent) {
                    log::trace!("skipping '{}'", vpath.to_string_lossy());
                    continue;
                }
            }
        }

        filtered.push(delta);
    }

    log::debug!("collapse done. items skipped: {}", start_count - filtered.len());
    filtered
}

impl ReportSummary {
    pub fn into_html(self, html: &str) -> String {
        let html = html.replace("${db-a}", self.db_a_name.as_str());
        let html = html.replace("${db-b}", self.db_b_name.as_str());

        let joined = String::from(self.roots_a.join(", "));
        let html = html.replace("${roots-a}", joined.as_str());

        let joined = String::from(self.roots_b.join(", "));
        let html = html.replace("${roots-b}", joined.as_str());

        html
    }
}