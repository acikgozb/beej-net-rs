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

    match cli.example {
        Example::ShowIp { host } => beej_net_rs::showip(&host)?,
        Example::Socket => beej_net_rs::socket()?,
        Example::Bind { reuse_port } => match reuse_port {
            true => beej_net_rs::reuse_port()?,
            false => beej_net_rs::bind()?,
        },
        Example::Connect => beej_net_rs::connect()?,
        Example::Listen => beej_net_rs::listen()?,
        Example::Accept => {
            let _ = beej_net_rs::accept()?;
        }
        Example::Send => beej_net_rs::send()?,
        Example::Recv => beej_net_rs::recv()?,
        Example::Sendto => beej_net_rs::sendto()?,
        Example::Recvfrom => beej_net_rs::recvfrom()?,
        Example::Close => beej_net_rs::close()?,
        Example::Shutdown => beej_net_rs::shutdown()?,
        Example::Getpeername => beej_net_rs::getpeername()?,
        Example::Gethostname => beej_net_rs::gethostname()?,
        Example::Stream { cmd } => match cmd {
            StreamCommand::Server => beej_net_rs::stream_server()?,
            StreamCommand::Client => beej_net_rs::stream_client()?,
        },
    };

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    example: Example,
}

#[derive(Subcommand)]
pub enum Example {
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

    /// Section 5.8 - `sendto() and recvfrom()` - Talk to me, DGRAM-style
    ///
    /// To test this example:
    ///
    /// Run this command to start our "UDP server".
    /// Send a UDP message from a separate terminal session by using `ncat -u 127.0.0.1 3490 <<< "hello UDP message!"` or via any command you prefer.
    /// Observe that the message "hello UDP message!" appears on our process' terminal session.
    Recvfrom,

    /// Section 5.9 - `close() and shutdown()` - Get outta my face!
    Close,

    /// Section 5.9 - `close() and shutdown()` - Get outta my face!
    ///
    /// To test this example:
    ///
    /// Run this command to start our "TCP" server.
    /// Connect to this server in a separate terminal session by using `ncat 127.0.0.1 3490` or via any command you prefer.
    /// Observe that the server cannot send a message due to EPIPE error, which happens because of `shutdown()`.
    Shutdown,

    /// Section 5.10 - `getpeername()` - Who are you?
    ///
    /// To test this example:
    ///
    /// Run this command to start our "TCP" server.
    /// Connect to this server in a separate terminal session by using `ncat 127.0.0.1 3490` or via any command you prefer.
    /// Observe that our server writes the source IP address and it's port to the stdout.
    Getpeername,

    /// Section 5.11 - `gethostname()` - Who am I?
    Gethostname,

    /// Section 6.1 & 6.2 - A Simple Stream Server & Client
    Stream {
        #[command(subcommand)]
        cmd: StreamCommand,
    },
}

#[derive(Subcommand)]
pub enum StreamCommand {
    /// Section 6.1 - A Simple Stream Server
    ///
    /// To test this example:
    ///
    /// Run this command to start our "TCP" server.
    /// In a separate terminal session, run the client command `beej_net_rs stream client`.
    /// Observe that the server sends the message "Hello world!" to the client.
    Server,
    /// Section 6.2 - A Simple Stream Client
    ///
    /// To test this example, check out `beej_net_rs help stream server`.
    /// You can also observe ECONNREFUSED error by running this command first before the server command.
    Client,
}
