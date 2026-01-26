# MT5 ZeroMQ Trading System - Architecture Diagram

This diagram illustrates the complete data flow and component integration of the MT5-Chart trading system with **bidirectional ZeroMQ communication** for real-time tick data, order execution, data recording, and historical data retrieval.

## System Overview

```mermaid
flowchart TB
    subgraph MT5["MetaTrader 5 Platform"]
        direction TB
        subgraph MQL5Layer["MQL5 Layer"]
            EA["ZmqPublisher.mq5<br/>(Expert Advisor v2.0)"]
            Header["Zmq.mqh<br/>(ZMQ Wrapper Class)"]
            Trade["Trade.mqh<br/>(Trading Functions)"]
            FileMgr["File Operations<br/>(Data Export)"]
        end
        
        subgraph DLLLayer["Native DLL Layer"]
            LibZmq["libzmq.dll<br/>(ZeroMQ Native Library)"]
        end
        
        EA -->|"#include"| Header
        EA -->|"#include"| Trade
        EA -->|"FileOpen"| FileMgr
        Header -->|"#import"| LibZmq
    end

    subgraph RustApp["mt5-chart (Rust Application)"]
        direction TB
        Main["main.rs"]
        
        subgraph UIComponents["UI Components"]
            TradingPanel["Trading Panel<br/>• Account Info<br/>• Trade Management<br/>• History Download & Recording"]
            PriceChart["Price Chart<br/>• Index-based X-Axis (Flexible)"]
        end
        
        subgraph Crates["Rust Dependencies"]
            ZmqRust["zeromq crate<br/>(SUB + REQ Sockets)"]
            Eframe["eframe + egui<br/>(GUI Framework)"]
            EguiPlot["egui_plot<br/>(Chart Rendering)"]
            Tokio["tokio<br/>(Async Runtime)"]
            Serde["serde + serde_json<br/>(JSON Parsing)"]
            Chrono["chrono<br/>(Time Formatting)"]
        end
        
        Main --> UIComponents
        Main --> ZmqRust
        Main --> Eframe
        Main --> EguiPlot
        Main --> Tokio
        Main --> Serde
        Main --> Chrono
    end

    subgraph Network["ZeroMQ Transport"]
        PUBSocket["PUB Socket<br/>tcp://127.0.0.1:5555<br/>(Tick Data + Account + Positions)"]
        REPSocket["REP Socket<br/>tcp://127.0.0.1:5556<br/>(Order Execution, History Requests)"]
    end

    LibZmq -->|"PUB Bind"| PUBSocket
    LibZmq -->|"REP Bind"| REPSocket
    PUBSocket -->|"SUB Connect"| ZmqRust
    ZmqRust -->|"REQ Connect"| REPSocket
```

## Bidirectional Communication Flow

```mermaid
sequenceDiagram
    participant MT5 as MetaTrader 5
    participant EA as ZmqPublisher.mq5
    participant PUB as PUB Socket :5555
    participant REP as REP Socket :5556
    participant Rust as mt5-chart (Rust)
    participant GUI as egui Trading Panel

    Note over MT5,GUI: Initialization Phase
    EA->>PUB: Bind PUB Socket
    EA->>REP: Bind REP Socket
    Rust->>PUB: Connect SUB Socket
    Rust->>REP: Connect REQ Socket

    Note over MT5,GUI: Runtime - Tick Data Flow (Continuous)
    loop Every Market Tick
        MT5->>EA: OnTick()
        EA->>EA: Get tick + positions + orders
        EA->>PUB: Publish JSON
        PUB->>Rust: Receive tick data
        Rust->>Rust: Append to Data Vector
        Rust->>Rust: Record to CSV (if enabled)
        Rust->>GUI: Update Index-based Chart
    end

    Note over MT5,GUI: History Download (New)
    GUI->>Rust: User requests History (Dates, TF)
    Rust->>REP: Send {"type":"download_history", "start":"...", ...}
    REP->>EA: Receive request
    EA->>MT5: CopyRates/CopyTicks
    EA->>MT5: FileWrite (Save CSV to MQL5/Files)
    MT5-->>EA: File Path
    EA->>REP: Response {"success":true, "message":"Saved to..."}
    REP->>Rust: Receive response
    Rust->>GUI: Show success message

    Note over MT5,GUI: Order Execution (User Action)
    GUI->>Rust: User clicks BUY/SELL
    Rust->>REP: Send request {"type":"market_buy", ...}
    REP->>EA: Receive order request
    EA->>MT5: Execute trade (CTrade)
    MT5-->>EA: Trade result (ticket)
    EA->>REP: Send response {"success":true, "ticket":123...}
    REP->>Rust: Receive response
    Rust->>GUI: Display outcome
```

