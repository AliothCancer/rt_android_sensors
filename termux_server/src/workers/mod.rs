pub mod sensor;
pub mod tcp_sender;


use std::thread::{self, JoinHandle};

pub trait Worker: Sized + Send + 'static {
    // InitState: I dati necessari "a freddo" per avviare l'attore.
    // Sostituisce la vecchia "Config".
    type InitState;

    // Controller: L'interfaccia per comunicare con l'attore mentre gira.
    type Controller: Send + 'static;

    fn name() -> String;

    // Costruisce l'attore e il suo controller
    fn build(state: Self::InitState) -> (Self, Self::Controller);

    // La logica di business (il loop)
    fn run(self);

    // Helper per lanciare il thread
    fn spawn(state: Self::InitState) -> (JoinHandle<()>, Self::Controller) {
        println!("Thread: {} has been spawned!", Self::name());
        let (worker, controller) = Self::build(state);
        let handle = thread::spawn(move || worker.run());
        (handle, controller)
    }
}
