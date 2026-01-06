mod workers;
//mod channels;

use std::{
    sync::mpsc,
    thread::{self},
    time::Duration,
};

use crate::workers::{
    Worker,
    tcp_sender::{TcpSenderInitState, TcpSenderWorker},
};
use crate::workers::{
    sensor::{SensorCommand, SensorInitState, SensorWorker},
    tcp_sender::TcpMess,
};

fn main() {
    // CTRL-C CAPTURING config
    let (sig_tx, sig_rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        sig_tx.send(()).unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    let (data_tx, data_rx) = mpsc::channel();

    // SENSOR DATA
    let sensor_state = SensorInitState {
        name: "Linear Acceleration".to_string(),
        delay_ms: 1,        // Più veloce per testare
        output_tx: data_tx, // Passiamo il trasmettitore al sensore
    };
    // TCP STREAM
    let tcp_state = TcpSenderInitState {
        input_rx: data_rx,
        target_addr: "172.17.62.41:8080".to_string(),
    };

    let (sensor_handle, sensor_ctrl) = SensorWorker::spawn(sensor_state);
    let (tcp_handle, tcp_ctrl) = TcpSenderWorker::spawn(tcp_state);

    println!("MAIN: Press Ctrl+C to exit.");

    // Controlling the threads
    sensor_ctrl
        .cmd_tx
        .send(SensorCommand::Start)
        .expect("Should start the sensor");
    tcp_ctrl
        .cmd_tx
        .send(TcpMess::TryConnect)
        .expect("Should connect");
    loop {
        if tcp_handle.is_finished() {
            println!("MAIN: TCP streaming working.");
            break;
        }

        if sig_rx.try_recv().is_ok() {
            println!("MAIN: Ctrl+C detected. Shutting down...");
            break;
        }

        thread::sleep(Duration::from_millis(1000));
    }

    // 5. Graceful Shutdown
    // Inviamo il comando di exit al sensore
    let _ = sensor_ctrl.cmd_tx.send(SensorCommand::Exit);
    let _ = tcp_ctrl.cmd_tx.send(TcpMess::Exit);
    // Attendiamo la chiusura dei thread
    // Nota: Il Display si chiuderà quando il Sensor smette di mandare dati (data_tx viene droppato)
    sensor_handle.join().unwrap();
    tcp_handle.join().unwrap();

    println!("MAIN: Tutto chiuso pulito.");
}
