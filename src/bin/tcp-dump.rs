// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(clippy::all)]

//==============================================================================
// Imports
//==============================================================================

use ::anyhow::{bail, Result};
use ::clap::{Arg, ArgMatches, Command};
use ::demikernel::{Ipv4Addr, Ipv4Endpoint, LibOS, OperationResult, Port16, QDesc, QToken};
use ::std::{
    num::NonZeroU16,
    time::{Duration, Instant},
};

//==============================================================================
// Program Arguments
//==============================================================================

/// Program Arguments
#[derive(Debug)]
pub struct ProgramArguments {
    /// Local IPv4 address.
    local_addr: Ipv4Addr,
    /// Local port number.
    local_port: Port16,
}

/// Associate functions for Program Arguments
impl ProgramArguments {
    /// Default local address.
    const DEFAULT_LOCAL_ADDR: &'static str = "127.0.0.1";

    /// Default local port.
    const DEFAULT_LOCAL_PORT: u16 = 12345;

    /// Parses the program arguments from the command line interface.
    pub fn new(app_name: &str, app_author: &str, app_about: &str) -> Result<Self> {
        let matches: ArgMatches = Command::new(app_name)
            .author(app_author)
            .about(app_about)
            .arg(
                Arg::new("local")
                    .long("local")
                    .takes_value(true)
                    .required(false)
                    .value_name("ADDRESS:PORT")
                    .help("Sets local address"),
            )
            .get_matches();

        // Default arguments.
        let mut args: ProgramArguments = ProgramArguments {
            local_addr: Self::DEFAULT_LOCAL_ADDR
                .parse()
                .expect("invalid local ipv4 address"),
            local_port: Port16::new(
                NonZeroU16::new(Self::DEFAULT_LOCAL_PORT).expect("invalid local port number"),
            ),
        };

        // Local address.
        if let Some(addr) = matches.value_of("local") {
            args.set_local_addr(addr)?;
        }

        Ok(args)
    }

    /// Returns the local endpoint address parameter stored in the target program arguments.
    pub fn get_local(&self) -> Ipv4Endpoint {
        Ipv4Endpoint::new(self.local_addr, self.local_port)
    }

    /// Parses an address string.
    fn parse_addr(addr: &str) -> Result<(Ipv4Addr, Port16)> {
        let tokens: Vec<&str> = addr.split(":").collect();
        if tokens.len() != 2 {
            bail!("invalid address")
        }
        let addr: Ipv4Addr = tokens[0].parse().expect("invalid ipv4 address");
        let portnum: u16 = tokens[1].parse().expect("invalid port number");
        let port: Port16 = Port16::new(NonZeroU16::new(portnum).expect("invalid port nubmer"));
        Ok((addr, port))
    }

    /// Sets the local address and port number parameters in the target program arguments.
    fn set_local_addr(&mut self, addr: &str) -> Result<()> {
        match Self::parse_addr(addr) {
            Ok((addr, port)) => {
                self.local_addr = addr;
                self.local_port = port;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

//==============================================================================
// Application
//==============================================================================

/// Application
struct Application {
    /// Underlying libOS.
    libos: LibOS,
    // Local socket descriptor.
    sockqd: QDesc,
}

/// Associated Functions for the Application
impl Application {
    /// Logging interval (in seconds).
    const LOG_INTERVAL: u64 = 5;

    /// Instantiates the application.
    pub fn new(mut libos: LibOS, args: &ProgramArguments) -> Self {
        // Extract arguments.
        let local: Ipv4Endpoint = args.get_local();

        // Create TCP socket.
        let sockqd: QDesc = match libos.socket(libc::AF_INET, libc::SOCK_STREAM, 0) {
            Ok(qd) => qd,
            Err(e) => panic!("failed to create socket: {:?}", e.cause),
        };

        // Bind to local address.
        match libos.bind(sockqd, local) {
            Ok(()) => (),
            Err(e) => panic!("failed to bind socket: {:?}", e.cause),
        };

        // Mark socket as a passive one.
        match libos.listen(sockqd, 16) {
            Ok(()) => (),
            Err(e) => panic!("failed to listen socket: {:?}", e.cause),
        }

        println!("Local Address: {:?}", local);

        Self { libos, sockqd }
    }

    /// Runs the target echo server.
    pub fn run(&mut self) -> ! {
        let start: Instant = Instant::now();
        let mut nbytes: usize = 0;
        let mut last_log: Instant = Instant::now();

        loop {
            // Dump statistics.
            if last_log.elapsed() > Duration::from_secs(Self::LOG_INTERVAL) {
                let elapsed: Duration = Instant::now() - start;
                println!("{:?} B / {:?} us", nbytes, elapsed.as_micros());
                last_log = Instant::now();
            }

            // Drain packets.
            let qt: QToken = match self.libos.pop(self.sockqd) {
                Ok(qt) => qt,
                Err(e) => panic!("failed to pop data from socket: {:?}", e.cause),
            };
            match self.libos.wait2(qt) {
                Ok((_, OperationResult::Pop(_, buf))) => {
                    nbytes += buf.len();
                }
                Err(e) => panic!("operation failed: {:?}", e.cause),
                _ => panic!("unexpected result"),
            }
        }
    }
}

//==============================================================================

fn main() -> Result<()> {
    let args: ProgramArguments = ProgramArguments::new(
        "tcp-dump",
        "Pedro Henrique Penna <ppenna@microsoft.com>",
        "Dumps incoming packets on a TCP port.",
    )?;

    let libos: LibOS = LibOS::new();

    Application::new(libos, &args).run();
}
