use crate::comparator::delta::{Delta, DeltaType, FieldDelta};
use std::io::Write;
use std::io;
use std::collections::HashSet;
use std::path::{PathBuf, Path};

const TR: &str =
    r#"<tr class="${class}">
        <td>${root}</td>
        <td class="slim">${ftype}</td>
        <td>${vpath}</td>
        <td>${info}</td>
    </tr>"#;

const SPAN: &str = "<span class=\"path\">{}/</span>";

/// Prints the generated report to the given stream.
pub fn write(mut out_stream: impl Write, deltas: Vec<Delta<'_>>) -> io::Result<()> {
    let html = include_str!("report.html");
    let rows = make_rows(&deltas);
    let generated = html.replace("${rows}", rows.as_str());
    let bytes_written = out_stream.write(generated.as_bytes())?;
    log::debug!("wrote {} bytes to output stream.", bytes_written);

    Ok(())
}

fn make_rows(deltas: &Vec<Delta<'_>>) -> String {
    let mut rows = String::new();

    for delta in deltas {
        let mut row = TR.replace("${class}", delta.delta_type().css_class());
        row = row.replace("${root}", delta.root_path_str());
        row = row.replace("${ftype}", delta.file_type());

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