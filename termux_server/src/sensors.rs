use std::io::Read;
use std::os::unix::process::CommandExt;
use std::process::Child;
use std::{
    process::{self, Stdio},
    sync::{Arc, Mutex, mpsc},
};

pub(crate) struct SensorConfig {
    pub name: String,
    pub delay_ms: u64,
}


impl SensorConfig {
    pub fn spawn_process(&self) -> Child {
        process::Command::new("termux-sensor")
            .args(["-s", &self.name])
            .args(["-d", &self.delay_ms.to_string()])
            //.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .process_group(0)
            .spawn()
            .expect("Impossibile avviare termux-sensor")
    }

    /// Will exit when the receiver `sensor_rx` is dropped
    pub fn start_reading(sensor_tx: mpsc::Sender<Vec<u8>>, child: Arc<Mutex<Child>>) {
        // Estrai stdout e rilascia immediatamente il lock
        let stdout_stream = {
            let mut guard = child.lock().expect("Mutex poisoned");
            guard.stdout.take()
        }; // guard droppato qui, lock rilasciato

        let mut stdout_stream = stdout_stream.unwrap();
        let mut buffer = [0u8; 4096];
        loop {
            match stdout_stream.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if sensor_tx.send(buffer[..n].to_vec()).is_err() {
                        eprintln!("Receiver disconnesso");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Errore lettura: {e}");
                    break;
                }
            }
        }
    }
}
