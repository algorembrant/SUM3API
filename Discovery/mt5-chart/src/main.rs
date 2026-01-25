use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::mpsc;
use zeromq::{Socket, SocketRecv};

#[derive(Clone, Debug, Deserialize)]
struct TickData {
    symbol: String,
    bid: f64,
    ask: f64,
    time: i64,
}

struct Mt5ChartApp {
    receiver: mpsc::Receiver<TickData>,
    data: Vec<TickData>,
    symbol: String,
}

impl Mt5ChartApp {
    fn new(receiver: mpsc::Receiver<TickData>) -> Self {
        Self {
            receiver,
            data: Vec::new(),
            symbol: "Waiting for data...".to_string(),
        }
    }
}

impl eframe::App for Mt5ChartApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Receive all available data from the channel without blocking
        while let Ok(tick) = self.receiver.try_recv() {
            self.symbol = tick.symbol.clone();
            self.data.push(tick);
            // Keep only last 1000 points to avoid memory issues for this demo
            if self.data.len() > 1000 {
                self.data.remove(0);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("MT5 Live Chart: {}", self.symbol));
            if let Some(last_tick) = self.data.last() {
                ui.label(format!("Bid: {:.5} | Ask: {:.5}", last_tick.bid, last_tick.ask));
            }

            let plot = Plot::new("mt5_plot")
                .view_aspect(2.0)
                .legend(egui_plot::Legend::default());

            plot.show(ui, |plot_ui| {
                let bid_points: PlotPoints = self.data
                    .iter()
                    .enumerate()
                    .map(|(i, t)| [i as f64, t.bid])
                    .collect();
                
                let ask_points: PlotPoints = self.data
                    .iter()
                    .enumerate()
                    .map(|(i, t)| [i as f64, t.ask])
                    .collect();

                plot_ui.line(Line::new(bid_points).name("Bid").color(egui::Color32::from_rgb(100, 200, 100)));
                plot_ui.line(Line::new(ask_points).name("Ask").color(egui::Color32::from_rgb(200, 100, 100)));
            });
        });

        // Request a repaint to update the chart continuously
        ctx.request_repaint();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel(100);

    // Spawn ZMQ Subscriber task
    tokio::spawn(async move {
        // ZMQ Context and Socket
        let mut socket = zeromq::SubSocket::new();
        match socket.connect("tcp://127.0.0.1:5555").await {
            Ok(_) => println!("Connected to ZMQ Publisher"),
            Err(e) => eprintln!("Failed to connect to ZMQ: {}", e),
        }
        
        let _ = socket.subscribe("").await;

        loop {
             match socket.recv().await {
                Ok(msg) => {
                    // msg is a MultiPart message, usually the first part depends on subscription
                    // In our simple case, payload is likely in the first frame or the whole string
                    if let Some(payload_bytes) = msg.get(0) {
                         if let Ok(json_str) = std::str::from_utf8(payload_bytes) {
                             // Attempt to parse JSON
                             // The MT5 EA sends: {"symbol":..., "bid":..., ...}
                             match serde_json::from_str::<TickData>(json_str) {
                                 Ok(tick) => {
                                     if let Err(e) = tx.send(tick).await {
                                         eprintln!("Channel error: {}", e);
                                         break;
                                     }
                                 }
                                 Err(e) => eprintln!("JSON Parse Error: {}. Msg: {}", e, json_str),
                             }
                         }
                    }
                }
                Err(e) => {
                    eprintln!("ZMQ Recv Error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                }
             }
        }
    });

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust + ZMQ + MT5's MQL5 Chart",
        options,
        Box::new(|_cc| Box::new(Mt5ChartApp::new(rx))),
    ).map_err(|e| e.into())
}
