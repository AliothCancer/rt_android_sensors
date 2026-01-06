use super::Worker;
use std::io::Read;
use std::os::unix::process::CommandExt; // Per process_group
use std::process::{self, Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

// --- MESSAGGI ---
pub enum SensorCommand {
    Start,
    Stop,
    Exit,
}

// --- INIT STATE (ex Config) ---
// Definisce lo stato iniziale per la creazione.
// Include il canale dove spedire i dati (output_tx).
pub struct SensorInitState {
    pub name: String,
    pub delay_ms: u64,
    pub output_tx: Sender<Vec<u8>>, // Il tubo dove buttare i dati
}

// --- CONTROLLER ---
pub struct SensorController {
    pub cmd_tx: Sender<SensorCommand>,
}

// --- WORKER (CONTEXT) ---
pub struct SensorWorker {
    child: Option<Child>, // POSSIEDE il processo. Niente Arc<Mutex>!
    cmd_rx: Receiver<SensorCommand>,
    output_tx: Sender<Vec<u8>>,
    sensor_config: SensorConfig,
}
pub struct SensorConfig {
    name: String,
    delay_ms: u64,
}
// Implementiamo Drop per garantire la pulizia automatica del processo
impl Drop for SensorWorker {
    fn drop(&mut self) {
        kill_child(self.child.as_mut());
    }
}
fn kill_child(child: Option<&mut Child>) {
    if let Some(child) = child {
        send_sig_int_term_kill_wait(child);
    } else {
        println!("SensorWorker: Child is None, no need to kill")
    }
}

impl Worker for SensorWorker {
    type InitState = SensorInitState;
    type Controller = SensorController;

    fn build(state: SensorInitState) -> (Self, Self::Controller) {
        // 2. Canale di comando interno
        let (cmd_tx, cmd_rx) = mpsc::channel();

        let worker = SensorWorker {
            child: None,
            cmd_rx,
            output_tx: state.output_tx,
            sensor_config: SensorConfig {
                name: state.name,
                delay_ms: state.delay_ms,
            },
        };

        let controller = SensorController { cmd_tx };

        (worker, controller)
    }

    fn run(mut self) {
        println!("SensorWorker: Started.");

        let mut buffer = [0u8; 4096];

        loop {
            // 1. Check non bloccante dei comandi
            match self.cmd_rx.try_recv() {
                Ok(SensorCommand::Start) => {
                    let SensorConfig { name, delay_ms } = &self.sensor_config;
                    self.child = Some(termux_sensor_command(name, *delay_ms));
                }
                Ok(SensorCommand::Stop) => kill_child(self.child.as_mut()),
                Ok(SensorCommand::Exit) | Err(TryRecvError::Disconnected) => {
                    println!("SensorWorker: Exit command received.");
                    break;
                }
                _ => match self.child {
                    Some(_) => (),
                    None => {
                        println!("SensorWorker is running but child is None");
                        thread::sleep(Duration::from_secs(3));
                    },
                }, // Continue
            }

            // 2. Lettura bloccante (nota: read può bloccare, ma termux-sensor streamma)
            // In un sistema reale useremmo `polling` o `mio` per rendere read non bloccante,
            // ma per semplicità qui va bene.
            // Prendiamo l'ownership dello stdout del figlio
            if let Some(stdout) = self.child.as_mut()
                && let Some(stdout) = stdout.stdout.as_mut()
            {
                match stdout.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let data = buffer[..n].to_vec();
                        // Inviamo al display. Se fallisce, il display è morto.
                        if self.output_tx.send(data).is_err() {
                            println!("SensorWorker: Failed to send data to TcpWorker");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("SensorWorker Error: {e}");
                        break;
                    }
                }
            };
        }
        // Quando usciamo dal loop, `self` viene droppato e `Drop` uccide il processo.
    }
    
    fn name() -> String {
        String::from("SensorWorker")
    }
}

fn termux_sensor_command(name: &str, delay_ms: u64) -> Child {
    Command::new("termux-sensor")
        .args(["-s", name])
        .args(["-d", &delay_ms.to_string()])
        .stdout(Stdio::piped()) // Catturiamo stdout
        .process_group(0) // Importante per i segnali
        .spawn()
        .expect("Failed to spawn termux-sensor")
}

/// Send SIGINT to the process group ID to make it propagate to all childs
/// and release the sensor resource. Then send SIGTERM and finally SIGKILL if needed.
fn send_sig_int_term_kill_wait(child: &mut Child) {
    let id = child.id();

    println!("SensorWorker: Siginting the child");
    // SIGINT al process group
    let _ = process::Command::new("kill")
        .args(["-2", &format!("-{}", id)])
        .output();

    thread::sleep(Duration::from_secs(2));

    println!("SensorWorker: Sigterminating the child");
    // SIGTERM al process group
    let _ = process::Command::new("kill")
        .args([&format!("-{}", id)])
        .output();

    thread::sleep(Duration::from_secs(2));

    // SIGKILL
    if let Err(e) = child.kill() {
        println!("Tried SIGTERM but wasn't enough, so SIGKILL-ed it: {e}");
    }

    let exit_status = child.wait();
    println!("SensorWorker Child: Exit status: {:?}", exit_status);
}
