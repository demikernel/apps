// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(clippy::all)]

//==============================================================================
// Imports
//==============================================================================

use ::anyhow::{bail, Result};
use ::clap::{Arg, ArgMatches, Command};
use ::demikernel::{LibOS, OperationResult, QDesc, QToken};
use ::std::time::{Duration, Instant};
use ::std::{net::SocketAddrV4, str::FromStr};
use rand::Rng;
use histogram::Histogram;
use std::convert::TryInto;
use std::fs::File;
use std::io::Write;

//==============================================================================
// Program Arguments
//==============================================================================

/// Program Arguments
#[derive(Debug)]
pub struct ProgramArguments {
    /// Local socket IPv4 address.
    local: SocketAddrV4,
    /// Remote socket IPv4 address.
    remote: SocketAddrV4,
    /// Buffer size (in bytes).
    bufsize: usize,
    /// Number of flows.
    flows: u16,
    /// Number of packets (x 10**6)
    packets: u64,
}

/// Associate functions for Program Arguments
impl ProgramArguments {
    /// Default local address.
    const DEFAULT_LOCAL: &'static str = "127.0.0.1:12345";

    /// Default host address.
    const DEFAULT_REMOTE: &'static str = "127.0.0.1:23456";

    // Default buffer size.
    const DEFAULT_BUFSIZE: usize = 1024;

    // Default number of flows
    const FLOWS: u16 = 1;

