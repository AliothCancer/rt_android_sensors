use super::Worker;
use std::{
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

// --- INIT STATE ---
pub struct DisplayInitState {
    pub input_rx: Receiver<Vec<u8>>,
    pub max_items: usize,
}

// --- CONTROLLER ---
// In questo caso non abbiamo comandi da inviare al display,
// ma il controller serve comunque come handle (se lo droppi, potremmo gestire cleanup)
pub struct DisplayController;

// --- WORKER ---
pub struct DisplayWorker {
    rx: Receiver<Vec<u8>>,
    limit: usize,
}

impl Worker for DisplayWorker {
    type InitState = DisplayInitState;
    type Controller = DisplayController;

    fn build(state: DisplayInitState) -> (Self, Self::Controller) {
        (
            DisplayWorker {
                rx: state.input_rx,
                limit: state.max_items,
            },
            DisplayController,
        )
    }

    fn run(self) {
        let mut counter = 0;

        loop {
            match self.rx.try_recv() {
                Ok(dati) => {
                    counter += 1;

                    if counter > self.limit {
                        break;
                    }
                    let dati = vec_to_string(dati);
                    println!("{counter}.{dati}");
                }
                Err(mpsc::TryRecvError::Empty) => {
                    println!("DisplayWorker: Waiting for data...");
                    // channel is empty wait it to be populated
                    thread::sleep(Duration::from_millis(3000));
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // receiver is dropped
                    break;
                }
            }
        }
        println!("DisplayWorker: Terminated.");
    }
}

// only for debug
fn vec_to_string(v: Vec<u8>) -> String {
    v.into_iter().map(|x| x as char).collect()
}
