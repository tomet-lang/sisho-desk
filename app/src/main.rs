use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

/// The interactive desk for Tomet: Markdown migration, batch metadata
/// editing, and structural search and replace over documents in a directory.
///
/// It takes over the terminal until you quit, and puts it back however
/// it ends.
#[derive(Parser)]
#[command(
    name = "sisho-desk",
    version,
    about = "Interactive desk for the Tomet ecosystem"
)]
struct Cli {
    /// File or directory to open (defaults to ".").
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Path to a config file (.tmt). If not given, the config found by walking
    /// up from the path is used, or the default if none is found.
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match sisho_desk_tui::run_tui(cli.path, cli.config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sisho-desk: {error}");
            ExitCode::from(1)
        }
    }
}
