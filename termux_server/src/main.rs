use std::io::stdout;
use std::os::unix::process::CommandExt;
use std::process::Child;
use std::sync::MutexGuard;
// 1. Necessary for process_group()
use std::{
    io::{BufRead, BufReader, Write},
    process::{self, Stdio},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};
struct TermuxSensor {
    name: String,
    delay_ms: u64,
}

//#[allow(clippy::zombie_processes)]
fn main() {
    // TIP: Esegui "termux-sensor -l" nel terminale per copiare il nome esatto
    let accelerometer = TermuxSensor {
        // Se "BMI120..." non va, prova il generico "Accelerometer"
        name: "Linear Acceleration".to_string(),
        delay_ms: 200,
    };

    let (tx, rx) = mpsc::channel();

    println!("MAIN: Avvio lettura sensore: {}", accelerometer.name);

    let child = Arc::new(Mutex::new(
        process::Command::new("termux-sensor")
            .args(["-s", &accelerometer.name])
            .args(["-d", &accelerometer.delay_ms.to_string()])
            .stdout(Stdio::piped())
            //.stdin(Stdio::inherit())
            .process_group(0)
            .spawn()
            .expect("THREAD: Impossibile avviare termux-sensor"),
    ));
    let child_ref = Arc::clone(&child);

    let _sh_handle = thread::spawn(move || {
        // 1. Apri il lucchetto e prendi ciò che serve
        let stdout_stream = {
            let mut guard = child_ref.lock().expect("Mutex poisoned");
            guard.stdout.take() // Prendi ownership e lascia None nella struct
        }; // <--- Qui la variabile 'guard' muore e il lucchetto viene RILASCIATO.

        // 2. Ora processa i dati senza bloccare gli altri thread
        if let Some(stdout) = stdout_stream {
            let reader = BufReader::new(stdout);

            for l in reader.lines().map_while(Result::ok) {
                if l.trim().len() > 1 && tx.send(l).is_err() {
                    break;
                }
            }
        }
    });

    println!("MAIN: In attesa dei dati...");

    let mut counter = 0;
    loop {
        match rx.try_recv() {
            Ok(dati) => {
                counter += 1;
                if counter > 100
                    && let Ok(child) = child.lock()
                {
                    
                    let _ = stdout().flush();
                    println!("\nexiting...");
                    send_sigint_and_then_sigterm(child);
                }

                // Sovrascriviamo la riga corrente per un effetto "dashboard" pulito
                print!("\r\x1b[K"); // \r torna a capo, \x1b[K pulisce la riga

                // Tronchiamo per evitare spam se il JSON è lungo
                let display = if dati.len() > 90 {
                    format!("{}...", &dati[..90])
                } else {
                    dati
                };

                print!("termux-sensor stdout: {}", display);
                std::io::stdout().flush().unwrap();
            }
            Err(mpsc::TryRecvError::Empty) => {
                // Piccola pausa per non consumare 100% CPU
                thread::sleep(Duration::from_millis(50));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                println!("\nMAIN: Il sensore ha smesso di inviare dati.");
                break;
            }
        }
    }
}

/// Send SIGINT to the process group ID to make
/// it propagate to all childs and so release the sensor resource,
/// otherwise the SIGINT won't have effect and sensor resource will
/// not be available anymore causing to not sending data. THEN it is
/// necessary to also send a SIGTERM after some secs to terminate the 
/// program, finally try to kill it after a little sleep from the SIGTERM,
/// just to be sure.
fn send_sigint_and_then_sigterm(mut child: MutexGuard<'_, Child>) {
    let id = child.id();
    let _ = process::Command::new("kill")
        .args(["-2", &format!("-{}", id)]) // Note the "-" before the PID
        .output();
    
    // GIVE TIME TO RELEASE SENSOR
    thread::sleep(Duration::from_secs(2));
    
    let _ = process::Command::new("kill")
        .args([&format!("-{}", id)]) // Note the "-" before the PID
        .output();

    // GIVE TIME TO EXIT
    thread::sleep(Duration::from_secs(2));

    // No pity! Avada Kedabra!
    match child.kill() {
        Ok(_) => (),
        Err(e) => {
            println!("Tried to send SIGTERM but was not enough, so the program SIGKILL-ed it: {e}")
        }
    }

    // CRITICAL: Reap the zombie
    // Without this, the process entry remains in the OS table (defunct)
    let _ = child.wait(); 
    println!("KILLER: Cleanup complete.");
}