## Data Structures

### Tick Data JSON (PUB → SUB)
Updated to include active positions and pending orders for chart visualization.
```json
{
    "symbol": "XAUUSD",
    "bid": 2000.50,
    "ask": 2000.80,
    "time": 1706188800,
    "volume": 150,
    "balance": 10000.00,
    "equity": 10050.00,
    "margin": 200.00,
    "free_margin": 9850.00,
    "min_lot": 0.01,
    "max_lot": 100.00,
    "lot_step": 0.01,
    "positions": [
        {"ticket": 12345, "type": "BUY", "volume": 1.0, "price": 2000.50, "profit": 50.0}
    ],
    "orders": [
        {"ticket": 12346, "type": "BUY LIMIT", "volume": 0.1, "price": 1990.00}
    ]
}
```

### Order Request JSON (REQ → REP)
Supported request `type`s now include history download actions.
```json
{
    "type": "download_history",
    "symbol": "XAUUSD",
    "timeframe": "M1",
    "start": "2024.01.01",
    "end": "2024.01.02",
    "mode": "OHLC"
}
```

**Supported Actions:**
| Action Type | Description | Required Fields |
|-------------|-------------|-----------------|
| `market_buy/sell` | Instant execution | `symbol`, `volume` |
| `limit_buy/sell` | Pending limit order | `symbol`, `volume`, `price` |
| `stop_buy/sell` | Pending stop order | `symbol`, `volume`, `price` |
| `close_position` | Close active position | `ticket` |
| `cancel_order` | Delete pending order | `ticket` |
| `download_history` | Download OHLC/Tick CSV | `start`, `end`, `timeframe`, `mode` |

## Visualization Logic

### 1. Index-Based Charting (Flexible Axis)
-   **X-Axis**: Represents **Index** (0, 1, 2, ... N).
    -   This allows for uniform spacing regardless of time gaps (e.g., weekends).
    -   Enables flexible zooming and stretching (unconstrained by time steps).
-   **Formatter**: Maps Index `i` back to `Time` from the data vector for display.
-   **Volume**: Removed entirely to maximize price visibility.

### 2. Live Data Recording
-   **Mechanism**: The Rust client appends every received tick to a local CSV file.
-   **Format**: `Time, Bid, Ask, Volume`.
-   **Control**: Toggle button in the UI.

### 3. UI Component Layout
```mermaid
flowchart LR
    subgraph Window["MT5 Trading Chart Window"]
        subgraph SidePanel["Left Side Panel (250px)"]
            direction TB
            AccountInfo["💰 Account Info"]
            History["📂 Historical Data<br/>(Download Control)"]
            Recording["🔴 Live Recording"]
            TradeControls["⚡ Order Controls<br/>(Market/Limit/Stop)"]
            ActivePos["💼 Active Positions"]
            PendingOrd["⏳ Pending Orders"]
            
            AccountInfo --> History
            History --> Recording
            Recording --> TradeControls
            TradeControls --> ActivePos
            ActivePos --> PendingOrd
        end
        
        subgraph MainPanel["Central Panel"]
            direction TB
            Header["Symbol + Prices"]
            PriceChart2["Price Chart (Index-Axis)<br/>• Bid/Ask Lines<br/>• Order Lines (Blue/Red)<br/>• Flexible Zoom/Stretch"]
            
            Header --> PriceChart2
        end
    end
```
