use ::anyhow::{bail, Result};
use ::clap::{Arg, ArgMatches, Command};
use ::demikernel::{LibOS, OperationResult, QDesc, QToken};
use ::std::net::SocketAddrV4;
use ::std::str::FromStr;
use ::std::time::{Duration, Instant};
use std::collections::HashMap;

pub const DEFAULT_KEYSIZE: usize = 64;
pub const DEFAULT_VALSIZE: usize = 4096;
pub const DEFAULT_NUMKEYS: usize =  32768;

#[derive(Debug)]
struct ProgramArguments {
    server: Option<SocketAddrV4>,
    keysize: u64,
    valsize: u64,
    numkeys: u64,
    peer_type: String,
}

/// Associate functions for Program Arguments
impl ProgramArguments {
    pub fn new(app_name: &str, app_author: &str, app_about: &str) -> Result<Self> {
        let matches: ArgMatches = Command::new(app_name)
        .author(app_author)
        .about(app_about)
        .arg(
            Arg::new("server")
                .long("server")
                .takes_value(true)
                .required(true)
                .value_name("ADDRESS:PORT")
                .help("Sets server address"),
        )
        .arg(
            Arg::new("keysize")
                .long("keysize")
                .takes_value(true)
                .required(true)
                .value_name("SIZE")
                .help("Sets key size"),
        )
        .arg(
            Arg::new("valuesize")
                .long("valuesize")
                .takes_value(true)
                .required(true)
                .value_name("SIZE")
                .help("Sets value size"),
        )
        .arg(
            Arg::new("numkeys")
                .long("numkeys")
                .takes_value(true)
                .required(true)
                .value_name("NUM")
                .help("Sets number of keys"),
        )
        .get_matches();

        let mut args: ProgramArguments = ProgramArguments {
            server: None,
            keysize: DEFAULT_KEYSIZE,
            valsize: DEFAULT_VALSIZE,
            numkeys: DEFAULT_NUMKEYS,
            peer_type: "server".to_string(),
        };

        if let Some(addr) = matches.value_of("server") {
            args.set_server(addr)?;
        }

        if let Some(keysize) = matches.value_of("keysize") {
            args.set_keysize(keysize)?;
        }

        if let Some(valsize) = matches.value_of("valsize") {
            args.set_valsize(valsize)?;
        }

        if let Some(numkeys) = matches.value_of("numkeys") {
            args.set_numkeys(numkeys)?;
        }

        if let Some(peer_type) = matches.value_of("peer") {
            args.set_peer_type(peer_type.to_string())?;
        }

        Ok(args)
    }

    pub fn get_server(&self) -> Option<SocketAddrV4> {
        self.server
    }

    pub fn get_keysize(&self) -> u64 {
        self.keysize
    }

    pub fn get_valsize(&self) -> u64 {
        self.valsize
    }

    pub fn get_numkeys(&self) -> u64 {
        self.numkeys
    }

    pub fn get_peer_type(&self) -> String {
        self.peer_type.to_string()
    }

    fn set_server(&mut self, addr: &str) -> Result<()> {
        self.server = Some(SocketAddrV4::from_str(addr)?);
        Ok(())
    }
    
    fn set_keysize(&mut self, keysize_str: &str) ->  Result<()> {
        let keysize: usize = keysize_str.parse()?;
        if keysize > 0 {
            self.keysize = keysize;
            Ok(())
        } else {
            bail!("invalid key size")
        }
    }

    fn set_valsize(&mut self, valsize_str: &str) ->  Result<()> {
        let valsize: usize = valsize_str.parse()?;
        if valsize > 0 {
            self.valsize = valsize;
            Ok(())
        } else {
            bail!("invalid value size")
        }
    }

    fn set_numkeys(&mut self, numkeys_str: &str) ->  Result<()> {
        let numkeys: usize = numkeys_str.parse()?;
        if numkeys > 0 {
            self.numkeys = numkeys;
            Ok(())
        } else {
            bail!("invalid number of keys")
        }
    }

    fn set_peer_type(&mut self, peer_type: String) -> Result<()> {
        if peer_type != "server" && peer_type != "client" {
            bail!("invalid peer type")
        } else {
            self.peer_type = peer_type;
            Ok(())
        }
    }
    
}

/*
 * KVStore is implemented as a 2-layer hash table. In theory, this will allow
 * for future registration of the overall hash table structure in smaller chunks.
 * The interface to the hash table is still the same. The user provides a key. The
 * key is first used in the outer hash table to find the inner hash table, and then
 * the same key is used to get the resulting value.
 */
struct KVStore {
    store: HashMap<String, HashMap<String, String>>
}

impl KVStore {
    pub fn new() -> Result<Self> {
        let mut kvs: KVStore = KVStore { 
            store: HashMap::default(),
        };

        Ok(kvs)
    }

    pub fn get(&self, key: &String) -> Option<&String> {
        let temp_hashmap: Option<HashMap<String, HashMap<String, String>>> = self.store.get(key);
        if temp_hashmap.is_none() { 
            None 
        }
        let hashmap: HashMap<String, String> = temp_hashmap.unwrap();
        hashmap.get(key)
    }

