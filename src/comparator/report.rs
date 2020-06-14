use crate::comparator::delta::Delta;
use std::io::Write;
use std::io;

/// Prints the generated report to the given stream.
pub fn write(mut out_stream: impl Write, deltas: Vec<Delta<'_>>) -> io::Result<()> {
    let html = include_str!("report.html");
    let generated = html.replace("${tree-nodes}", "");
    let bytes_written = out_stream.write(generated.as_bytes())?;
    log::debug!("wrote {} bytes to output stream.", bytes_written);

    Ok(())
}