use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Bar, BarChart};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use zeromq::{Socket, SocketRecv, SocketSend};

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Clone, Debug, Deserialize)]
struct PositionData {
    ticket: u64,
    #[serde(rename = "type")]
    pos_type: String, // "BUY" or "SELL"
    volume: f64,
    price: f64,
    profit: f64,
}

#[derive(Clone, Debug, Deserialize)]
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
}

#[derive(Clone, Debug, Deserialize)]
struct OrderResponse {
    success: bool,
    ticket: Option<i64>,
    error: Option<String>,
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
    stop_price: String,
    last_order_result: Option<String>,
    
    // Volume display
    volume_history: Vec<(i64, u64)>, // (time, volume)
    
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
            volume_history: Vec::new(),
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
        };
        
        // Non-blocking send
        if let Err(e) = self.order_sender.try_send(request) {
            self.last_order_result = Some(format!("Failed to send order: {}", e));
        } else {
            self.last_order_result = Some("Order sent, waiting for response...".to_string());
        }
    }
    
    fn adjust_lot_size(&mut self, delta: f64) {
        let new_lot = self.lot_size + delta;
        // Round to lot_step
        let steps = (new_lot / self.lot_step).round();
        self.lot_size = (steps * self.lot_step).max(self.min_lot).min(self.max_lot);
        self.lot_size_str = format!("{:.2}", self.lot_size);
    }
}

