mod sensors;

use std::process::Child;
use std::sync::MutexGuard;
use std::thread::JoinHandle;
use std::{
    process::{self},
    sync::{Arc, Mutex, mpsc},
    thread::{self},
    time::Duration,
};

use crate::sensors::TermuxSensor;

fn main() {
    let accelerometer = TermuxSensor {
        name: "Linear Acceleration".to_string(),
        delay_ms: 0,
    };

    let (sensor_tx, sensor_rx) = mpsc::channel();

    println!(
        "MAIN: Avvio lettura\n delay: {} ms || sensore: {}",
        accelerometer.delay_ms, accelerometer.name
    );

    let child = Arc::new(Mutex::new(accelerometer.spawn_process()));
    let reader_child = Arc::clone(&child);
    let termination_child = Arc::clone(&child);

    ctrlc::set_handler(move || {
        println!("main thread: SIGINT Received!");
        println!("Terminating termux-sensor...");
        send_sig_int_term_kill_wait(termination_child.lock().unwrap());
        std::process::exit(0);
    })
    .expect("Errore nell'impostare il gestore SIGINT");

    let reader_thread = ThreadWrapper {
        name: "Reader Thread".to_string(),
        handle: thread::spawn(|| TermuxSensor::start_reading(sensor_tx, reader_child)),
    };
    println!("MAIN: In attesa dei dati...");

    let writer_thread = ThreadWrapper {
        name: String::from("Writer Thread"),
        handle: thread::spawn(|| display_sensor_data(sensor_rx)),
    };

    for x in [writer_thread, reader_thread] {
        println!("{} is finished: {}", x.name, x.handle.is_finished());
        match x.handle.join() {
            Ok(_) => println!("{} terminated correctly", x.name),
            Err(e) => println!("{} terminated with error: {e:?}", x.name),
        };
    }
    send_sig_int_term_kill_wait(child.lock().unwrap());
}

struct ThreadWrapper {
    name: String,
    handle: JoinHandle<()>,
}

/// Gestisce la visualizzazione dei dati del sensore e la terminazione.
fn display_sensor_data(sensor_rx: mpsc::Receiver<Vec<u8>>) {
    let mut counter = 0;

    loop {
        match sensor_rx.try_recv() {
            Ok(dati) => {
                counter += 1;

                if counter > 10_000 {
                    break;
                }
                let dati = vec_to_string(dati);
                println!("{counter}.{dati}");
            }
            Err(mpsc::TryRecvError::Empty) => {
                // channel is empty wait it to be populated
                thread::sleep(Duration::from_millis(300));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // receiver is dropped
                break;
            }
        }
    }
}

fn vec_to_string(v: Vec<u8>) -> String {
    v.into_iter().map(|x| x as char).collect()
}

/// Send SIGINT to the process group ID to make it propagate to all childs
/// and release the sensor resource. Then send SIGTERM and finally SIGKILL if needed.
fn send_sig_int_term_kill_wait(mut child: MutexGuard<'_, Child>) {
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
