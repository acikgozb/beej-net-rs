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
        Examples::Connect => beej_net_rs::connect()?,
        Examples::Listen => beej_net_rs::listen()?,
        Examples::Accept => {
            let _ = beej_net_rs::accept()?;
        }
        Examples::Send => beej_net_rs::send()?,
        Examples::Recv => beej_net_rs::recv()?,
        Examples::Sendto => beej_net_rs::sendto()?,
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

    /// Section 5.4 - `connect()` - Hey, you!
    Connect,

    /// Section 5.5 - `listen()` - Will Somebody Please Call Me?
    Listen,

    /// Section 5.6 - `accept()` - "Thank you for calling port 3490."
    Accept,

    /// Section 5.7 - `send() and recv()` - Talk to me, baby!
    ///
    /// To test the example:
    ///
    /// Run this command in the background.
    /// Find out the listened IP address (IP or IPv6) via `lsof -niTCP:3490` or via any command you prefer.
    /// Initiate a connection to see the sent data. The easiest would probably be `ncat <IP_ADDR> 3490`.
    Send,

    /// Section 5.7 - `send() and recv()` - Talk to me, baby!
    ///
    /// To test this example:
    ///
    /// Run this command in the background.
    /// Find out the listened IP address (IP or IPv6) via `lsof -niTCP:3490` or via any command you prefer.
    /// Initiate a connection and send a message to the process. The easiest would be `ncat <IP_ADDR> 3490 <<< "string message"`.
    Recv,

    /// Section 5.8 - `sendto() and recvfrom()` - Talk to me, DGRAM-style
    ///
    /// To test this example:
    ///
    /// Boot up a UDP server listening on localhost, on port 3490 by using `ncat -ul 127.0.0.1 3490`.
    /// Run this command in a separate terminal session.
    /// Observe that the message "hello world!" appears on the UDP server's terminal session.
    Sendto,
}