impl eframe::App for Mt5ChartApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Receive all available tick data from the channel without blocking
        while let Ok(tick) = self.tick_receiver.try_recv() {
            self.symbol = tick.symbol.clone();
            self.volume_history.push((tick.time, tick.volume));
            
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
            // Keep only last 1000 points to avoid memory issues
            if self.data.len() > 1000 {
                self.data.remove(0);
            }
            if self.volume_history.len() > 100 {
                self.volume_history.remove(0);
            }
        }
        
        // Check for order responses
        while let Ok(response) = self.response_receiver.try_recv() {
            if response.success {
                self.last_order_result = Some(format!(
                    "✓ Order executed! Ticket: {}",
                    response.ticket.unwrap_or(0)
                ));
            } else {
                self.last_order_result = Some(format!(
                    "✗ Order failed: {}",
                    response.error.unwrap_or_else(|| "Unknown error".to_string())
                ));
            }
        }

        // ====================================================================
        // Side Panel - Trading Controls
        // ====================================================================
        egui::SidePanel::left("trading_panel")
            .min_width(250.0)
            .show(ctx, |ui| {
                ui.heading("📊 Trading Panel");
                ui.separator();
                
                // =============================================================
                // Account Information Section
                // =============================================================
                ui.heading("💰 Account Info");
                ui.add_space(5.0);
                
                egui::Grid::new("account_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Balance:");
                        ui.colored_label(egui::Color32::from_rgb(100, 200, 100), 
                            format!("${:.2}", self.balance));
                        ui.end_row();
                        
                        ui.label("Equity:");
                        ui.colored_label(egui::Color32::from_rgb(100, 180, 255), 
                            format!("${:.2}", self.equity));
                        ui.end_row();
                        
                        ui.label("Margin Used:");
                        ui.colored_label(egui::Color32::from_rgb(255, 200, 100), 
                            format!("${:.2}", self.margin));
                        ui.end_row();
                        
                        ui.label("Free Margin:");
                        ui.colored_label(egui::Color32::from_rgb(100, 255, 200), 
                            format!("${:.2}", self.free_margin));
                        ui.end_row();
                    });
                
                ui.separator();
                
                // Current prices
                if let Some(last_tick) = self.data.last() {
                    ui.heading("📈 Prices");
                    egui::Grid::new("price_grid")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Bid:");
                            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), 
                                format!("{:.5}", last_tick.bid));
                            ui.end_row();
                            
                            ui.label("Ask:");
                            ui.colored_label(egui::Color32::from_rgb(200, 100, 100), 
                                format!("{:.5}", last_tick.ask));
                            ui.end_row();
                            
                            ui.label("Volume:");
                            ui.colored_label(egui::Color32::from_rgb(100, 150, 255), 
                                format!("{}", last_tick.volume));
                            ui.end_row();
                        });
                }
                
                ui.separator();
                
                // =============================================================
                // Lot Size Adjustment Section
                // =============================================================
                ui.heading("📦 Lot Size");
                ui.add_space(5.0);
                
                ui.horizontal(|ui| {
                    ui.label(format!("Min: {:.2} | Max: {:.2} | Step: {:.2}", 
                        self.min_lot, self.max_lot, self.lot_step));
                });
                
                ui.add_space(5.0);
                
                ui.horizontal(|ui| {
                    if ui.button("−").clicked() {
                        self.adjust_lot_size(-self.lot_step);
                    }
                    
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.lot_size_str)
                            .desired_width(80.0)
                            .horizontal_align(egui::Align::Center)
                    );
                    if response.lost_focus() {
                        if let Ok(parsed) = self.lot_size_str.parse::<f64>() {
                            self.lot_size = parsed.max(self.min_lot).min(self.max_lot);
                            self.lot_size_str = format!("{:.2}", self.lot_size);
                        }
                    }
                    
                    if ui.button("+").clicked() {
                        self.adjust_lot_size(self.lot_step);
                    }
                });
                
                // Quick lot size buttons
                ui.horizontal(|ui| {
                    if ui.small_button("0.01").clicked() { self.lot_size = 0.01; self.lot_size_str = "0.01".to_string(); }
                    if ui.small_button("0.1").clicked() { self.lot_size = 0.1; self.lot_size_str = "0.10".to_string(); }
                    if ui.small_button("0.5").clicked() { self.lot_size = 0.5; self.lot_size_str = "0.50".to_string(); }
                    if ui.small_button("1.0").clicked() { self.lot_size = 1.0; self.lot_size_str = "1.00".to_string(); }
                });
                
                ui.add_space(10.0);
                ui.separator();
                
                // ============================================================
                // Market Orders
                // ============================================================
                ui.heading("🔥 Market Orders");
                ui.horizontal(|ui| {
                    let buy_btn = egui::Button::new(
                        egui::RichText::new("BUY").color(egui::Color32::WHITE).strong()
                    ).fill(egui::Color32::from_rgb(50, 150, 50));
                    if ui.add(buy_btn).clicked() {
                        self.send_order("market_buy", None, None);
                    }
                    
                    let sell_btn = egui::Button::new(
                        egui::RichText::new("SELL").color(egui::Color32::WHITE).strong()
                    ).fill(egui::Color32::from_rgb(180, 50, 50));
                    if ui.add(sell_btn).clicked() {
                        self.send_order("market_sell", None, None);
                    }
                });
                
                ui.add_space(10.0);
                
                // ============================================================
                // Limit Orders
                // ============================================================
                ui.heading("📋 Limit Orders");
                ui.horizontal(|ui| {
                    ui.label("Price:");
                    ui.add(egui::TextEdit::singleline(&mut self.limit_price).desired_width(100.0));
                });
                ui.horizontal(|ui| {
                    let price: f64 = self.limit_price.parse().unwrap_or(0.0);
                    
                    let buy_btn = egui::Button::new(
                        egui::RichText::new("BUY LIMIT").color(egui::Color32::WHITE)
                    ).fill(egui::Color32::from_rgb(50, 120, 50));
                    if ui.add(buy_btn).clicked() {
                        self.send_order("limit_buy", Some(price), None);
                    }
                    
                    let sell_btn = egui::Button::new(
                        egui::RichText::new("SELL LIMIT").color(egui::Color32::WHITE)
                    ).fill(egui::Color32::from_rgb(150, 50, 50));
                    if ui.add(sell_btn).clicked() {
                        self.send_order("limit_sell", Some(price), None);
                    }
                });
                
                ui.add_space(10.0);
                
                // ============================================================
                // Stop Orders
                // ============================================================
                ui.heading("🛑 Stop Orders");
                ui.horizontal(|ui| {
                    ui.label("Price:");
                    ui.add(egui::TextEdit::singleline(&mut self.stop_price).desired_width(100.0));
                });
                ui.horizontal(|ui| {
                    let price: f64 = self.stop_price.parse().unwrap_or(0.0);
                    
                    let buy_btn = egui::Button::new(
                        egui::RichText::new("BUY STOP").color(egui::Color32::WHITE)
                    ).fill(egui::Color32::from_rgb(50, 100, 150));
                    if ui.add(buy_btn).clicked() {
                        self.send_order("stop_buy", Some(price), None);
                    }
                    
                    let sell_btn = egui::Button::new(
                        egui::RichText::new("SELL STOP").color(egui::Color32::WHITE)
                    ).fill(egui::Color32::from_rgb(150, 100, 50));
                    if ui.add(sell_btn).clicked() {
                        self.send_order("stop_sell", Some(price), None);
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                
                ui.add_space(10.0);
                ui.separator();
                
                // ============================================================
                // Active Positions Management
                // ============================================================
                if !self.positions.is_empty() {
                    ui.heading("💼 Active Positions");
                    egui::ScrollArea::vertical().id_source("positions_scroll").max_height(150.0).show(ui, |ui| {
                        for pos in &self.positions {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    let color = if pos.pos_type == "BUY" {
                                        egui::Color32::from_rgb(100, 200, 100)
                                    } else {
                                        egui::Color32::from_rgb(200, 100, 100)
                                    };
                                    ui.colored_label(color, &pos.pos_type);
                                    ui.label(format!("{:.2} lots @ {:.5}", pos.volume, pos.price));
                                });
                                ui.horizontal(|ui| {
                                    let profit_color = if pos.profit >= 0.0 {
                                        egui::Color32::from_rgb(100, 200, 100)
                                    } else {
                                        egui::Color32::from_rgb(200, 100, 100)
                                    };
                                    ui.colored_label(profit_color, format!("${:.2}", pos.profit));
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("❌ Close").clicked() {
                                            self.send_order("close_position", None, Some(pos.ticket));
                                        }
                                    });
                                });
                            });
                        }
                    });
                    ui.separator();
                }

                // ============================================================
                // Pending Orders Management
                // ============================================================
                if !self.pending_orders.is_empty() {
                    ui.heading("⏳ Pending Orders");
                    egui::ScrollArea::vertical().id_source("orders_scroll").max_height(100.0).show(ui, |ui| {
                        for order in &self.pending_orders {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&order.order_type);
                                    ui.label(format!("{:.2} lots @ {:.5}", order.volume, order.price));
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("❌ Cancel").clicked() {
                                        self.send_order("cancel_order", None, Some(order.ticket));
                                    }
                                });
                            });
                        }
                    });
                    ui.separator();
                }

                // Order result feedback
                if let Some(ref result) = self.last_order_result {
                    ui.heading("📨 Last Order");
                    if result.starts_with("✓") {
                        ui.colored_label(egui::Color32::from_rgb(100, 200, 100), result);
                    } else if result.starts_with("✗") {
                        ui.colored_label(egui::Color32::from_rgb(200, 100, 100), result);
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(200, 200, 100), result);
                    }
                }
            });

        // ====================================================================
        // Central Panel - Chart
        // ====================================================================
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("📈 MT5 Live Chart: {}", self.symbol));
            
            // Tick volume display in header
            if let Some(last_tick) = self.data.last() {
                ui.horizontal(|ui| {
                    ui.label(format!("Bid: {:.5}", last_tick.bid));
                    ui.label(" | ");
                    ui.label(format!("Ask: {:.5}", last_tick.ask));
                    ui.label(" | ");
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 200, 100),
                        format!("Tick Vol: {}", last_tick.volume)
                    );
                });
            }
            
            ui.separator();

            // Price chart
            let price_plot = Plot::new("mt5_price_plot")
                .height(ui.available_height() * 0.65)
                .legend(egui_plot::Legend::default())
                .x_axis_formatter(|x, _range, _width| {
                    let timestamp = x as i64;
                    // Simple HH:MM:SS formatter
                    let seconds = timestamp % 60;
                    let minutes = (timestamp / 60) % 60;
                    let hours = (timestamp / 3600) % 24;
                    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                });

            price_plot.show(ui, |plot_ui| {
                let bid_points: PlotPoints = self.data
                    .iter()
                    .map(|t| [t.time as f64, t.bid])
                    .collect();
                
                let ask_points: PlotPoints = self.data
                    .iter()
                    .map(|t| [t.time as f64, t.ask])
                    .collect();

                plot_ui.line(Line::new(bid_points).name("Bid").color(egui::Color32::from_rgb(100, 200, 100)));
                plot_ui.line(Line::new(ask_points).name("Ask").color(egui::Color32::from_rgb(200, 100, 100)));
                
                // Draw Active Positions
                for pos in &self.positions {
                    let color = if pos.pos_type == "BUY" {
                        egui::Color32::from_rgb(50, 100, 255) // Blue
                    } else {
                        egui::Color32::from_rgb(255, 50, 50) // Red
                    };
                    
                    plot_ui.hline(
                        egui_plot::HLine::new(pos.price)
                            .color(color)
                            .name(format!("{} #{}", pos.pos_type, pos.ticket))
                            .style(egui_plot::LineStyle::Dashed { length: 10.0 })
                    );
                    
                    // Note: Actual buttons need to be outside the plot or using sophisticated Overlay
                    // For now, listing them in a separate panel or relying on the plot legend/tooltip is easier,
                    // but the user asked for labels ON the chart. egui_plot doesn't easily support interactive buttons inside.
                    // We will implement a list below the chart or overlay text.
                    plot_ui.text(egui_plot::Text::new(
                        egui_plot::PlotPoint::new(
                            self.data.last().map(|t| t.time as f64).unwrap_or(0.0), 
                            pos.price
                        ),
                        format!("{} {:.2}", pos.pos_type, pos.volume)
                    ).color(color));
                }
                
                // Draw Pending Orders
                for order in &self.pending_orders {
                    let color = if order.order_type.contains("BUY") {
                         egui::Color32::from_rgb(50, 100, 255) // Blue
                    } else {
                         egui::Color32::from_rgb(255, 50, 50) // Red
                    };
                    
                    plot_ui.hline(
                        egui_plot::HLine::new(order.price)
                            .color(color)
                            .name(format!("{} #{}", order.order_type, order.ticket))
                            .style(egui_plot::LineStyle::Dotted { spacing: 10.0 })
                    );
                }
            });
            
            ui.add_space(5.0);
            ui.label("Tick Volume");
            
            // Volume chart
            let volume_plot = Plot::new("mt5_volume_plot")
                .height(ui.available_height())
                .legend(egui_plot::Legend::default())
                .show_axes([true, true])
                .x_axis_formatter(|x, _range, _width| {
                    let timestamp = x as i64;
                    let seconds = timestamp % 60;
                    let minutes = (timestamp / 60) % 60;
                    let hours = (timestamp / 3600) % 24;
                    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                });

            volume_plot.show(ui, |plot_ui| {
                let bars: Vec<Bar> = self.volume_history
                    .iter()
                    .map(|(t, v)| Bar::new(*t as f64, *v as f64).width(0.8))  // Note: Width might need adjustment for time scale
                    .collect();
                
                plot_ui.bar_chart(
                    BarChart::new(bars)
                        .name("Volume")
                        .color(egui::Color32::from_rgb(100, 150, 255))
                );
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
            
            println!("Sending order: {}", json_request);
            
            // Send request
            if let Err(e) = socket.send(json_request.into()).await {
                eprintln!("Failed to send order: {}", e);
                let _ = response_tx.send(OrderResponse {
                    success: false,
                    ticket: None,
                    error: Some(format!("Send failed: {}", e)),
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
                                    }).await;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Order response recv error: {}", e);
                    let _ = response_tx.send(OrderResponse {
                        success: false,
                        ticket: None,
                        error: Some(format!("Recv failed: {}", e)),
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
