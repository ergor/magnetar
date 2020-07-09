use crate::comparator::delta::Delta;
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
pub fn write(mut out_stream: impl Write, deltas: Vec<Delta<'_>>, summary: ReportSummary) -> io::Result<()> {
    let template = include_str!("report.html");

    let output_html = summary.into_html(template);

    let rows = make_rows(&deltas);
    let output_html = output_html.replace("${rows}", rows.as_str());
    let bytes_written = out_stream.write(output_html.as_bytes())?;

    log::debug!("wrote {} bytes to output stream.", bytes_written);

    Ok(())
}

fn make_rows(deltas: &Vec<Delta<'_>>) -> String {
    let mut rows = String::new();

    for delta in deltas {
        let mut row = TR.replace("${class}", delta.delta_type().css_class());
        row = row.replace("${root}", delta.root_path_str());
        row = row.replace("${ftype}", delta.file_type().to_str());
        row = row.replace("${vpath_attrs}",
                          if let NodeType::Directory = delta.file_type() {
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
        row = row.replace("${vpath}", v_path.as_str());
        //row = row.replace("${vpath}", delta.virtual_path_str());

        let delta_info = delta.delta_info();
        row = row.replace("${info}", delta_info.as_str());

        rows.push_str(row.as_str());
        rows.push_str("\n");
    }

    rows
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