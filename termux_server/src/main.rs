mod workers;
//mod channels;
mod sensors;

use std::process::Child;
use std::sync::MutexGuard;
use std::{
    process::{self},
    sync::mpsc,
    thread::{self},
    time::Duration,
};

use crate::workers::Worker;
use crate::workers::display::{DisplayInitState, DisplayWorker};
use crate::workers::sensor::{SensorCommand, SensorInitState, SensorWorker};

fn main() {
    // Channel to read data from SensorWorker to DisplayWorker or other consumer
    let (data_tx, data_rx) = mpsc::channel();

    // DisplayWorker initial_state/config
    let display_state = DisplayInitState {
        input_rx: data_rx,
        max_items: 10_000, // Si ferma dopo 200 letture
    };
    let (display_handle, _display_ctrl) = DisplayWorker::spawn(display_state);

    // 3. Configurazione e Avvio SENSOR
    let sensor_state = SensorInitState {
        name: "Linear Acceleration".to_string(),
        delay_ms: 500,       // Più veloce per testare
        output_tx: data_tx, // Passiamo il trasmettitore al sensore
    };
    let (sensor_handle, sensor_ctrl) = SensorWorker::spawn(sensor_state);

    // 4. Gestione Segnali (CTRL+C)
    // Non dobbiamo più gestire kill complicati qui. Basta dire al controller di fermarsi.
    // Clono il controller (o meglio, il sender interno se volessi clonarlo)
    // Ma qui usiamo un trucco: passiamo il sender a una variabile statica o usiamo un channel
    // Per semplicità nel main, simuliamo solo un timer o un wait.

    // Setup handler CTRL-C semplice
    let (sig_tx, sig_rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        sig_tx.send(()).unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    println!("MAIN: Sistema avviato. Premi Ctrl+C per uscire.");


    // For debugging im gonna emulate the laptop server Start signal
    sensor_ctrl.cmd_tx.send(SensorCommand::Start).expect("Should start the sensor");

    
    loop {
        // Controlla se il display ha finito
        if display_handle.is_finished() {
            println!("MAIN: Display finished work.");
            break;
        }

        // Controlla Ctrl+C
        if sig_rx.try_recv().is_ok() {
            println!("MAIN: Ctrl+C detected. Shutting down...");
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    // 5. Graceful Shutdown
    // Inviamo il comando di exit al sensore
    let _ = sensor_ctrl.cmd_tx.send(SensorCommand::Exit);

    // Attendiamo la chiusura dei thread
    // Nota: Il Display si chiuderà quando il Sensor smette di mandare dati (data_tx viene droppato)
    sensor_handle.join().unwrap();
    display_handle.join().unwrap();

    println!("MAIN: Tutto chiuso pulito.");
}

