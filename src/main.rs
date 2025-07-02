use std::{error, process::ExitCode};

use clap::{Parser, Subcommand, command};

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn error::Error>> {
    let cli = Cli::parse();

    match cli.examples {
        Examples::ShowIp { host } => beej_net_rs::showip(&host)?,
        Examples::Socket => beej_net_rs::socket()?,
        Examples::Bind { reuse_port } => match reuse_port {
            true => beej_net_rs::reuse_port()?,
            false => beej_net_rs::bind()?,
        },
    };

    Ok(())
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

    /// Section 5.3 - `bind()` - What Port Am I On?
    Bind {
        /// Set SO_REUSEADDR socket option.
        #[arg(short, long, default_value_t = false)]
        reuse_port: bool,
    },
}
