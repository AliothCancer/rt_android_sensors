use super::Worker;
use std::{
    io::Write,              // Necessario per scrivere nello stream
    net::TcpStream,         // Per la connessione TCP
    sync::mpsc::Receiver,
};

// --- INIT STATE ---
// Oltre al canale, passiamo l'indirizzo a cui connetterci
pub struct TcpSenderInitState {
    pub input_rx: Receiver<Vec<u8>>,
    pub target_addr: String, // es: "172.17.62.41:8080"
}

// --- CONTROLLER ---
pub struct TcpSenderController;

// --- WORKER ---
pub struct TcpSenderWorker {
    rx: Receiver<Vec<u8>>,
    stream: TcpStream,       // Manteniamo la connessione aperta
}

impl Worker for TcpSenderWorker {
    type InitState = TcpSenderInitState;
    type Controller = TcpSenderController;

    fn build(state: TcpSenderInitState) -> (Self, Self::Controller) {
        println!("TcpSender: Tentativo di connessione a {}...", state.target_addr);
        
        // Tentiamo la connessione TCP durante la costruzione.
        // Se fallisce, il programma andrà in panico (come da tuo stile expect/unwrap).
        // In produzione potresti voler gestire l'errore o riprovare nel run().
        let stream = TcpStream::connect(&state.target_addr)
            .expect("Impossibile connettersi al server TCP remoto");

        println!("TcpSender: Connesso con successo!");

        (
            TcpSenderWorker {
                rx: state.input_rx,
                stream,
            },
            TcpSenderController,
        )
    }

    fn run(mut self) {
        // Loop di lettura e invio
        // Usiamo 'recv()' che è bloccante: il thread si sospende finché non c'è un messaggio.
        // È molto più efficiente del try_recv() con sleep.
        while let Ok(data) = self.rx.recv() {
            
            // Tentiamo di scrivere i dati nello stream TCP
            if let Err(e) = self.stream.write_all(&data) {
                eprintln!("TcpSender Error: Connessione interrotta durante l'invio: {}", e);
                break; // Usciamo dal loop se la connessione cade
            }
            
            // Opzionale: stampa di debug locale
            // println!("Inviati {} bytes", data.len());
        }

        println!("TcpSender: Terminated (Channel closed or Network error).");
    }
}
// only for debug
fn vec_to_string(v: Vec<u8>) -> String {
    v.into_iter().map(|x| x as char).collect()
}
