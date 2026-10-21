use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use sisho_sdk::{Capabilities, ExtensionInfo};

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
    /// Disclose extension metadata for sisho.
    #[arg(long = "sisho-extension-info", hide = true)]
    sisho_extension_info: bool,

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

    if cli.sisho_extension_info {
        let info = ExtensionInfo::new(
            "desk",
            env!("CARGO_PKG_VERSION"),
            "Interactive desk for the Tomet ecosystem",
        )
        .with_capabilities(Capabilities::new().with_tui(true).with_interactive(true));
        if let Err(error) = info.print() {
            eprintln!("sisho-desk: failed to write extension info: {error}");
            return ExitCode::from(1);
        }
        return ExitCode::SUCCESS;
    }

    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        eprintln!("sisho-desk: standard input and output must be a terminal");
        return ExitCode::from(1);
    }

    match sisho_desk_tui::run_tui(cli.path, cli.config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sisho-desk: {error}");
            ExitCode::from(1)
        }
    }
}
