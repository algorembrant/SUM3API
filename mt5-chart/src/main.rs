use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use zeromq::{Socket, SocketRecv, SocketSend};
use std::fs::OpenOptions;
use std::io::Write;

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
struct PositionData {
    ticket: u64,
    #[serde(rename = "type")]
    pos_type: String, // "BUY" or "SELL"
    volume: f64,
    price: f64,
    profit: f64,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
struct PendingOrderData {
    ticket: u64,
    #[serde(rename = "type")]
    order_type: String, // "BUY LIMIT", "SELL STOP", etc.
    volume: f64,
    price: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct TickData {
    symbol: String,
    bid: f64,
    ask: f64,
    time: i64,
    #[serde(default)]
    volume: u64,
    // Account info
    #[serde(default)]
    balance: f64,
    #[serde(default)]
    equity: f64,
    #[serde(default)]
    margin: f64,
    #[serde(default)]
    free_margin: f64,
    // Trading constraints
    #[serde(default)]
    min_lot: f64,
    #[serde(default)]
    max_lot: f64,
    #[serde(default)]
    lot_step: f64,
    
    // Active trades
    #[serde(default)]
    positions: Vec<PositionData>,
    #[serde(default)]
    orders: Vec<PendingOrderData>,
}

#[derive(Clone, Debug, Serialize)]
struct OrderRequest {
    #[serde(rename = "type")]
    order_type: String,
    symbol: String,
    volume: f64,
    price: f64,
    #[serde(default)]
    ticket: u64, // For close/cancel
    // History params
    #[serde(skip_serializing_if = "Option::is_none")]
    timeframe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct OrderResponse {
    success: bool,
    ticket: Option<i64>,
    error: Option<String>,
    message: Option<String>,
}

// ============================================================================
// Application State
// ============================================================================

struct Mt5ChartApp {
    // Tick data
    tick_receiver: mpsc::Receiver<TickData>,
    data: Vec<TickData>,
    symbol: String,
    
    // Latest account info
    balance: f64,
    equity: f64,
    margin: f64,
    free_margin: f64,
    min_lot: f64,
    max_lot: f64,
    lot_step: f64,
    
    // Order handling
    order_sender: mpsc::Sender<OrderRequest>,
    response_receiver: mpsc::Receiver<OrderResponse>,
    
    // UI state for order panel
    lot_size: f64,
    lot_size_str: String,
    limit_price: String,
    #[allow(dead_code)]
    stop_price: String,
    last_order_result: Option<String>,
    
    // History Download UI
    history_start_date: String,
    history_end_date: String,
    history_tf: String,
    history_mode: String,
    
    // Live Recording
    is_recording: bool,
    live_record_file: Option<std::fs::File>,
    
    // Live Trade Data
    positions: Vec<PositionData>,
    pending_orders: Vec<PendingOrderData>,
}

impl Mt5ChartApp {
    fn new(
        tick_receiver: mpsc::Receiver<TickData>,
        order_sender: mpsc::Sender<OrderRequest>,
        response_receiver: mpsc::Receiver<OrderResponse>,
    ) -> Self {
        // Defaults dates to "yyyy.mm.dd"
        let now = chrono::Local::now();
        let today_str = now.format("%Y.%m.%d").to_string();
        
        Self {
            tick_receiver,
            data: Vec::new(),
            symbol: "Waiting for data...".to_string(),
            balance: 0.0,
            equity: 0.0,
            margin: 0.0,
            free_margin: 0.0,
            min_lot: 0.01,
            max_lot: 100.0,
            lot_step: 0.01,
            order_sender,
            response_receiver,
            lot_size: 0.01,
            lot_size_str: "0.01".to_string(),
            limit_price: "0.0".to_string(),
            stop_price: "0.0".to_string(),
            last_order_result: None,
            
            history_start_date: today_str.clone(),
            history_end_date: today_str,
            history_tf: "M1".to_string(),
            history_mode: "OHLC".to_string(),
            
            is_recording: false,
            live_record_file: None,
            
            positions: Vec::new(),
            pending_orders: Vec::new(),
        }
    }
    
    fn send_order(&mut self, order_type: &str, price: Option<f64>, ticket: Option<u64>) {
        let price_val = price.unwrap_or(0.0);
        let ticket_val = ticket.unwrap_or(0);
        
        let request = OrderRequest {
            order_type: order_type.to_string(),
            symbol: self.symbol.clone(),
            volume: self.lot_size,
            price: price_val,
            ticket: ticket_val,
            timeframe: None,
            start: None,
            end: None,
            mode: None,
        };
        
        self.send_request_impl(request);
    }
    
