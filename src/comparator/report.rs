use crate::comparator::delta::{Delta, DeltaType};
use std::io::Write;
use std::io;

const TR: &str =
    r#"<tr class="${class}">
        <td>${root}</td>
        <td>${vpath}</td>
        <td>${ftype}</td>
        <td>${delta}</td>
    </tr>"#;


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
        let css_class = match delta.delta_type() {
            DeltaType::Creation => { "creation" },
            DeltaType::Deletion => { "deletion" },
            DeltaType::Modification => { "modification" },
            DeltaType::NoChange => { "no-change" },
        };
        let mut row = TR.replace("${class}", css_class);
        row = row.replace("${root}", delta.root_path_str());
        row = row.replace("${vpath}", delta.virtual_path_str());
        row = row.replace("${ftype}", "N/A");
        row = row.replace("${delta}", "N/A");
        rows.push_str(row.as_str());
        rows.push_str("\n");
    }

    rows
}