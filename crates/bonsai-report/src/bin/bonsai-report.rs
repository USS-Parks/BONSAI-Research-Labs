use bonsai_report::{ReportData, generate_static_report};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bonsai-report: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let input = arguments
        .next()
        .ok_or("usage: bonsai-report <REPORT_INPUT_JSON> <OUTPUT_DIRECTORY>")?;
    let output = arguments
        .next()
        .ok_or("usage: bonsai-report <REPORT_INPUT_JSON> <OUTPUT_DIRECTORY>")?;
    if arguments.next().is_some() {
        return Err("usage: bonsai-report <REPORT_INPUT_JSON> <OUTPUT_DIRECTORY>".into());
    }
    let data: ReportData = serde_json::from_slice(&fs::read(input)?)?;
    let report = generate_static_report(&data)?;
    let output = Path::new(&output);
    fs::create_dir_all(output)?;
    fs::write(output.join("report.json"), report.machine_json)?;
    fs::write(output.join("report.html"), report.html)?;
    Ok(())
}
