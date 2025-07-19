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
        Example::Syscall { cmd } => match cmd {
            SyscallCommand::Getaddrinfo { host } => bjrs::syscall::getaddrinfo(&host)?,
            SyscallCommand::Socket => bjrs::syscall::socket()?,
            SyscallCommand::Bind { reuse_port } => {
                if reuse_port {
                    bjrs::syscall::reuse_port()
                } else {
                    bjrs::syscall::bind()
                }?
            }
            SyscallCommand::Connect => bjrs::syscall::connect()?,
            SyscallCommand::Listen => bjrs::syscall::listen()?,
            SyscallCommand::Accept => {
                let _ = bjrs::syscall::accept()?;
            }
            SyscallCommand::Send => bjrs::syscall::send()?,
            SyscallCommand::Recv => bjrs::syscall::recv()?,
            SyscallCommand::Sendto => bjrs::syscall::sendto()?,
            SyscallCommand::Recvfrom => bjrs::syscall::recvfrom()?,
            SyscallCommand::Close => bjrs::syscall::close()?,
            SyscallCommand::Shutdown => bjrs::syscall::shutdown()?,
            SyscallCommand::Getpeername => bjrs::syscall::getpeername()?,
            SyscallCommand::Gethostname => bjrs::syscall::gethostname()?,
        },
        Example::Stream { cmd } => match cmd {
            StreamCommand::Server => bjrs::stream::server()?,
            StreamCommand::Client => bjrs::stream::client()?,
        },
        Example::Dgram { cmd } => match cmd {
            DgramCommand::Server => bjrs::dgram::server()?,
            DgramCommand::Client => bjrs::dgram::client()?,
        },
        Example::Techniques { cmd } => match cmd {
            TechniquesCommand::Blocking => bjrs::techniques::blocking()?,
            TechniquesCommand::Poll => bjrs::techniques::poll()?,
            TechniquesCommand::Pollserver => bjrs::techniques::pollserver()?,
            TechniquesCommand::Select => bjrs::techniques::select()?,
            TechniquesCommand::Selectserver => bjrs::techniques::selectserver()?,
            TechniquesCommand::Broadcaster { host, msg } => {
                bjrs::techniques::broadcaster(&host, &msg)?
            }
        },
    }

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    example: Example,
}

#[derive(Subcommand)]
enum Example {
    /// Chapter 5 - System Calls or Bust
    #[clap(alias = "sys")]
    Syscall {
        #[command(subcommand)]
        cmd: SyscallCommand,
    },

    /// Section 6.1 & 6.2 - A Simple Stream Server & Client
    Stream {
        #[command(subcommand)]
        cmd: StreamCommand,
    },

    /// Section 6.3 - Datagram Sockets
    Dgram {
        #[command(subcommand)]
        cmd: DgramCommand,
    },

    /// Chapter 7 - Slightly Advanced Techniques
    Techniques {
        #[command(subcommand)]
        cmd: TechniquesCommand,
    },
}

#[derive(Subcommand)]
enum SyscallCommand {
    /// Section 5.1 - `getaddrinfo()` - Prepare to Launch!
    Getaddrinfo { host: String },

    /// Section 5.2 - `socket()` - Get the File Descriptor!
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
}

#[derive(Subcommand)]
pub enum StreamCommand {
    /// Section 6.1 - A Simple Stream Server
    ///
    /// To test this example:
    ///
    /// Run this command to start our "TCP" server.
    /// In a separate terminal session, run the client command `bjrs stream client`.
    /// Observe that the server sends the message "Hello world!" to the client.
    Server,

    /// Section 6.2 - A Simple Stream Client
    ///
    /// To test this example, check out `bjrs help stream server`.
    /// You can also observe ECONNREFUSED error by running this command first before the server command.
    Client,
}

#[derive(Subcommand)]
pub enum DgramCommand {
    /// Section 6.3 - Datagram Sockets
    ///
    /// To test this example:
    ///
    /// Run this command to start our "UDP" server.
    /// In a separate terminal session, run the client command `bjrs dgram client`.
    /// Observe that the server receives the message "Hello UDP server!" from the client.
    Server,

    /// Section 6.3 - Datagram Sockets
    ///
    /// To test this example, check out `bjrs help dgram server`.
    /// You can also observe the nature of UDP packets by just running this command without the server. You will see that the packets will be sent without any errors.
    ///
    /// That's the gist with datagram sockets, the data sent through them is not guaranteed to arrive at the destination!
    Client,
}

#[derive(Subcommand)]
enum TechniquesCommand {
    /// Section 7.1 - Blocking
    Blocking,

    /// Section 7.2 - `poll()` - Synchronous I/O Multiplexing
    Poll,

    /// Section 7.2 - `poll()` - Synchronous I/O Multiplexing
    ///
    /// To test this example:
    ///
    /// Run this command to start our "TCP" server.
    /// Create connections from multiple terminal sessions via `telnet 127.0.0.1 9034` or via any command you prefer.
    /// Send messages from each terminal session to observe the server sending each message to all other clients.
    /// Close a client connection to observe that our server acknowleges it.
    /// Send messages from remaining connections to see that server does not try to send each message to the closed connections.
    Pollserver,

    /// Section 7.3 - `select()` - Synchronous I/O Multiplexing, Old School
    Select,

    /// Section 7.3 - `select()` - Synchronous I/O Multiplexing, Old School
    ///
    /// To test this example:
    ///
    /// Run this command to start our "TCP" server.
    /// Create connections from multiple terminal sessions via `telnet 0.0.0.0 9034` or via any command you prefer.
    /// Send messages from each terminal session to observe the server sending each message to all other clients.
    /// Close a client connection to observe that our server acknowleges it.
    /// Send messages from remaining connections to see that server does not try to send each message to the closed connections.
    Selectserver,

    /// Section 7.7 - Broadcast Packets - Hello, World!
    ///
    /// To test this example:
    ///
    /// Run `bjrs dgram server` to start our "UDP" server.
    ///
    /// Run this command with three different addresses: loopback (127.0.0.1), your local network's broadcast (192.168.X.255), and the broadcast of zero network (255.255.255.255). The message content does not matter.
    ///
    /// Observe that the server can receive the broadcast messages.
    /// Since the UDP server is implemented to recv a single message only, you will need to restart the server while trying different addresses.
    Broadcaster {
        /// The host address to send the message.
        host: String,

        /// The message to send.
        msg: String,
    },
}