    pub fn insert(&mut self, key: &String, value: &String) -> Option<(String)> {
        let inner: HashMap<String, String> = HashMap::new();
        inner.insert(key, value);
        self.store.insert(key, inner)
    }

    pub fn remove(&mut self, key: &String) -> Option<(String)> {
        let temp_hashmap: Option<HashMap<String, HashMap<String, String>>> = self.store.get(key);
        if temp_hashmap.is_none() { 
            None 
        }
        let hashmap: HashMap<String, String> = temp_hashmap.unwrap();
        hashmap.remove(key)
    }

    pub fn len(&self) -> usize {
        let length: usize = 0;
        for value in self.store.values() {
            length = length + value.len()
        }
        length
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }


}

impl Default for KVStore {
    fn default() -> Self {
        Self::new()
    }
}

struct KVApp {
    libos: LibOS,
    sockqd: QDesc,
    is_server: bool,
}

impl KVApp {
    pub fn new(libos: LibOS, args: &ProgramArguments) -> Result<Self> {
        let peer_type: String = args.get_peer_type();

        if peer_type == "server" {
            Self::new_server(libos, args)
        } else {
            Self::new_client(libos, args)
        }
    }

    fn is_server(&self) -> bool {
        self.is_server
    }

    fn new_server(mut libos: LibOS, args: &ProgramArguments) -> Result<Self> {
        if let Some(server) = args.get_server() {
            // Create TCP socket.
            let sockqd: QDesc = match libos.socket(libc::AF_INET, libc::SOCK_STREAM, 0) {
                Ok(qd) => qd,
                Err(e) => panic!("failed to create socket: {:?}", e.cause),
            };

            // Bind to local address.
            match libos.bind(sockqd, server) {
                Ok(()) => (),
                Err(e) => panic!("failed to bind socket: {:?}", e.cause),
            };

            // Mark socket as a passive one.
            match libos.listen(sockqd, 16) {
                Ok(()) => (),
                Err(e) => panic!("failed to listen socket: {:?}", e.cause),
            }

            println!("Local Address: {:?}", server);

            return Ok(Self {
                libos,
                sockqd,
                is_server: true,
            });
        }

        bail!("missing local address")
    }

    fn new_client(mut libos: LibOS, args: &ProgramArguments) -> Result<Self> {
        if let Some(server) = args.get_server() {
            // Create TCP socket.
            let sockqd: QDesc = match libos.socket(libc::AF_INET, libc::SOCK_STREAM, 0) {
                Ok(qd) => qd,
                Err(e) => panic!("failed to create socket: {:?}", e.cause),
            };

            // Setup connection.
            let qt: QToken = match libos.connect(sockqd, server) {
                Ok(qt) => qt,
                Err(e) => panic!("failed to connect socket: {:?}", e.cause),
            };
            match libos.wait2(qt) {
                Ok((_, OperationResult::Connect)) => println!("connected!"),
                Err(e) => panic!("operation failed: {:?}", e),
                _ => panic!("unexpected result"),
            }

            println!("Remote Address: {:?}", server);

            return Ok(Self {
                libos,
                sockqd,
                is_server: false,
            });
        };

        bail!("missing remote address")
    }

    fn run_server(&mut self, args: &ProgramArguments) -> ! {
        let kvstore: KVStore = KVStore::new();
        let mut qtokens: Vec<QToken> = Vec::new();

        // Accept first connection.
        let qt: QToken = match self.libos.accept(self.sockqd) {
            Ok(qt) => qt,
            Err(e) => panic!("failed to accept connection on socket: {:?}", e.cause),
        };
        qtokens.push(qt);

        loop {
            let (i, qd, result) = match self.libos.wait_any2(&qtokens) {
                Ok((i, qd, result)) => (i, qd, result),
                Err(e) => panic!("operation failed: {:?}", e),
            };
            qtokens.swap_remove(i);

            // Parse result.
            match result {
                OperationResult::Accept(qd) => {
                    println!("connection accepted!");
                    // Pop first packet.
                    let qt: QToken = match self.libos.pop(qd) {
                        Ok(qt) => qt,
                        Err(e) => panic!("failed to pop data from socket: {:?}", e.cause),
                    };
                    qtokens.push(qt);
                }
                // Pop completed.
                OperationResult::Pop(_, buf) => {
                    // TODO
                }
                // Push completed.
                OperationResult::Push => {
                    // TODO
                }
                OperationResult::Failed(e) => panic!("operation failed: {:?}", e),
                _ => panic!("unexpected result"),
            }
        }
    }

    fn run_client(&mut self) -> ! {
        let start: Instant = Instant::now();
        let mut nbytes: usize = 0;
        let mut last_log: Instant = Instant::now();

        loop {
            // TODO
        }
    }
}


fn main() -> Result<()> {
    let args: ProgramArguments = ProgramArguments::new(
        "kv-store",
        "Amanda Raybuck",
        "Demikernel Key-Value Store",
    )?;

    let libos: LibOS = LibOS::new();
    let mut app: Application = Application::new(libos, &args)?;

    if app.is_server() {
        app.run_server(args);
    } else {
        app.run_client(args);
    }

    Ok(())
}