    // Default number of packets (x 10**6)
    const PACKETS: u64 = 1;

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
                Arg::new("flows")
                    .long("flows")
                    .takes_value(true)
                    .required(false)
                    .value_name("FLOWS")
                    .help("Set the number of flows"),
            )
            .arg(
                Arg::new("packets")
                    .long("packets")
                    .takes_value(true)
                    .required(false)
                    .value_name("PACKETS")
                    .help("Set the number of packets"),
            ) 
            .get_matches();

        // Default arguments.
        let mut args: ProgramArguments = ProgramArguments {
            local: SocketAddrV4::from_str(Self::DEFAULT_LOCAL)?,
            remote: SocketAddrV4::from_str(Self::DEFAULT_REMOTE)?,
            bufsize: Self::DEFAULT_BUFSIZE,
            flows: Self::FLOWS,
            packets: Self::PACKETS,
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

        // Flows
        if let Some(flows) = matches.value_of("flows") {
            args.set_flows(flows)?;
        }

        // Packets
        if let Some(packets) = matches.value_of("packets") {
            args.set_packets(packets)?;
        }

        Ok(args)
    }

    /// Returns the local endpoint address parameter stored in the target program arguments.
    pub fn get_local(&self) -> SocketAddrV4 {
        self.local
    }
    
    /// Returns the first local port parameter stored in the target program arguments.
    pub fn get_first_port(&self) -> u16 {
        self.local.port()
    }

    /// Returns the remote endpoint address parameter stored in the target program arguments.
    pub fn get_remote(&self) -> SocketAddrV4 {
        self.remote
    }

    /// Returns the buffer size parameter stored in the target program arguments.
    pub fn get_bufsize(&self) -> usize {
        self.bufsize
    }

    /// Returns the number of flows stored in the target program arguments.
    pub fn get_flows(&self) -> u16 {
        self.flows
    }

    /// Returns the number of packets (x 10**6) stored in the target program arguments.
    pub fn get_packets(&self) -> u64 {
        // self.packets * 1000000
        self.packets * 10000
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

    /// Sets the number of flows parameter in the target program arguments.
    fn set_flows(&mut self, flows_str: &str) -> Result<()> {
        let flows: u16 = flows_str.parse()?;
        if flows > 0 {
            self.flows = flows;
            Ok(())
        } else {
            bail!("invalid number of flows")
        }
    }

    /// Sets the number of packets parameter in the target program arguments.
    fn set_packets(&mut self, packets_str: &str) -> Result<()> {
        let packets: u64 = packets_str.parse()?;
        if packets > 0 {
            self.packets = packets;
            Ok(())
        } else {
            bail!("invalid number of packets")
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
    /// Array of local socket descriptor
    sockets: Vec<QDesc>,
    /// Remote endpoint.
    remote: SocketAddrV4,
    /// Buffer size.
    bufsize: usize,
    /// Number of packets (x 10**6)
    packets: u64,
}

/// Associated Functions for the Application
impl Application {
    /// Instantiates the application.
    pub fn new(mut libos: LibOS, args: &ProgramArguments) -> Self {
        // Extract arguments.
        let remote: SocketAddrV4 = args.get_remote();
        let bufsize: usize = args.get_bufsize();
        let flows: u16 = args.get_flows();
        let mut sockets : Vec<QDesc> = Vec::new();
        let packets: u64 = args.get_packets();

        for i in 0..flows {
            // Create UDP socket.
            let sockqd: QDesc = match libos.socket(libc::AF_INET, libc::SOCK_DGRAM, 1) {
                Ok(qd) => qd,
                Err(e) => panic!("failed to create socket: {:?}", e.cause),
            };

            // Create local address
            let local: SocketAddrV4 = SocketAddrV4::new(*args.get_local().ip(), i);
        
            // Bind to local address.
            match libos.bind(sockqd, local) {
                Ok(()) => (),
                Err(e) => panic!("failed to bind socket: {:?}", e.cause),
            };

	        sockets.push(sockqd);
            println!("Local Address:  {:?}", local);
        }

        println!("Remote Address: {:?}", remote);

        Self {
            libos,
            sockets,
            remote,
            bufsize,
            packets,
        }
    }

    /// Runs the target application.
    pub fn run(&mut self) {
        let mut qtokens: Vec<QToken> = Vec::new();
        let mut latencies: Vec<u64> = Vec::new();
        let mut npackets: u64 = 0;
	    let mut last_push: Instant = Instant::now();
        let data: Vec<u8> = Self::mkbuf(self.bufsize, 0x65);

        // Push the first packet
        let mut idx: usize = rand::thread_rng().gen_range(0, (self.sockets.len()).into());
        let qt: QToken = match self.libos.pushto2(self.sockets[idx], &data, self.remote) {
            Ok(qt) => qt,
            Err(e) => panic!("failed to push data to socket: {:?}", e.cause),
        };

        last_push = Instant::now();
        qtokens.push(qt);

        loop {
            if npackets == self.packets {
                let mut histogram = Histogram::new();
                let mut file = File::create("latency.txt").unwrap();
                for i in latencies {
                    histogram.increment(i);
                    writeln!(&mut file, "{}", i).expect("Could not write to file");
                }

                println!("Percentiles: p50: {} ns p90: {} ns p99: {} ns p999: {}",
                    histogram.percentile(50.0).unwrap(),
                    histogram.percentile(90.0).unwrap(),
                    histogram.percentile(99.0).unwrap(),
                    histogram.percentile(99.9).unwrap(),
                );

                println!("Latency (ns): Min: {} Avg: {} Max: {} StdDev: {}",
                    histogram.minimum().unwrap(),
                    histogram.mean().unwrap(),
                    histogram.maximum().unwrap(),
                    histogram.stddev().unwrap(),
                );

                break;
            }

            let (i, _, result) = match self.libos.wait_any2(&qtokens) {
                Ok((i, qd, result)) => (i, qd, result),
                Err(e) => panic!("operation failed: {:?}", e),
            };
            qtokens.swap_remove(i);

            // Parse result
            match result {
                // Push completed.
                OperationResult::Push => {
                    let qt: QToken = match self.libos.pop(self.sockets[idx]) {
                        Ok(qt) => qt,
                        Err(e) => panic!("failed to pop data from socket: {:?}", e.cause),
                    };
                    qtokens.push(qt);
                }
                OperationResult::Pop(_, buf) => {
                    let start: Instant = Instant::now();
                    let roundtrip: Duration = start - last_push;
                    latencies.push(roundtrip.as_nanos().try_into().unwrap());

                    idx = rand::thread_rng().gen_range(0, (self.sockets.len()).into());
                    let qt: QToken = match self.libos.pushto2(self.sockets[idx], &buf, self.remote) {
                        Ok(qt) => qt,
                        Err(e) => panic!("failed to push data to socket: {:?}", e.cause),
                    };
                    last_push = Instant::now();
                    qtokens.push(qt);
                    npackets += 1;
                }
                OperationResult::Failed(e) => panic!("operation failed: {:?}", e),
                _ => panic!("unexpected result"),
            };
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
        "udp-latency",
        "Fabricio Carvalho <fabricio.carvalho@ufms.br>",
        "Measures latency using UDP packets.",
    )?;

    let libos: LibOS = LibOS::new(0,1);

    Application::new(libos, &args).run();

    Ok(())
}
