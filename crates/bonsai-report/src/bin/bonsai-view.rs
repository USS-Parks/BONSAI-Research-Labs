use bonsai_report::viewer::ReadOnlyBundleViewer;
use std::error::Error;
use std::ffi::OsString;
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bonsai-view: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: &[OsString]) -> Result<(), Box<dyn Error>> {
    if arguments.is_empty() || arguments.len() > 2 {
        return Err("usage: bonsai-view <BUNDLE_ROOT> [RELATIVE_FILE]".into());
    }
    let viewer = ReadOnlyBundleViewer::open(&arguments[0])?;
    let bytes = if arguments.len() == 2 {
        viewer.read(&arguments[1])?
    } else {
        viewer.load_static_report()?.html.into_bytes()
    };
    io::stdout().lock().write_all(&bytes)?;
    Ok(())
}
