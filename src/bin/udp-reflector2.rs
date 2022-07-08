#![feature(once_cell)]
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(clippy::all)]

//==============================================================================
// External Crates
//==============================================================================
extern crate core_affinity;

//==============================================================================
// Imports
//==============================================================================

use ::anyhow::Result;
use ::clap::{Arg, ArgMatches, Command};
use ::demikernel::{LibOS, OperationResult, QDesc, QToken};
use ::std::{net::SocketAddrV4, str::FromStr};
use ::std::thread;
use core_affinity::CoreId;

//==============================================================================
// Program Arguments
//==============================================================================

/// Program Arguments
#[derive(Debug, Clone)]
pub struct ProgramArguments {
    /// Local socket IPv4 address.
    local: SocketAddrV4,
    /// Remote socket IPv4 address.
    remote: SocketAddrV4,
    /// Number of cores.
    cores: u32,
}

/// Associate functions for Program Arguments
impl ProgramArguments {
    /// Default local address.
    const DEFAULT_LOCAL: &'static str = "127.0.0.1:12345";

    /// Default host address.
    const DEFAULT_REMOTE: &'static str = "127.0.0.1:23456";

    /// Default number of cores
    const DEFAULT_CORES: u32 = 1;

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
                Arg::new("cores")
                    .long("cores")
                    .takes_value(true)
                    .required(true)
                    .value_name("CORES")
                    .help("Sets the number of cores"),
            )
            .get_matches();

        // Default arguments.
        let mut args: ProgramArguments = ProgramArguments {
            local: SocketAddrV4::from_str(Self::DEFAULT_LOCAL)?,
            remote: SocketAddrV4::from_str(Self::DEFAULT_REMOTE)?,
            cores: Self::DEFAULT_CORES,
        };

        // Local address.
        if let Some(addr) = matches.value_of("local") {
            args.set_local_addr(addr)?;
        }

        // Remote address.
        if let Some(addr) = matches.value_of("remote") {
            args.set_remote_addr(addr)?;
        }

        // Cores.
        if let Some(cores) = matches.value_of("cores") {
            args.set_cores(cores.parse().unwrap())?;
        }

        Ok(args)
    }

    /// Returns the local endpoint address parameter stored in the target program arguments.
    pub fn get_local(&self) -> SocketAddrV4 {
        self.local
    }

    /// Returns the remote endpoint address parameter stored in the target program arguments.
    pub fn get_remote(&self) -> SocketAddrV4 {
        self.remote
    }

    /// Returns the number of cores in the target program arguments.
    pub fn get_cores(&self) -> u32 {
        self.cores
    }

    /// Sets the local address and port number parameters in the target program arguments.
    fn set_local_addr(&mut self, addr: &str) -> Result<()> {
        self.local = SocketAddrV4::from_str(addr)?;
        Ok(())
    }

    /// Sets the remote address and port number parameters in the target program arguments.
    fn set_remote_addr(&mut self, addr: &str) -> Result<()> {
        self.remote = SocketAddrV4::from_str(addr)?;
        Ok(())
    }

    /// Sets the number of cores in the target program arguments.
    fn set_cores(&mut self, cores: u32) -> Result<()> {
        self.cores = cores;
        Ok(())
    }
}

//==============================================================================
// Application
//==============================================================================

/// Application
struct Application {
    /// Underlying libOS.
    libos: LibOS,
    /// Local socket descriptor.
    sockqd: QDesc,
}

/// Associated Functions for the Application
impl Application {
    /// Instantiates the application.
    pub fn new(mut libos: LibOS, args: &ProgramArguments) -> Self {
        // Extract arguments.
        let local: SocketAddrV4 = args.get_local();

        // Create UDP socket.
        let sockqd: QDesc = match libos.socket(libc::AF_INET, libc::SOCK_DGRAM, 0) {
            Ok(qd) => qd,
            Err(e) => panic!("failed to create socket: {:?}", e.cause),
        };

        // Bind to local address.
        match libos.bind(sockqd, local) {
            Ok(()) => (),
            Err(e) => panic!("failed to bind socket: {:?}", e.cause),
        };

        Self {
            libos,
            sockqd,
        }
    }

    /// Runs the target echo server.
    pub fn run(&mut self) -> ! { 
        let mut qtokens: Vec<QToken> = Vec::new();

        loop {
            let qt: QToken = match self.libos.pop(self.sockqd) {
                Ok(qt) => qt,
                Err(e) => panic!("failed to pop data from socket: {:?}", e.cause),
            };
            qtokens.push(qt);

            let (i, _qd, result) = match self.libos.wait_any2(&qtokens) {
                Ok((i, qd, result)) => (i, qd, result),
                Err(e) => panic!("operation failed: {:?}", e),
            };
            qtokens.swap_remove(i);

            match result {
                OperationResult::Pop(remote, buf) => {
                    let qt: QToken = match self.libos.pushto2(self.sockqd, &buf, remote.unwrap()) {
                        Ok(qt) => qt,
                        Err(e) => panic!("failed to push data to socket: {:?}", e.cause),
                    };
                    match self.libos.wait(qt) {
                        Ok(_) => (),
                        Err(e) => panic!("operation failed: {:?}", e.cause),
                    };
                }
                OperationResult::Failed(e) => panic!("operation failed: {:?}", e),
                _ => panic!("unexpected result"),
            };
        }
    }
}

//==============================================================================

fn main() -> Result<()> {
    let args0: ProgramArguments = ProgramArguments::new(
        "udp-reflector2",
        "Fabricio Carvalho <fabricio.carvalho@ufms.br>",
        "Echoes UDP packets using multiples cores."
    )?;

    let pool: Vec<_> = (0..args0.get_cores()).map(|i| {
        let args = args0.clone();
        thread::spawn(move || {
            core_affinity::set_for_current( CoreId { id: (2*(i+2)) as usize });
            let libos: LibOS = LibOS::new(i as u16, args.get_cores() as u16);
            Application::new(libos, &args).run();
        })
    }).collect();

    for handle in pool {
        handle.join().unwrap();
    }

    Ok(())
}