    fn send_download_request(&mut self) {
        let request = OrderRequest {
            order_type: "download_history".to_string(),
            symbol: self.symbol.clone(),
            volume: 0.0,
            price: 0.0,
            ticket: 0,
            timeframe: Some(self.history_tf.clone()),
            start: Some(self.history_start_date.clone()),
            end: Some(self.history_end_date.clone()),
            mode: Some(self.history_mode.clone()),
        };
        
        self.send_request_impl(request);
    }
    
    fn send_request_impl(&mut self, request: OrderRequest) {
        if let Err(e) = self.order_sender.try_send(request) {
            self.last_order_result = Some(format!("Failed to send: {}", e));
        } else {
            self.last_order_result = Some("Request sent...".to_string());
        }
    }
    
    fn adjust_lot_size(&mut self, delta: f64) {
        let new_lot = self.lot_size + delta;
        // Round to lot_step
        let steps = (new_lot / self.lot_step).round();
        self.lot_size = (steps * self.lot_step).max(self.min_lot).min(self.max_lot);
        self.lot_size_str = format!("{:.2}", self.lot_size);
    }
    
    fn toggle_recording(&mut self) {
        self.is_recording = !self.is_recording;
        if self.is_recording {
            let filename = format!("Live_{}_{}.csv", self.symbol, chrono::Local::now().format("%Y%m%d_%H%M%S"));
            match OpenOptions::new().create(true).append(true).open(&filename) {
                Ok(mut file) => {
                    let _ = writeln!(file, "Time,Bid,Ask,Volume");
                    self.live_record_file = Some(file);
                    self.last_order_result = Some(format!("Recording to {}", filename));
                }
                Err(e) => {
                    self.is_recording = false;
                    self.last_order_result = Some(format!("Rec Error: {}", e));
                }
            }
        } else {
            self.live_record_file = None;
            self.last_order_result = Some("Recording Stopped".to_string());
        }
    }
}

impl eframe::App for Mt5ChartApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Receive all available tick data from the channel without blocking
        while let Ok(tick) = self.tick_receiver.try_recv() {
            self.symbol = tick.symbol.clone();
            
            // Record if active
            if self.is_recording {
                if let Some(mut file) = self.live_record_file.as_ref() {
                     let _ = writeln!(file, "{},{},{},{}", tick.time, tick.bid, tick.ask, tick.volume);
                }
            }
            
            // Update account info from latest tick
            if tick.balance > 0.0 {
                self.balance = tick.balance;
                self.equity = tick.equity;
                self.margin = tick.margin;
                self.free_margin = tick.free_margin;
                self.min_lot = tick.min_lot;
                self.max_lot = tick.max_lot;
                if tick.lot_step > 0.0 {
                    self.lot_step = tick.lot_step;
                }
            }
            
            // Update active trades
            self.positions = tick.positions.clone();
            self.pending_orders = tick.orders.clone();
            
            self.data.push(tick);
            // Keep only last 2000 points
            if self.data.len() > 2000 {
                self.data.remove(0);
            }
        }
        
        // Check for order responses
        while let Ok(response) = self.response_receiver.try_recv() {
            if response.success {
                if let Some(msg) = response.message {
                    self.last_order_result = Some(format!("✓ {}", msg));
                } else {
                    self.last_order_result = Some(format!(
                        "✓ Order executed! Ticket: {}",
                        response.ticket.unwrap_or(0)
                    ));
                }
            } else {
                self.last_order_result = Some(format!(
                    "✗ Failed: {}",
                    response.error.unwrap_or_else(|| "Unknown error".to_string())
                ));
            }
        }

        // ====================================================================
        // Side Panel - Trading Controls
        // ====================================================================
        egui::SidePanel::left("trading_panel")
            .min_width(280.0) // Widen slightly
            .show(ctx, |ui| {
                ui.heading("📊 Trading Panel");
                ui.separator();
                
                // Account Info
                ui.collapsing("💰 Account Info", |ui| {
                    egui::Grid::new("account_grid")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Balance:");
                            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), format!("${:.2}", self.balance));
                            ui.end_row();
                            ui.label("Equity:");
                            ui.colored_label(egui::Color32::from_rgb(100, 180, 255), format!("${:.2}", self.equity));
                            ui.end_row();
                            ui.label("Margin Used:");
                            ui.colored_label(egui::Color32::from_rgb(255, 200, 100), format!("${:.2}", self.margin));
                            ui.end_row();
                            ui.label("Free Margin:");
                            ui.colored_label(egui::Color32::from_rgb(100, 255, 200), format!("${:.2}", self.free_margin));
                            ui.end_row();
                        });
                });
                
                ui.separator();
                
                // Historical Data Section
                ui.heading("📂 Historical Data");
                ui.add_space(5.0);
                
