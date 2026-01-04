use super::Worker;
use std::io::Read;
use std::os::unix::process::CommandExt; // Per process_group
use std::process::{self, Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

// --- MESSAGGI ---
pub enum SensorCommand {
    Stop,
    Start,
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
        if let Some(child) = self.child.as_mut() {
            let id = child.id();
            // SIGINT al process group
            let _ = process::Command::new("kill")
                .args(["-2", &format!("-{}", id)])
                .output();

            thread::sleep(Duration::from_secs(2));

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
            println!("Exit status: {:?}", exit_status);
        }
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

        let mut buffer = [0u8; 1024];

        loop {
            // 1. Check non bloccante dei comandi
            match self.cmd_rx.try_recv() {
                Ok(SensorCommand::Start) => {
                    let SensorConfig { name, delay_ms } = &self.sensor_config;
                    self.child = Some(termux_sensor_command(name, *delay_ms));
                }
                Ok(SensorCommand::Stop) | Err(TryRecvError::Disconnected) => {
                    println!("SensorWorker: Stop command received.");
                    break;
                }
                _ => {} // Continue
            }

            // 2. Lettura bloccante (nota: read può bloccare, ma termux-sensor streamma)
            // In un sistema reale useremmo `polling` o `mio` per rendere read non bloccante,
            // ma per semplicità qui va bene.
            // Prendiamo l'ownership dello stdout del figlio
            if let Some(stdout) = &mut self.child
                && let Some(mut stdout) = stdout.stdout.take()
            {
                match stdout.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let data = buffer[..n].to_vec();
                        // Inviamo al display. Se fallisce, il display è morto.
                        if self.output_tx.send(data).is_err() {
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
