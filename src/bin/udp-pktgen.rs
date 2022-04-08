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
    /// Remote address.
    remote_addr: Ipv4Addr,
    /// Remote port number.
    remote_port: Port16,
    /// Buffer size (in bytes).
    bufsize: usize,
    /// Injection rate (in micro-seconds).
    injection_rate: u64,
}

/// Associate functions for Program Arguments
impl ProgramArguments {
    /// Default local address.
    const DEFAULT_LOCAL_ADDR: &'static str = "127.0.0.1";

    /// Default local port.
    const DEFAULT_LOCAL_PORT: u16 = 12345;

    /// Default host address.
    const DEFAULT_REMOTE_ADDR: &'static str = "127.0.0.1";

    /// Default host port number.
    const DEFAULT_REMOTE_PORT: u16 = 23456;

    // Default buffer size.
    const DEFAULT_BUFSIZE: usize = 1024;

    // Default injection rate.
    const DEFAULT_INJECTION_RATE: u64 = 100;

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
            .arg(
                Arg::new("remote")
                    .long("remote")
                    .takes_value(true)
                    .required(true)
                    .value_name("ADDRESS:PORT")
                    .help("Sets remote address"),
            )
            .arg(
                Arg::new("bufsize")
                    .long("bufsize")
                    .takes_value(true)
                    .required(true)
                    .value_name("SIZE")
                    .help("Sets buffer size"),
            )
            .arg(
                Arg::new("injection_rate")
                    .long("injection_rate")
                    .takes_value(true)
                    .required(true)
                    .value_name("RATE")
                    .help("Sets packet injection rate"),
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
            remote_addr: Self::DEFAULT_REMOTE_ADDR
                .parse()
                .expect("invalid remote ipv4 address"),
            remote_port: Port16::new(
                NonZeroU16::new(Self::DEFAULT_REMOTE_PORT).expect("invalid remote port number"),
            ),
            bufsize: Self::DEFAULT_BUFSIZE,
            injection_rate: Self::DEFAULT_INJECTION_RATE,
        };

        // Local address.
        if let Some(addr) = matches.value_of("local") {
            args.set_local_addr(addr)?;
        }

        // Remote address.
        if let Some(addr) = matches.value_of("remote") {
            args.set_remote_addr(addr)?;
        }

        // Buffer size.
        if let Some(bufsize) = matches.value_of("bufsize") {
            args.set_bufsize(bufsize)?;
        }

        // Injection rate.
        if let Some(injection_rate) = matches.value_of("injection_rate") {
            args.set_injection_rate(injection_rate)?;
        }

        Ok(args)
    }

    /// Returns the local endpoint address parameter stored in the target program arguments.
    pub fn get_local(&self) -> Ipv4Endpoint {
        Ipv4Endpoint::new(self.local_addr, self.local_port)
    }

    /// Returns the remote endpoint address parameter stored in the target program arguments.
    pub fn get_remote(&self) -> Ipv4Endpoint {
        Ipv4Endpoint::new(self.remote_addr, self.remote_port)
    }

    /// Returns the buffer size parameter stored in the target program arguments.
    pub fn get_bufsize(&self) -> usize {
        self.bufsize
    }

    /// Returns the injection rate parameter stored in the target program arguments.
    pub fn get_injection_rate(&self) -> u64 {
        self.injection_rate
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

    /// Sets the remote address and port number parameters in the target program arguments.
    fn set_remote_addr(&mut self, addr: &str) -> Result<()> {
        match Self::parse_addr(addr) {
            Ok((addr, port)) => {
                self.remote_addr = addr;
                self.remote_port = port;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Sets the buffer size parameter in the target program arguments.
    fn set_bufsize(&mut self, bufsize_str: &str) -> Result<()> {
        let bufsize: usize = bufsize_str.parse()?;
        if bufsize > 0 {
            self.bufsize = bufsize;
            Ok(())
        } else {
            bail!("invalid buffer size")
        }
    }

    /// Sets the injection rate parameter in the target program arguments.
    fn set_injection_rate(&mut self, injection_rate_str: &str) -> Result<()> {
        let injection_rate: u64 = injection_rate_str.parse()?;
        if injection_rate > 0 {
            self.injection_rate = injection_rate;
            Ok(())
        } else {
            bail!("invalid injection rate")
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
    /// Remote endpoint.
    remote: Ipv4Endpoint,
    /// Buffer size.
    bufsize: usize,
    /// Injection rate
    injection_rate: u64,
}

/// Associated Functions for the Application
impl Application {
    /// Logging interval (in seconds).
    const LOG_INTERVAL: u64 = 5;

    /// Instantiates the application.
    pub fn new(mut libos: LibOS, args: &ProgramArguments) -> Self {
        // Extract arguments.
        let local: Ipv4Endpoint = args.get_local();
        let remote: Ipv4Endpoint = args.get_remote();
        let bufsize: usize = args.get_bufsize();
        let injection_rate: u64 = args.get_injection_rate();

        // Create UDP socket.
        let sockqd: QDesc = match libos.socket(libc::AF_INET, libc::SOCK_DGRAM, 1) {
            Ok(qd) => qd,
            Err(e) => panic!("failed to create socket: {:?}", e.cause),
        };

        // Bind to local address.
        match libos.bind(sockqd, local) {
            Ok(()) => (),
            Err(e) => panic!("failed to bind socket: {:?}", e.cause),
        };

        println!("Local Address:  {:?}", local);
        println!("Remote Address: {:?}", remote);

        Self {
            libos,
            sockqd,
            remote,
            bufsize,
            injection_rate,
        }
    }

    /// Runs the target application.
    pub fn run(&mut self) {
        let start: Instant = Instant::now();
        let mut nbytes: usize = 0;
        let mut last_push: Instant = Instant::now();
        let mut last_log: Instant = Instant::now();
        let data: Vec<u8> = Self::mkbuf(self.bufsize, 0x65);

        loop {
            // Dump statistics.
            if last_log.elapsed() > Duration::from_secs(Self::LOG_INTERVAL) {
                let elapsed: Duration = Instant::now() - start;
                println!("{:?} B / {:?} us", nbytes, elapsed.as_micros());
                last_log = Instant::now();
            }

            // Push packet.
            if last_push.elapsed() > Duration::from_micros(self.injection_rate) {
                let qt: QToken = match self.libos.pushto2(self.sockqd, &data, self.remote) {
                    Ok(qt) => qt,
                    Err(e) => panic!("failed to push data to socket: {:?}", e.cause),
                };
                match self.libos.wait2(qt) {
                    Ok((_, OperationResult::Push)) => (),
                    Err(e) => panic!("operation failed: {:?}", e.cause),
                    _ => panic!("unexpected result"),
                };
                nbytes += self.bufsize;
                last_push = Instant::now();
            }
        }
    }

    /// Makes a buffer.
    fn mkbuf(bufsize: usize, fill_char: u8) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::<u8>::with_capacity(bufsize);

        for _ in 0..bufsize {
            data.push(fill_char);
        }

        data
    }
}

//==============================================================================

/// Drives the application.
fn main() -> Result<()> {
    let args: ProgramArguments = ProgramArguments::new(
        "udp-pktgen",
        "Pedro Henrique Penna <ppenna@microsoft.com>",
        "Generates UDP traffic.",
    )?;

    let libos: LibOS = LibOS::new();

    Application::new(libos, &args).run();

    Ok(())
}
