use std::process::ExitCode;

use clap::{Parser, Subcommand, command};

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.examples {
        Examples::ShowIp { host } => beej_net_rs::showip(&host),
        Examples::Socket => beej_net_rs::socket(),
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(ecode) => {
            let ecode = u8::try_from(ecode).ok().unwrap_or(1u8);
            ExitCode::from(ecode)
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    examples: Examples,
}

#[derive(Subcommand)]
pub enum Examples {
    /// Section 5.1 - `getaddrinfo()` - Prepare to Launch!
    #[clap(name = "showip")]
    ShowIp { host: String },

    /// Section 5.2 - `socket()` - Get the File Descriptor!
    #[clap(name = "sock")]
    Socket,
}
