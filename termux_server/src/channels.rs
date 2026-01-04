use std::sync::mpsc::{self, Receiver, Sender};

pub struct SystemProducers {
    pub sensor_tx: Sender<Vec<u8>>,
}

pub struct SystemConsumers {
    pub sensor_rx: Receiver<Vec<u8>>,
}

pub struct SystemChannels;
impl SystemChannels {
    pub fn build() -> (SystemProducers, SystemConsumers) {
        let (sensor_tx, sensor_rx) = mpsc::channel();

        let producers = SystemProducers { sensor_tx};

        let consumers = SystemConsumers { sensor_rx };

        (producers, consumers)
    }
}
