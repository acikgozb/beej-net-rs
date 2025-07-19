# `bjrs`

This repository contains the Rust rewrites of examples that are explained in [Beej's Guide to Network Programming](https://beej.us/guide/bgnet/).

## Table of Contents

<!--toc:start-->
  - [Requirements](#requirements)
  - [Implementation](#implementation)
  - [Usage](#usage)
  - [Examples](#examples)
    - [Section 5.1 - `getaddrinfo()` - Prepare to Launch!](#section-51-getaddrinfo-prepare-to-launch)
    - [Section 5.2 - `socket()` - Get the File Descriptor!](#section-52-socket-get-the-file-descriptor)
    - [Section 5.3 - `bind()` - What Port Am I On?](#section-53-bind-what-port-am-i-on)
    - [Section 5.4 - `connect()` - Hey, you!](#section-54-connect-hey-you)
    - [Section 5.5 - `listen()` - Will Somebody Please Call Me?](#section-55-listen-will-somebody-please-call-me)
    - [Section 5.6 - `accept()` - "Thank you for calling port 3490."](#section-56-accept-thank-you-for-calling-port-3490)
    - [Section 5.7 - `send() and recv()` - Talk to me, baby!](#section-57-send-and-recv-talk-to-me-baby)
    - [Section 5.8 - `sendto() and recvfrom()` - Talk to me, DGRAM-style](#section-58-sendto-and-recvfrom-talk-to-me-dgram-style)
    - [Section 5.9 - `close() and shutdown()` - Get outta my face!](#section-59-close-and-shutdown-get-outta-my-face)
    - [Section 5.10 - `getpeername()` - Who are you?](#section-510-getpeername-who-are-you)
    - [Section 5.11 - `gethostname()` - Who am I?](#section-511-gethostname-who-am-i)
    - [Section 6.1 & 6.2 - A Simple Stream Server & Client](#section-61-62-a-simple-stream-server-client)
    - [Section 6.3 - Datagram Sockets](#section-63-datagram-sockets)
    - [Section 7.1 - Blocking](#section-71-blocking)
    - [Section 7.2 - `poll()` - Synchronous I/O Multiplexing](#section-72-poll-synchronous-io-multiplexing)
    - [Section 7.3 - `select()` - Synchronous I/O Multiplexing, Old School](#section-73-select-synchronous-io-multiplexing-old-school)
    - [Section 7.7 - Broadcast Packets - Hello, World!](#section-77-broadcast-packets-hello-world)
  - [Notes](#notes)
<!--toc:end-->

## <a id='requirements'></a> Requirements

The Rust toolchain needs to be installed on the host to run the examples.
It can be installed from [here](https://www.rust-lang.org/tools/install).

## <a id='implementation'></a> Implementation

I tried to keep the examples as close to the original implementations as possible, therefore the project only uses the `libc` crate for each example.
As a result, `unsafe` blocks are heavily used throughout the project.

The only exception to this decision is the C examples of `pollserver` and `selectserver`.
These examples do more unsafe operations than the underlying syscalls that are basically not accepted by safe Rust (e.g. mutating a collection while traversing).
Therefore, in order to keep `unsafe` blocks under control, additional structures are used for these, which made them a bit more verbose compared to their C counterparts.

## <a id='usage'></a> Usage

Some examples can be run standalone, but some require multiple shell sessions.
Before running the examples, I would recommend to check out the long help text first.

```bash
$ cargo run -- --help
# Usage: bjrs <COMMAND>
#                                                                                                                                                                             
# Commands:
#   syscall     Chapter 5 - System Calls or Bust
#   stream      Section 6.1 & 6.2 - A Simple Stream Server & Client
#   dgram       Section 6.3 - Datagram Sockets
#   techniques  Chapter 7 - Slightly Advanced Techniques
#   help        Print this message or the help of the given subcommand(s)
#                                                                                                                                                                             
# Options:
#   -h, --help     Print help
#   -V, --version  Print version
```

```bash
$ cargo run -- syscall --help
# Chapter 5 - System Calls or Bust
#                                                                                                                                                                             
# Usage: bjrs syscall <COMMAND>
#                                                                                                                                                                             
# Commands:
#   getaddrinfo  Section 5.1 - `getaddrinfo()` - Prepare to Launch!
#   socket       Section 5.2 - `socket()` - Get the File Descriptor!
#   bind         Section 5.3 - `bind()` - What Port Am I On?
#   connect      Section 5.4 - `connect()` - Hey, you!
#   listen       Section 5.5 - `listen()` - Will Somebody Please Call Me?
#   accept       Section 5.6 - `accept()` - "Thank you for calling port 3490."
#   send         Section 5.7 - `send() and recv()` - Talk to me, baby!
#   recv         Section 5.7 - `send() and recv()` - Talk to me, baby!
#   sendto       Section 5.8 - `sendto() and recvfrom()` - Talk to me, DGRAM-style
#   recvfrom     Section 5.8 - `sendto() and recvfrom()` - Talk to me, DGRAM-style
#   close        Section 5.9 - `close() and shutdown()` - Get outta my face!
#   shutdown     Section 5.9 - `close() and shutdown()` - Get outta my face!
#   getpeername  Section 5.10 - `getpeername()` - Who are you?
#   gethostname  Section 5.11 - `gethostname()` - Who am I?
#   help         Print this message or the help of the given subcommand(s)
#                                                                                                                                                                             
# Options:
#   -h, --help  Print help
```

Here is an example where it needs manual preparation:

```bash
$ cargo run -- stream server --help
# Section 6.1 - A Simple Stream Server
#                                                                                                                                                                             
# To test this example:
#                                                                                                                                                                             
# Run this command to start our "TCP" server. In a separate terminal session, run the client command `bjrs stream client`. Observe that the server sends the message "Hello world!" to the client.
#                                                                                                                                                                             
# Usage: bjrs stream server
#                                                                                                                                                                             
# Options:
#   -h, --help
#           Print help (see a summary with '-h')
```

## <a id='examples'></a> Examples

Some examples in Beej's book are in their own C files and some of them are inlined to `.md` pages.

Here is how the examples in this project are mapped to Beej's book. The inlined examples are shown with `md` prefix, and C files are shown as is:

### <a id='section-51-getaddrinfo-prepare-to-launch'></a> Section 5.1 - `getaddrinfo()` - Prepare to Launch!

[showip.c](https://github.com/beejjorgensen/bgnet/blob/main/source/examples/showip.c) -> [getaddrinfo.rs](./src/syscall/getaddrinfo.rs)

### <a id='section-52-socket-get-the-file-descriptor'></a> Section 5.2 - `socket()` - Get the File Descriptor!

For `socket()`, Beej shows the usage via a pseudocode. In here, I tried to build on `getaddrinfo()` to showcase how a basic `socket()` call can be made.

[md-socket](https://github.com/beejjorgensen/bgnet/blob/main/src/bgnet_part_0500_syscalls.md?plain=1#L280) -> [socket.rs](./src/syscall/socket.rs)

### <a id='section-53-bind-what-port-am-i-on'></a> Section 5.3 - `bind()` - What Port Am I On?

[md-bind](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L336) -> [bind.rs](./src/syscall/bind.rs) (fn `bind`)


[md-bind-port-reuse](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L411) -> [bind.rs](./src/syscall/bind.rs) (fn `reuse_port`)

### <a id='section-54-connect-hey-you'></a> Section 5.4 - `connect()` - Hey, you!

[md-connect](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L461) -> [connect.rs](./src/syscall/connect.rs)


### <a id='section-55-listen-will-somebody-please-call-me'></a> Section 5.5 - `listen()` - Will Somebody Please Call Me?

[md-listen](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L527) -> [listen.rs](./src/syscall/listen.rs)

### <a id='section-56-accept-thank-you-for-calling-port-3490'></a> Section 5.6 - `accept()` - "Thank you for calling port 3490."

[md-accept](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L578) -> [accept.rs](./src/syscall/accept.rs)


### <a id='section-57-send-and-recv-talk-to-me-baby'></a> Section 5.7 - `send() and recv()` - Talk to me, baby!

[md-send](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L662) -> [send.rs](./src/syscall/send.rs)

[md-recv-synopsis](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L687) -> [recv.rs](./src/syscall/recv.rs)


### <a id='section-58-sendto-and-recvfrom-talk-to-me-dgram-style'></a> Section 5.8 - `sendto() and recvfrom()` - Talk to me, DGRAM-style

[md-sendto-synopsis](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L717) -> [sendto.rs](./src/syscall/sendto.rs)

[md-recvfrom-synopsis](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L741) -> [recvfrom.rs](./src/syscall/recvfrom.rs)

### <a id='section-59-close-and-shutdown-get-outta-my-face'></a> Section 5.9 - `close() and shutdown()` - Get outta my face!

[md-close-synopsis](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L783) -> [close.rs](./src/syscall/close.rs)

[md-shutdown-synopsis](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L796) -> [shutdown.rs](./src/syscall/shutdown.rs)

### <a id='section-510-getpeername-who-are-you'></a> Section 5.10 - `getpeername()` - Who are you?

[md-getpeername-synopsis](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L838) -> [getpeername.rs](./src/syscall/getpeername.rs)


### <a id='section-511-gethostname-who-am-i'></a> Section 5.11 - `gethostname()` - Who am I?

[md-gethostname-synopsis](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0500_syscalls.md?plain=1#L872) -> [gethostname.rs](./src/syscall/gethostname.rs)


### <a id='section-61-62-a-simple-stream-server-client'></a> Section 6.1 & 6.2 - A Simple Stream Server & Client

[server.c](https://github.com/beejjorgensen/bgnet/blob/main/source/examples/server.c) -> [server.rs](./src/stream/server.rs)

[client.c](https://github.com/beejjorgensen/bgnet/blob/main/source/examples/client.c) -> [client.rs](./src/stream/client.rs)

### <a id='section-63-datagram-sockets'></a> Section 6.3 - Datagram Sockets

[listener.c](https://github.com/beejjorgensen/bgnet/blob/main/source/examples/listener.c) -> [server.rs](./src/dgram/server.rs)

[talker.c](https://github.com/beejjorgensen/bgnet/blob/main/source/examples/talker.c) -> [client.rs](./src/dgram/client.rs)

### <a id='section-71-blocking'></a> Section 7.1 - Blocking

[md-blocking](https://github.com/beejjorgensen/bgnet/blob/0b0f028a51ba5eea738c175c170ef52312c77d65/src/bgnet_part_0700_advanced.md?plain=1#L30) -> [blocking.rs](./src/techniques/blocking.rs)

### <a id='section-72-poll-synchronous-io-multiplexing'></a> Section 7.2 - `poll()` - Synchronous I/O Multiplexing

[poll.c](https://github.com/beejjorgensen/bgnet/blob/main/source/examples/poll.c) -> [poll.rs](./src/techniques/poll.rs)

[pollserver.c](https://github.com/beejjorgensen/bgnet/blob/main/source/examples/pollserver.c) -> [pollserver.rs](./src/techniques/pollserver.rs)

### <a id='section-73-select-synchronous-io-multiplexing-old-school'></a> Section 7.3 - `select()` - Synchronous I/O Multiplexing, Old School

[select.c](https://github.com/beejjorgensen/bgnet/blob/main/source/examples/select.c) -> [select.rs](./src/techniques/select.rs)

[selectserver.c](https://github.com/beejjorgensen/bgnet/blob/main/source/examples/selectserver.c) -> [selectserver.rs](./src/techniques/select.rs)

### <a id='section-77-broadcast-packets-hello-world'></a> Section 7.7 - Broadcast Packets - Hello, World!

[broadcaster.c](https://github.com/beejjorgensen/bgnet/blob/main/source/examples/broadcaster.c) -> [broadcaster.rs](./src/techniques/broadcaster.rs)

## <a id='notes'></a> Notes

If you check the very first example and compare it to the last ones, you will see that there are quite a bit differences regarding how the unsafe operations are executed, such as:

- The contents of `unsafe` blocks,
- The contents of `SAFETY` comments,
- The countless casts done between pointers and types themselves.

This results in an inconsistency between the examples, and I intentionally kept this fact.
It shows how unsafe Rust effects the thought process the more you write, which is pleasant to reveal.
