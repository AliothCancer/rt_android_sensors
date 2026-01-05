use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;


use serde::Deserialize;

// 1. Definiamo la struct interna che contiene la lista di valori
#[derive(Debug, Deserialize)]
struct AccelerationValues {
    values: Vec<f64>,
}

// 2. Definiamo la struct esterna (root)
#[derive(Debug, Deserialize)]
struct SensorData {
    // Usiamo l'attributo rename per gestire lo spazio nella chiave JSON
    #[serde(rename = "Linear Acceleration")]
    linear_acceleration: AccelerationValues,
}


use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

const DATA_DELAY: u64 = 1;
fn main() -> eframe::Result<()> {
    let (tx, rx) = mpsc::channel::<LinAcc>();

    let sender_handle = thread::spawn(move || send_data(tx));

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Real-time Plot Example",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new(rx)))),
    )?;

    sender_handle.join().unwrap();

    Ok(())
}

struct MyApp {
    rx: Receiver<LinAcc>,
    time: f64,
    lines: Vec<[f64;2]>
}

impl MyApp {
    fn new(rx: Receiver<LinAcc>) -> Self {
        Self { time: 0.0 , rx, lines: vec![]}
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Aggiorna il tempo basandosi sul tempo delta del frame precedente
        self.time += ctx.input(|i| i.stable_dt) as f64;
        dbg!(&self.time);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Grafico Real-time in Rust");

            for linear_acc in self.rx.try_iter(){
                self.lines.push([self.time, linear_acc.y]);
            }
            
            let accel: PlotPoints = self.lines.clone().into_iter().collect();
            let line = Line::new( "lineA", accel);

            // 3. Disegna il grafico
            Plot::new("my_plot")
                .view_aspect(2.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
                });
        });

        // 4. Forza il ridisegno continuo per l'animazione
        ctx.request_repaint();
    }
}
fn send_data(tx: Sender<LinAcc>) {
    let addr = "172.17.62.41:8080";
    let listener = TcpListener::bind(addr).unwrap();
    println!("SERVER: In ascolto su {addr}...");

    let tcp_stream = listener.incoming().next().unwrap().unwrap();

    //let buf_reader = BufReader::new(tcp_stream);
    loop {
        let k = serde_json::Deserializer::from_reader(tcp_stream.try_clone().unwrap())
            .into_iter::<SensorData>();
        for data in k.flatten() {
            let d = LinAcc::from(data);
            
            tx.send(d).unwrap();
        }
        
        thread::sleep(Duration::from_millis(DATA_DELAY));
    }
}
#[derive(Debug, Deserialize)]
struct LinAcc {
    x: f64,
    y: f64,
    z: f64,
}

impl From<SensorData> for LinAcc {
    fn from(value: SensorData) -> Self {
        let values = value.linear_acceleration.values;
        LinAcc {
            x: values[0],
            y: values[1],
            z: values[2],
        }
    }
}