                egui::Grid::new("history_grid").num_columns(2).spacing([10.0, 5.0]).show(ui, |ui| {
                    ui.label("Start (yyyy.mm.dd):");
                    ui.add(egui::TextEdit::singleline(&mut self.history_start_date).desired_width(100.0));
                    ui.end_row();
                    
                    ui.label("End (yyyy.mm.dd):");
                    ui.add(egui::TextEdit::singleline(&mut self.history_end_date).desired_width(100.0));
                    ui.end_row();
                    
                    ui.label("Timeframe:");
                    egui::ComboBox::from_id_source("tf_combo")
                        .selected_text(&self.history_tf)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.history_tf, "M1".to_string(), "M1");
                            ui.selectable_value(&mut self.history_tf, "M5".to_string(), "M5");
                            ui.selectable_value(&mut self.history_tf, "M15".to_string(), "M15");
                            ui.selectable_value(&mut self.history_tf, "H1".to_string(), "H1");
                            ui.selectable_value(&mut self.history_tf, "D1".to_string(), "D1");
                        });
                    ui.end_row();
                    
                    ui.label("Mode:");
                    egui::ComboBox::from_id_source("mode_combo")
                        .selected_text(&self.history_mode)
                        .show_ui(ui, |ui| {
                             ui.selectable_value(&mut self.history_mode, "OHLC".to_string(), "OHLC");
                             ui.selectable_value(&mut self.history_mode, "TICKS".to_string(), "TICKS");
                        });
                    ui.end_row();
                });
                
                ui.add_space(5.0);
                if ui.button("⬇ Download History (CSV)").clicked() {
                    self.send_download_request();
                }
                
                ui.separator();
                
                // Live Recording
                ui.heading("🔴 Live Recording");
                ui.horizontal(|ui| {
                   ui.label(if self.is_recording { "Recording..." } else { "Idle" });
                   if ui.button(if self.is_recording { "Stop" } else { "Start Recording" }).clicked() {
                       self.toggle_recording();
                   }
                });
                
                ui.separator();

                // Order Controls
                ui.heading("📦 Trade Controls");
                
                // Lot Size
                ui.horizontal(|ui| {
                    if ui.button("−").clicked() { self.adjust_lot_size(-self.lot_step); }
                    let response = ui.add(egui::TextEdit::singleline(&mut self.lot_size_str).desired_width(60.0));
                    if response.lost_focus() {
                        if let Ok(parsed) = self.lot_size_str.parse::<f64>() {
                            self.lot_size = parsed.max(self.min_lot).min(self.max_lot);
                            self.lot_size_str = format!("{:.2}", self.lot_size);
                        }
                    }
                    if ui.button("+").clicked() { self.adjust_lot_size(self.lot_step); }
                    
                    ui.label(format!("Lots (Max: {:.1})", self.max_lot));
                });
                
                ui.add_space(5.0);
                ui.label("Market Orders:");
                ui.horizontal(|ui| {
                    if ui.button("BUY").clicked() { self.send_order("market_buy", None, None); }
                    if ui.button("SELL").clicked() { self.send_order("market_sell", None, None); }
                });
                
                ui.add_space(5.0);
                ui.label("Pending Orders:");
                 ui.horizontal(|ui| {
                    ui.label("@ Price:");
                    ui.add(egui::TextEdit::singleline(&mut self.limit_price).desired_width(70.0));
                });
                ui.horizontal(|ui| {
                    let p = self.limit_price.parse().unwrap_or(0.0);
                    if ui.small_button("Buy Limit").clicked() { self.send_order("limit_buy", Some(p), None); }
                    if ui.small_button("Sell Limit").clicked() { self.send_order("limit_sell", Some(p), None); }
                    if ui.small_button("Buy Stop").clicked() { self.send_order("stop_buy", Some(p), None); }
                    if ui.small_button("Sell Stop").clicked() { self.send_order("stop_sell", Some(p), None); }
                });

                ui.separator();

                // Order result feedback
                if let Some(ref result) = self.last_order_result {
                    ui.heading("📨 Last Message");
                    ui.label(result); // Allow wrapping
                }
            });

        // ====================================================================
        // Central Panel - Chart
        // ====================================================================
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("📈 {}", self.symbol));
            
            // Header Info
            if let Some(last_tick) = self.data.last() {
                ui.horizontal(|ui| {
                    ui.label(format!("{:.5} / {:.5}", last_tick.bid, last_tick.ask));
                });
            }
            
            ui.separator();

            // Price chart - Index-based X Axis
            let time_map: Vec<i64> = self.data.iter().map(|t| t.time).collect();
            
            let plot = Plot::new("mt5_price_plot")
                .legend(egui_plot::Legend::default())
                .allow_boxed_zoom(true)
                .allow_drag(true)
                .allow_scroll(true)
                .allow_zoom(true)
                .x_axis_formatter(move |x, _range, _width| {
                    let idx = x.value.round() as isize;
                    if idx >= 0 && (idx as usize) < time_map.len() {
                        let timestamp = time_map[idx as usize];
                        let seconds = timestamp % 60;
                        let minutes = (timestamp / 60) % 60;
                        let hours = (timestamp / 3600) % 24;
                        return format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
                    }
                    "".to_string()
                });

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
                
                // Draw Active Positions
                for pos in &self.positions {
                    let color = if pos.pos_type == "BUY" {
                        egui::Color32::from_rgb(50, 100, 255) 
                    } else {
                        egui::Color32::from_rgb(255, 50, 50) 
                    };
                    
                    plot_ui.hline(
                        egui_plot::HLine::new(pos.price)
                            .color(color)
                            .name(format!("{} #{}", pos.pos_type, pos.ticket))
                            .style(egui_plot::LineStyle::Dashed { length: 10.0 })
                    );
                }
            });
        });

        // Request a repaint to update the chart continuously
        ctx.request_repaint();
    }
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Channels for tick data
    let (tick_tx, tick_rx) = mpsc::channel(100);
    
    // Channels for order requests and responses
    let (order_tx, mut order_rx) = mpsc::channel::<OrderRequest>(10);
    let (response_tx, response_rx) = mpsc::channel::<OrderResponse>(10);

    // ========================================================================
    // Spawn ZMQ Tick Subscriber task
    // ========================================================================
    tokio::spawn(async move {
        let mut socket = zeromq::SubSocket::new();
        match socket.connect("tcp://127.0.0.1:5555").await {
            Ok(_) => println!("Connected to ZMQ Tick Publisher on port 5555"),
            Err(e) => eprintln!("Failed to connect to ZMQ tick publisher: {}", e),
        }
        
        let _ = socket.subscribe("").await;

        loop {
            match socket.recv().await {
                Ok(msg) => {
                    if let Some(payload_bytes) = msg.get(0) {
                        if let Ok(json_str) = std::str::from_utf8(payload_bytes) {
                            match serde_json::from_str::<TickData>(json_str) {
                                Ok(tick) => {
                                    if let Err(e) = tick_tx.send(tick).await {
                                        eprintln!("Tick channel error: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => eprintln!("JSON Parse Error: {}. Msg: {}", e, json_str),
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("ZMQ Tick Recv Error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                }
            }
        }
    });

    // ========================================================================
    // Spawn ZMQ Order Request/Response task
    // ========================================================================
    tokio::spawn(async move {
        let mut socket = zeromq::ReqSocket::new();
        match socket.connect("tcp://127.0.0.1:5556").await {
            Ok(_) => println!("Connected to ZMQ Order Handler on port 5556"),
            Err(e) => {
                eprintln!("Failed to connect to ZMQ order handler: {}", e);
                return;
            }
        }

        while let Some(order_request) = order_rx.recv().await {
            // Serialize order request to JSON
            let json_request = match serde_json::to_string(&order_request) {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Failed to serialize order request: {}", e);
                    continue;
                }
            };
            
            println!("Sending request: {}", json_request);
            
            // Send request
            if let Err(e) = socket.send(json_request.into()).await {
                eprintln!("Failed to send: {}", e);
                let _ = response_tx.send(OrderResponse {
                    success: false,
                    ticket: None,
                    error: Some(format!("Send failed: {}", e)),
                    message: None,
                }).await;
                continue;
            }
            
            // Wait for response
            match socket.recv().await {
                Ok(msg) => {
                    if let Some(payload_bytes) = msg.get(0) {
                        if let Ok(json_str) = std::str::from_utf8(payload_bytes) {
                            println!("Received response: {}", json_str);
                            match serde_json::from_str::<OrderResponse>(json_str) {
                                Ok(response) => {
                                    let _ = response_tx.send(response).await;
                                }
                                Err(e) => {
                                    let _ = response_tx.send(OrderResponse {
                                        success: false,
                                        ticket: None,
                                        error: Some(format!("Parse error: {}", e)),
                                        message: None,
                                    }).await;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Response recv error: {}", e);
                    let _ = response_tx.send(OrderResponse {
                        success: false,
                        ticket: None,
                        error: Some(format!("Recv failed: {}", e)),
                        message: None,
                    }).await;
                }
            }
        }
    });

    // ========================================================================
    // Run the egui application
    // ========================================================================
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("MT5 Trading Chart"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Rust + ZMQ + MT5 Trading Chart",
        options,
        Box::new(|_cc| Box::new(Mt5ChartApp::new(tick_rx, order_tx, response_rx))),
    ).map_err(|e| e.into())
}
