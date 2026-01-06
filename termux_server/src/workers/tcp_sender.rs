use super::Worker;
use std::{
    io::Write,      // Necessario per scrivere nello stream
    net::TcpStream, // Per la connessione TCP
    sync::mpsc::{self, Receiver, Sender, channel},
    thread,
    time::Duration,
};

// --- INIT STATE ---
// Oltre al canale, passiamo l'indirizzo a cui connetterci
pub struct TcpSenderInitState {
    pub input_rx: Receiver<Vec<u8>>,
    pub target_addr: String, // es: "172.17.62.41:8080"
}

pub enum TcpMess {
    TryConnect,
    Suspend,
    Exit
}
enum ThreadState {
    Idle,
    Disconnected,
    WaitForConnection,
    Connected,
    Exit,
}

// --- CONTROLLER ---
pub struct TcpSenderController {
    pub cmd_tx: Sender<TcpMess>,
}

pub struct TcpConfig {
    target_addr: String,
}

// --- WORKER ---
pub struct TcpSenderWorker {
    state: ThreadState,
    cmd_rx: Receiver<TcpMess>,
    rx: Receiver<Vec<u8>>,
    stream: Option<TcpStream>, // Manteniamo la connessione aperta
    tcp: TcpConfig,
}

impl Worker for TcpSenderWorker {
    type InitState = TcpSenderInitState;
    type Controller = TcpSenderController;

    fn build(state: TcpSenderInitState) -> (Self, Self::Controller) {
        let (cmd_tx, cmd_rx) = mpsc::channel::<TcpMess>();

        (
            TcpSenderWorker {
                state: ThreadState::Idle,
                cmd_rx,
                rx: state.input_rx,
                stream: None,
                tcp: TcpConfig {
                    target_addr: state.target_addr,
                },
            },
            TcpSenderController { cmd_tx },
        )
    }

    fn run(mut self) {
        'ext_loop: loop {
            // Use recv_timeout to allow periodic state checks
            match self.cmd_rx.try_recv() {
                Ok(mess) => match mess {
                    TcpMess::TryConnect => {
                        self.state = ThreadState::WaitForConnection;
                        self.stream = None; // Clear any existing connection
                    }
                    TcpMess::Suspend => {
                        self.state = ThreadState::Idle;
                        self.stream = None; // Close connection on suspend
                    }
                    TcpMess::Exit => break 'ext_loop,
                },
                Err(mpsc::TryRecvError::Empty) => {
                    // No command, continue with current state
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    println!("TcpWorker: Command channel closed");
                    self.state = ThreadState::Exit;
                }
            }

            match self.state {
                ThreadState::Exit => break 'ext_loop,

                ThreadState::Idle => {
                    thread::sleep(Duration::from_millis(500));
                    continue;
                }

                ThreadState::WaitForConnection => {
                    self.stream = try_connect(&self.tcp.target_addr);
                    if self.stream.is_some() {
                        self.state = ThreadState::Connected;
                        println!("TcpSender: Connection success!");
                    } else {
                        // Stay in WaitForConnection, but add delay to avoid spinning
                        thread::sleep(Duration::from_secs(2));
                    }
                    continue;
                }

                ThreadState::Connected => {
                    if let Some(stream) = self.stream.as_mut() {
                        // Try to receive data with timeout
                        match self.rx.recv() {
                            Ok(data) => {
                                if let Err(e) = stream.write_all(&data) {
                                    eprintln!(
                                        "TcpSender Error: Connection interrupted during data sending: {}",
                                        e
                                    );
                                    self.stream = None;
                                    self.state = ThreadState::Disconnected;
                                }
                            }
                            Err(mpsc::RecvError) => {
                                println!("{}: Data channel closed", Self::name());
                                self.state = ThreadState::Exit;
                            }
                        }
                    } else {
                        // Connected state but no stream - should not happen
                        self.state = ThreadState::Disconnected;
                    }
                }

                ThreadState::Disconnected => {
                    // Attempt reconnection
                    self.stream = try_connect(&self.tcp.target_addr);
                    if self.stream.is_some() {
                        self.state = ThreadState::Connected;
                        println!("TcpSender: Reconnection success!");
                    } else {
                        thread::sleep(Duration::from_secs(2)); // Backoff on failure
                    }
                }
            }
        }

        println!("TcpSender: Terminated.");
    }

    fn name() -> String {
        String::from("TcpWorker")
    }
}
fn try_connect(target_addr: &str) -> Option<TcpStream> {
    println!("TcpSender: Attempting to connect to {}...", target_addr);
    match TcpStream::connect(target_addr) {
        Ok(stream) => Some(stream),
        Err(e) => {
            println!("Trying connecting to {}: {e}", target_addr);
            None
        }
    }
}

// only for debug
fn _vec_to_string(v: Vec<u8>) -> String {
    v.into_iter().map(|x| x as char).collect()
}
