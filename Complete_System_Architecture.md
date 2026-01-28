# Complete End-to-End System Architecture: MQL5 ↔ ZeroMQ ↔ Rust for SUM3API

**Version**: 2.0.0  
**Last Updated**: 2026-01-28  
**Purpose**: Comprehensive technical documentation covering all micro-level implementation details

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Complete Architecture Diagram](#complete-architecture-diagram)
3. [Security Architecture](#security-architecture)
4. [Component Deep Dive](#component-deep-dive)
5. [Data Flow & Communication Patterns](#data-flow--communication-patterns)
6. [Account Information Fetching](#account-information-fetching)
7. [Complete Data Structures](#complete-data-structures)
8. [ZeroMQ Layer Details](#zeromq-layer-details)
9. [Async Task Management](#async-task-management)
10. [File Structure & Dependencies](#file-structure--dependencies)

---

## System Overview

This system implements a **secure, real-time bidirectional trading bridge** between MetaTrader 5 and a Rust-based GUI application using ZeroMQ as the transport layer.

### Core Design Principles

1. **Security First**: No credentials in code or transmitted over network
2. **Real-time Performance**: Tick-level granularity with minimal latency
3. **Separation of Concerns**: Authentication vs. Trading logic
4. **Async Architecture**: Non-blocking I/O for maximum throughput
5. **Type Safety**: Strong typing in both MQL5 and Rust

---

## Complete Architecture Diagram

### High-Level System Architecture

```mermaid
flowchart TB
    subgraph USER_SPACE["User Space"]
        USER[("User")]
    end
    
    subgraph MT5_PLATFORM["MetaTrader 5 Platform (Authenticated Process)"]
        direction TB
        
        subgraph AUTH["Authentication Layer"]
            MT5_GUI[MT5 Terminal GUI]
            SESSION["Authenticated Session<br/>[+] Account ID<br/>[+] Server Connection<br/>[+] Trading Permissions"]
        end
        
        subgraph DATA_SOURCES["MT5 Data Sources"]
            MARKET[("Market Data Feed<br/>Tick Stream")]
            ACCOUNT_DB[("Account Database<br/>ACCOUNT_BALANCE<br/>ACCOUNT_EQUITY<br/>ACCOUNT_MARGIN<br/>ACCOUNT_MARGIN_FREE")]
            POSITIONS_DB[("Positions Database<br/>Active Trades")]
            ORDERS_DB[("Orders Database<br/>Pending Orders")]
            HISTORY_DB[("Historical Database<br/>OHLC & Tick Data")]
        end
        
        subgraph EA_LAYER["Expert Advisor Layer"]
            EA[ZmqPublisher.mq5<br/>Expert Advisor]
            TRADE_ENGINE[CTrade Engine<br/>Order Execution]
        end
        
        USER -->|1. Manual Login<br/>account + password + server| MT5_GUI
        MT5_GUI --> SESSION
        SESSION -.->|Inherits Session| EA
        
        MARKET --> EA
        ACCOUNT_DB --> EA
        POSITIONS_DB --> EA
        ORDERS_DB --> EA
        HISTORY_DB --> EA
        EA --> TRADE_ENGINE
    end
    
    subgraph ZMQ_LAYER["ZeroMQ Transport Layer (localhost)"]
        direction LR
        PUB_SOCKET[["[PUB] PUB Socket<br/>tcp://0.0.0.0:5555<br/>Broadcast Mode"]]
        REP_SOCKET[["[REP] REP Socket<br/>tcp://0.0.0.0:5556<br/>Request-Reply Mode"]]
    end
    
    subgraph RUST_APP["Rust Application (mt5-chart)"]
        direction TB
        
        subgraph ASYNC_RUNTIME["Tokio Async Runtime"]
            TICK_TASK[["[Task] Tick Subscriber Task<br/>SubSocket<br/>Port 5555"]]
            ORDER_TASK[["[Task] Order Handler Task<br/>ReqSocket<br/>Port 5556"]]
        end
        
        subgraph CHANNELS["MPSC Channels"]
            TICK_CHAN[Tick Channel<br/>capacity: 100]
            ORDER_CHAN[Order Request Channel<br/>capacity: 10]
            RESPONSE_CHAN[Order Response Channel<br/>capacity: 10]
        end
        
        subgraph APP_STATE["Application State"]
            STATE[Mt5ChartApp<br/>• data: Vec&lt;TickData&gt;<br/>• balance, equity, margin<br/>• positions, orders<br/>• UI state]
        end
        
        subgraph GUI["egui GUI Components"]
            CHART[["[Chart] Price Chart<br/>Bid/Ask Lines<br/>Position Lines<br/>Order Breaklines"]]
            ACCOUNT_PANEL[["[Account] Account Info Panel<br/>Balance, Equity<br/>Margin, Free Margin"]]
            TRADE_PANEL[["[Trade] Trade Controls<br/>Market Orders<br/>Pending Orders"]]
            HISTORY_PANEL[["[History] History Download<br/>OHLC/Tick CSV Export"]]
            RECORD_PANEL[["[REC] Live Recording<br/>Real-time CSV Capture"]]
            POSITIONS_PANEL[["[Pos] Active Positions<br/>Close Management"]]
            ORDERS_PANEL[["[Orders] Pending Orders<br/>Cancel Management"]]
        end
        
        TICK_TASK --> TICK_CHAN
        ORDER_TASK <--> ORDER_CHAN
        ORDER_TASK <--> RESPONSE_CHAN
        
        TICK_CHAN --> STATE
        STATE <--> ORDER_CHAN
        RESPONSE_CHAN --> STATE
        
        STATE --> CHART
        STATE --> ACCOUNT_PANEL
        STATE --> TRADE_PANEL
        STATE --> HISTORY_PANEL
        STATE --> RECORD_PANEL
        STATE --> POSITIONS_PANEL
        STATE --> ORDERS_PANEL
    end
    
    EA --> PUB_SOCKET
    EA <--> REP_SOCKET
    
    PUB_SOCKET -.->|JSON Tick Stream<br/>Non-blocking| TICK_TASK
    ORDER_TASK -.->|JSON Request<br/>Blocking| REP_SOCKET
    REP_SOCKET -.->|JSON Response<br/>Blocking| ORDER_TASK
    
    style USER_SPACE fill:#f0f0f0,stroke:#666,stroke-width:2px
    style MT5_PLATFORM fill:#e6f3ff,stroke:#0066cc,stroke-width:3px
    style AUTH fill:#fff9e6,stroke:#ffcc00,stroke-width:2px
    style ZMQ_LAYER fill:#f0fff0,stroke:#00cc00,stroke-width:3px
    style RUST_APP fill:#ffe6f0,stroke:#cc0066,stroke-width:3px
    style SESSION fill:#ccffcc,stroke:#00cc00,stroke-width:2px
```

---

## Security Architecture

### Authentication Flow & Credential Isolation

```mermaid
sequenceDiagram
    participant User
    participant MT5_GUI as MT5 Terminal GUI
    participant Broker as Broker Server
    participant Session as Authenticated Session
    participant EA as MQL5 Expert Advisor
    participant ZMQ as ZeroMQ Sockets
    participant Rust as Rust Application
    
    rect rgb(255, 240, 200)
        Note over User,Session: Phase 1: One-Time Authentication (Manual)
        User->>MT5_GUI: Enter credentials<br/>• Account ID: 12345678<br/>• Password: ********<br/>• Server: MetaQuotes-Demo
        MT5_GUI->>Broker: Authenticate
        Broker-->>MT5_GUI: [+] Authentication Success
        MT5_GUI->>Session: Create Authenticated Session
        Note over Session: Session stores:<br/>[+] Account credentials<br/>[+] Server connection<br/>[+] Trading permissions<br/>[+] Account state
    end
    
    rect rgb(230, 255, 230)
        Note over Session,EA: Phase 2: EA Initialization (Session Inheritance)
        User->>MT5_GUI: Attach EA to chart
        MT5_GUI->>EA: OnInit()
        EA->>Session: Request session access
        Session-->>EA: [+] Grant access (no credentials needed)
        Note over EA: EA now has:<br/>[+] Authenticated session<br/>[+] Account info access<br/>[+] Trading permissions<br/>[-] NO credentials stored
    end
    
    rect rgb(230, 240, 255)
        Note over EA,Rust: Phase 3: External Communication (Credential-Free)
        EA->>ZMQ: Bind PUB socket (port 5555)
        EA->>ZMQ: Bind REP socket (port 5556)
        Rust->>ZMQ: Connect SUB socket (127.0.0.1:5555)
        Rust->>ZMQ: Connect REQ socket (127.0.0.1:5556)
        Note over ZMQ,Rust: [+] Only localhost TCP addresses<br/>[-] NO credentials transmitted<br/>[-] NO authentication required
    end
    
    rect rgb(255, 230, 230)
        Note over EA,Rust: Phase 4: Runtime Operations (Secure)
        loop Every Tick
            EA->>Session: AccountInfoDouble(ACCOUNT_BALANCE)
            Session-->>EA: balance value
            EA->>Session: AccountInfoDouble(ACCOUNT_EQUITY)
            Session-->>EA: equity value
            EA->>ZMQ: Publish JSON {balance, equity, ...}
            ZMQ-->>Rust: Receive data (no auth needed)
        end
        
        Rust->>ZMQ: Send order request {type: "market_buy", ...}
        ZMQ-->>EA: Receive request
        EA->>Session: Execute trade via CTrade
        Session-->>EA: Trade result
        EA->>ZMQ: Send response {success: true, ticket: ...}
        ZMQ-->>Rust: Receive response
    end
```

### Security Comparison: MT5 Python API vs. MQL5+ZMQ+Rust

| Security Aspect | MT5 Python API | MQL5 + ZeroMQ + Rust |
|----------------|----------------|----------------------|
| **Credentials in Code** | Required (`account`, `password`, `server`) | Not Required |
| **Credential Storage** | Must store in config/env vars | No storage needed |
| **Credential Transmission** | Transmitted via Python API | Never transmitted |
| **Authentication Method** | Programmatic (code-based) | Manual (GUI-based) |
| **Session Model** | Python creates new session | EA inherits existing session |
| **Attack Surface** | High (credentials exposed) | Low (no credentials) |
| **Version Control Risk** | High (accidental commits) | None |
| **Network Exposure** | Depends on configuration | Localhost only (default) |
| **Credential Interception** | Possible during transmission | Not applicable |
| **Separation of Concerns** | Mixed (auth + trading) | Clear (auth separate) |

### Account Information Access Pattern

```mermaid
flowchart LR
    subgraph MT5["MT5 Authenticated Session"]
        ACC_API["Account Info API<br/>AccountInfoDouble()"]
        ACC_DATA[(Account Data<br/>ACCOUNT_BALANCE<br/>ACCOUNT_EQUITY<br/>ACCOUNT_MARGIN<br/>ACCOUNT_MARGIN_FREE)]
    end
    
    subgraph EA["Expert Advisor"]
        FETCH[Fetch Account Info<br/>Lines 366-369]
        JSON_BUILD[Build JSON Payload<br/>Lines 428-443]
    end
    
    subgraph ZMQ["ZeroMQ"]
        PUB[PUB Socket<br/>Port 5555]
    end
    
    subgraph RUST["Rust App"]
        PARSE[Parse JSON<br/>Lines 745-753]
        UPDATE[Update State<br/>Lines 338-348]
        DISPLAY[Display in GUI<br/>Lines 449-466]
    end
    
    ACC_API --> ACC_DATA
    ACC_DATA -->|No credentials needed| FETCH
    FETCH --> JSON_BUILD
    JSON_BUILD --> PUB
    PUB -.->|JSON over TCP| PARSE
    PARSE --> UPDATE
    UPDATE --> DISPLAY
    
    style ACC_DATA fill:#ccffcc,stroke:#00cc00,stroke-width:2px
    style FETCH fill:#e6f3ff,stroke:#0066cc,stroke-width:2px
    style PUB fill:#fff9e6,stroke:#ffcc00,stroke-width:2px
    style DISPLAY fill:#ffe6f0,stroke:#cc0066,stroke-width:2px
```

---

## Component Deep Dive

### 1. MQL5 Expert Advisor: ZmqPublisher.mq5

#### File Structure
- **Location**: `MQL5/Experts/ZmqPublisher.mq5`
- **Lines**: 451
- **Size**: 19,014 bytes
- **Dependencies**: `Zmq.mqh`, `Trade.mqh`

#### Input Parameters

```mql5
input string InpPubAddress = "tcp://0.0.0.0:5555"; // Tick Publisher Address
input string InpRepAddress = "tcp://0.0.0.0:5556"; // Order Handler Address
input double InpDefaultSlippage = 10;              // Default Slippage (points)
```

#### Global Variables

```mql5
CZmq *g_publisher;  // PUB socket for tick data broadcasting
CZmq *g_responder;  // REP socket for order request handling
CTrade g_trade;     // MT5 trading helper class
```

#### Initialization Sequence (OnInit)

```mermaid
flowchart TD
    START([OnInit Called]) --> INIT_PUB[Create CZmq Publisher]
    INIT_PUB --> PUB_INIT{Init ZMQ_PUB?}
    PUB_INIT -->|Failed| FAIL1[Return INIT_FAILED]
    PUB_INIT -->|Success| PUB_BIND{Bind to Port 5555?}
    PUB_BIND -->|Failed| FAIL2[Return INIT_FAILED]
    PUB_BIND -->|Success| INIT_REP[Create CZmq Responder]
    
    INIT_REP --> REP_INIT{Init ZMQ_REP?}
    REP_INIT -->|Failed| FAIL3[Return INIT_FAILED]
    REP_INIT -->|Success| REP_BIND{Bind to Port 5556?}
    REP_BIND -->|Failed| FAIL4[Return INIT_FAILED]
    REP_BIND -->|Success| CONFIG_TRADE[Configure CTrade]
    
    CONFIG_TRADE --> SET_SLIP[SetDeviationInPoints]
    SET_SLIP --> SET_FILL[SetTypeFilling IOC]
    SET_FILL --> SUCCESS[Return INIT_SUCCEEDED]
    
    style START fill:#e6f3ff,stroke:#0066cc,stroke-width:2px
    style SUCCESS fill:#ccffcc,stroke:#00cc00,stroke-width:2px
    style FAIL1 fill:#ffcccc,stroke:#cc0000,stroke-width:2px
    style FAIL2 fill:#ffcccc,stroke:#cc0000,stroke-width:2px
    style FAIL3 fill:#ffcccc,stroke:#cc0000,stroke-width:2px
    style FAIL4 fill:#ffcccc,stroke:#cc0000,stroke-width:2px
```

#### OnTick() Processing Flow

```mermaid
flowchart TB
    TICK([OnTick Event]) --> CHECK_REQ{Check REP Socket<br/>Non-blocking}
    
    CHECK_REQ -->|Request Available| RECV_REQ[Receive Request JSON]
    RECV_REQ --> PROCESS[ProcessOrderRequest]
    PROCESS --> SEND_RESP[Send Response JSON<br/>Blocking]
    SEND_RESP --> CHECK_PUB
    
    CHECK_REQ -->|No Request| CHECK_PUB{Check Publisher}
    
    CHECK_PUB -->|NULL| END([Return])
    CHECK_PUB -->|Valid| GET_TICK[SymbolInfoTick]
    
    GET_TICK --> GET_ACCOUNT[Get Account Info<br/>Lines 366-369]
    GET_ACCOUNT --> GET_CONSTRAINTS[Get Symbol Constraints<br/>Lines 372-374]
    GET_CONSTRAINTS --> GET_POSITIONS[Get Active Positions<br/>Lines 377-397]
    GET_POSITIONS --> GET_ORDERS[Get Pending Orders<br/>Lines 400-425]
    GET_ORDERS --> BUILD_JSON[Build Complete JSON<br/>Lines 428-443]
    BUILD_JSON --> PUBLISH[Publish to PUB Socket<br/>Line 445]
    PUBLISH --> END
    
    style TICK fill:#e6f3ff,stroke:#0066cc,stroke-width:2px
    style GET_ACCOUNT fill:#fff9e6,stroke:#ffcc00,stroke-width:2px
    style PUBLISH fill:#ccffcc,stroke:#00cc00,stroke-width:2px
```

#### Account Information Fetching (Detailed)

**Lines 366-369: Account Info Retrieval**

```mql5
// Get account info
double balance = AccountInfoDouble(ACCOUNT_BALANCE);
double equity = AccountInfoDouble(ACCOUNT_EQUITY);
double margin = AccountInfoDouble(ACCOUNT_MARGIN);
double freeMargin = AccountInfoDouble(ACCOUNT_MARGIN_FREE);
```

**MQL5 Account Info Functions**:
- `AccountInfoDouble(ACCOUNT_BALANCE)` - Current account balance
- `AccountInfoDouble(ACCOUNT_EQUITY)` - Current equity (balance + floating P/L)
- `AccountInfoDouble(ACCOUNT_MARGIN)` - Margin currently used
- `AccountInfoDouble(ACCOUNT_MARGIN_FREE)` - Free margin available

**Security Note**: These functions access the authenticated session's account data **without requiring credentials**. The EA inherits the session from the MT5 terminal.

#### Symbol Trading Constraints (Lines 372-374)

```mql5
// Get symbol trading constraints
double minLot = SymbolInfoDouble(_Symbol, SYMBOL_VOLUME_MIN);
double maxLot = SymbolInfoDouble(_Symbol, SYMBOL_VOLUME_MAX);
double lotStep = SymbolInfoDouble(_Symbol, SYMBOL_VOLUME_STEP);
```

#### Position Fetching Loop (Lines 377-397)

```mql5
// Get Active Positions (Only for current symbol to simplify)
string positionsJson = "[";
int posCount = PositionsTotal();
bool firstPos = true;
for(int i = 0; i < posCount; i++) {
   ulong ticket = PositionGetTicket(i);
   if(PositionSelectByTicket(ticket)) {
      if(PositionGetString(POSITION_SYMBOL) == _Symbol) {
         if(!firstPos) StringAdd(positionsJson, ",");
         
         string posType = (PositionGetInteger(POSITION_TYPE) == POSITION_TYPE_BUY) ? "BUY" : "SELL";
         StringAdd(positionsJson, "{\"ticket\":" + IntegerToString(ticket) + 
                   ",\"type\":\"" + posType + "\"" +
                   ",\"volume\":" + DoubleToString(PositionGetDouble(POSITION_VOLUME), 2) +
                   ",\"price\":" + DoubleToString(PositionGetDouble(POSITION_PRICE_OPEN), _Digits) +
                   ",\"profit\":" + DoubleToString(PositionGetDouble(POSITION_PROFIT), 2) + 
                   "}");
         firstPos = false;
      }
   }
}
StringAdd(positionsJson, "]");
```

#### Order Request Processing (Lines 87-188)

```mermaid
flowchart TD
    START([ProcessOrderRequest]) --> PARSE[Parse JSON Request<br/>Extract: type, symbol, volume, price, ticket]
    
    PARSE --> ROUTE{Route by Type}
    
    ROUTE -->|market_buy| MB[Get ASK price<br/>g_trade.Buy]
    ROUTE -->|market_sell| MS[Get BID price<br/>g_trade.Sell]
    ROUTE -->|limit_buy| LB[g_trade.BuyLimit]
    ROUTE -->|limit_sell| LS[g_trade.SellLimit]
    ROUTE -->|stop_buy| SB[g_trade.BuyStop]
    ROUTE -->|stop_sell| SS[g_trade.SellStop]
    ROUTE -->|close_position| CP[g_trade.PositionClose]
    ROUTE -->|cancel_order| CO[g_trade.OrderDelete]
    ROUTE -->|download_history| DH[DownloadHistory]
    ROUTE -->|unknown| ERR[Unknown order type]
    
    MB --> CHECK{Success?}
    MS --> CHECK
    LB --> CHECK
    LS --> CHECK
    SB --> CHECK
    SS --> CHECK
    CP --> CHECK
    CO --> CHECK
    DH --> CHECK
    ERR --> BUILD_FAIL
    
    CHECK -->|Yes| BUILD_SUCCESS["Build Success JSON<br/>{success: true, ticket: ...}"]
    CHECK -->|No| BUILD_FAIL["Build Failure JSON<br/>{success: false, error: ...}"]
    
    BUILD_SUCCESS --> RETURN[Return JSON Response]
    BUILD_FAIL --> RETURN
    
    style START fill:#e6f3ff,stroke:#0066cc,stroke-width:2px
    style BUILD_SUCCESS fill:#ccffcc,stroke:#00cc00,stroke-width:2px
    style BUILD_FAIL fill:#ffcccc,stroke:#cc0000,stroke-width:2px
```

### 2. ZMQ Wrapper: Zmq.mqh

#### File Structure
- **Location**: `MQL5/Include/Zmq/Zmq.mqh`
- **Lines**: 145
- **Size**: 4,100 bytes
- **Purpose**: MQL5 wrapper around libzmq.dll

#### Class Structure

```mermaid
classDiagram
    class CZmq {
        -long m_context
        -long m_socket
        -bool m_initialized
        +CZmq()
        +~CZmq()
        +bool Init(int type)
        +bool Bind(string endpoint)
        +bool Connect(string endpoint)
        +int Send(string message, bool nonBlocking)
        +string Receive(bool nonBlocking)
        +void Shutdown()
    }
    
    class libzmq_dll {
        <<external>>
        +long zmq_ctx_new()
        +int zmq_ctx_term(long context)
        +long zmq_socket(long context, int type)
        +int zmq_close(long socket)
        +int zmq_bind(long socket, uchar endpoint[])
        +int zmq_connect(long socket, uchar endpoint[])
        +int zmq_send(long socket, uchar buf[], int len, int flags)
        +int zmq_recv(long socket, uchar buf[], int len, int flags)
        +int zmq_errno()
    }
    
    CZmq --> libzmq_dll : imports
```

#### Socket Type Constants

```mql5
#define ZMQ_PUB 1    // Publisher socket (one-to-many)
#define ZMQ_SUB 2    // Subscriber socket (many-to-one)
#define ZMQ_REQ 3    // Request socket (synchronous client)
#define ZMQ_REP 4    // Reply socket (synchronous server)
#define ZMQ_NOBLOCK 1 // Non-blocking flag
```

#### Method Details

**Init(int type)** - Lines 51-68
```mql5
bool Init(int type) {
   if(m_initialized) return true;
   
   m_context = zmq_ctx_new();  // Create ZMQ context
   if(m_context == 0) {
      Print("ZMQ Init failed: Context creation error");
      return false;
   }
   
   m_socket = zmq_socket(m_context, type);  // Create socket of specified type
   if(m_socket == 0) {
      Print("ZMQ Init failed: Socket creation error");
      return false;
   }
   
   m_initialized = true;
   return true;
}
```

**Send(string message, bool nonBlocking)** - Lines 98-114
```mql5
int Send(string message, bool nonBlocking = true) {
   if(!m_initialized) return -1;
   
   uchar data[];
   StringToCharArray(message, data, 0, WHOLE_ARRAY, CP_UTF8);
   int len = ArraySize(data) - 1; // Exclude null terminator
   if (len < 0) len = 0;

   int flags = 0;
   if(nonBlocking) flags = ZMQ_NOBLOCK;
   
   int bytesSent = zmq_send(m_socket, data, len, flags);
   return bytesSent;
}
```

**Receive(bool nonBlocking)** - Lines 117-131
```mql5
string Receive(bool nonBlocking = true) {
   if(!m_initialized) return "";
   
   uchar buffer[4096];
   ArrayInitialize(buffer, 0);
   
   int flags = 0;
   if(nonBlocking) flags = ZMQ_NOBLOCK;
   
   int bytesReceived = zmq_recv(m_socket, buffer, ArraySize(buffer) - 1, flags);
   
   if(bytesReceived <= 0) return "";
   
   return CharArrayToString(buffer, 0, bytesReceived, CP_UTF8);
}
```

### 3. Rust Application: main.rs

#### File Structure
- **Location**: `Rustmt5-chart/src/main.rs`
- **Lines**: 853
- **Size**: 35,504 bytes
- **Language**: Rust 2021 Edition

#### Dependencies (Cargo.toml)

```toml
[dependencies]
eframe = "0.27"           # egui framework
egui = "0.27"             # Immediate mode GUI
egui_plot = "0.27"        # Plotting library
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"        # JSON serialization
tokio = { version = "1", features = ["full"] }
zeromq = "0.3"            # ZeroMQ bindings
chrono = "0.4"            # Date/time handling
```

#### Data Structure Hierarchy

```mermaid
classDiagram
    class TickData {
        +String symbol
        +f64 bid
        +f64 ask
        +i64 time
        +u64 volume
        +f64 balance
        +f64 equity
        +f64 margin
        +f64 free_margin
        +f64 min_lot
        +f64 max_lot
        +f64 lot_step
        +Vec~PositionData~ positions
        +Vec~PendingOrderData~ orders
    }
    
    class PositionData {
        +u64 ticket
        +String pos_type
        +f64 volume
        +f64 price
        +f64 profit
    }
    
    class PendingOrderData {
        +u64 ticket
        +String order_type
        +f64 volume
        +f64 price
    }
    
    class OrderRequest {
        +String order_type
        +String symbol
        +f64 volume
        +f64 price
        +u64 ticket
        +Option~String~ timeframe
        +Option~String~ start
        +Option~String~ end
        +Option~String~ mode
        +Option~u64~ request_id
    }
    
    class OrderResponse {
        +bool success
        +Option~i64~ ticket
        +Option~String~ error
        +Option~String~ message
    }
    
    class OrderBreakline {
        +usize index
        +String order_type
        +i64 ticket
    }
    
    class Mt5ChartApp {
        +Receiver~TickData~ tick_receiver
        +Vec~TickData~ data
        +String symbol
        +f64 balance
        +f64 equity
        +f64 margin
        +f64 free_margin
        +Sender~OrderRequest~ order_sender
        +Receiver~OrderResponse~ response_receiver
        +Vec~PositionData~ positions
        +Vec~PendingOrderData~ pending_orders
        +Vec~OrderBreakline~ order_breaklines
        +update()
        +send_order()
        +send_download_request()
    }
    
    TickData "1" *-- "*" PositionData
    TickData "1" *-- "*" PendingOrderData
    Mt5ChartApp "1" *-- "*" TickData
    Mt5ChartApp "1" *-- "*" OrderBreakline
```

---

## Data Flow & Communication Patterns

### Complete Tick Data Flow

```mermaid
sequenceDiagram
    participant MT5 as MT5 Market
    participant EA as ZmqPublisher.mq5
    participant PUB as PUB Socket :5555
    participant SUB as SUB Socket (Rust)
    participant CHAN as Tick Channel
    participant APP as Mt5ChartApp
    participant GUI as egui GUI
    
    rect rgb(230, 255, 230)
        Note over MT5,EA: Every Tick Event
        MT5->>EA: OnTick()
        
        EA->>EA: SymbolInfoTick(_Symbol, tick)
        EA->>EA: AccountInfoDouble(ACCOUNT_BALANCE)
        EA->>EA: AccountInfoDouble(ACCOUNT_EQUITY)
        EA->>EA: AccountInfoDouble(ACCOUNT_MARGIN)
        EA->>EA: AccountInfoDouble(ACCOUNT_MARGIN_FREE)
        EA->>EA: SymbolInfoDouble(SYMBOL_VOLUME_MIN/MAX/STEP)
        
        loop For each position
            EA->>EA: PositionGetTicket(i)
            EA->>EA: Build position JSON
        end
        
        loop For each order
            EA->>EA: OrderGetTicket(i)
            EA->>EA: Build order JSON
        end
        
        EA->>EA: StringConcatenate(json, ...)
        EA->>PUB: Send(json, non-blocking)
    end
    
    rect rgb(230, 240, 255)
        Note over PUB,APP: Async Rust Processing
        PUB-->>SUB: TCP transmission
        SUB->>SUB: recv().await
        SUB->>SUB: serde_json::from_str::<TickData>()
        SUB->>CHAN: tick_tx.send(tick).await
        
        CHAN-->>APP: tick_receiver.try_recv()
        APP->>APP: Update balance, equity, margin
        APP->>APP: Update positions, orders
        APP->>APP: data.push(tick)
        APP->>APP: Record to CSV if recording
    end
    
    rect rgb(255, 240, 230)
        Note over APP,GUI: GUI Update (60 FPS)
        APP->>GUI: update(&mut self, ctx, frame)
        GUI->>GUI: Draw price chart
        GUI->>GUI: Draw account panel
        GUI->>GUI: Draw positions/orders
        GUI->>GUI: ctx.request_repaint()
    end
```

### Complete Order Execution Flow

```mermaid
sequenceDiagram
    participant GUI as egui GUI
    participant APP as Mt5ChartApp
    participant CHAN as Order Channel
    participant REQ as REQ Socket (Rust)
    participant REP as REP Socket :5556
    participant EA as ZmqPublisher.mq5
    participant TRADE as CTrade Engine
    participant MT5 as MT5 Terminal
    
    rect rgb(255, 240, 230)
        Note over GUI,APP: User Interaction
        GUI->>APP: Button clicked: "BUY"
        APP->>APP: send_order("market_buy", None, None)
        APP->>APP: Build OrderRequest struct
        APP->>APP: serde_json::to_string(&request)
        APP->>CHAN: order_sender.try_send(request)
    end
    
    rect rgb(230, 240, 255)
        Note over CHAN,EA: Async Order Task
        CHAN-->>REQ: order_rx.recv().await
        REQ->>REQ: Serialize to JSON
        REQ->>REP: socket.send(json).await (blocking)
        
        REP-->>EA: Receive(non-blocking) in OnTick
        EA->>EA: ProcessOrderRequest(request)
        EA->>EA: ExtractJsonString(request, "type")
        EA->>EA: ExtractJsonDouble(request, "volume")
    end
    
    rect rgb(230, 255, 230)
        Note over EA,MT5: Trade Execution
        EA->>EA: if(orderType == "market_buy")
        EA->>EA: askPrice = SymbolInfoDouble(SYMBOL_ASK)
        EA->>TRADE: g_trade.Buy(volume, symbol, askPrice, 0, 0, "Rust GUI Order")
        TRADE->>MT5: Execute market order
        MT5-->>TRADE: Trade result
        TRADE-->>EA: success = true, resultTicket = 12345678
        
        EA->>EA: Build response JSON
        EA->>EA: {"success":true,"ticket":12345678}
        EA->>REP: Send(response, blocking)
    end
    
    rect rgb(240, 230, 255)
        Note over REP,APP: Response Processing
        REP-->>REQ: socket.recv().await (blocking)
        REQ->>REQ: serde_json::from_str::<OrderResponse>()
        REQ->>CHAN: response_tx.send(response).await
        
        CHAN-->>APP: response_receiver.try_recv()
        APP->>APP: if response.success
        APP->>APP: Create OrderBreakline
        APP->>APP: order_breaklines.push(breakline)
        APP->>APP: last_order_result = "✓ Order executed!"
    end
    
    rect rgb(255, 240, 230)
        Note over APP,GUI: GUI Feedback
        APP->>GUI: Update chart with breakline
        GUI->>GUI: Draw vertical line at execution point
        GUI->>GUI: Display success message
    end
```

---

## Account Information Fetching

### MQL5 Account Info API

```mermaid
flowchart LR
    subgraph MT5_SESSION["MT5 Authenticated Session"]
        AUTH[Authenticated User Session]
        ACC_STATE[(Account State<br/>• Balance<br/>• Equity<br/>• Margin<br/>• Free Margin<br/>• Leverage<br/>• Currency)]
    end
    
    subgraph MQL5_API["MQL5 Account API"]
        API1[AccountInfoDouble<br/>ACCOUNT_BALANCE]
        API2[AccountInfoDouble<br/>ACCOUNT_EQUITY]
        API3[AccountInfoDouble<br/>ACCOUNT_MARGIN]
        API4[AccountInfoDouble<br/>ACCOUNT_MARGIN_FREE]
    end
    
    subgraph EA_CODE["Expert Advisor Code"]
        FETCH["Lines 366-369:<br/>double balance = AccountInfoDouble(ACCOUNT_BALANCE);<br/>double equity = AccountInfoDouble(ACCOUNT_EQUITY);<br/>double margin = AccountInfoDouble(ACCOUNT_MARGIN);<br/>double freeMargin = AccountInfoDouble(ACCOUNT_MARGIN_FREE);"]
    end
    
    AUTH --> ACC_STATE
    ACC_STATE --> API1
    ACC_STATE --> API2
    ACC_STATE --> API3
    ACC_STATE --> API4
    
    API1 --> FETCH
    API2 --> FETCH
    API3 --> FETCH
    API4 --> FETCH
    
    style AUTH fill:#ccffcc,stroke:#00cc00,stroke-width:2px
    style ACC_STATE fill:#e6f3ff,stroke:#0066cc,stroke-width:2px
    style FETCH fill:#fff9e6,stroke:#ffcc00,stroke-width:2px
```

### Account Info Constants (MQL5)

| Constant | Type | Description |
|----------|------|-------------|
| `ACCOUNT_BALANCE` | double | Account balance in deposit currency |
| `ACCOUNT_EQUITY` | double | Account equity (balance + floating P/L) |
| `ACCOUNT_MARGIN` | double | Margin currently used |
| `ACCOUNT_MARGIN_FREE` | double | Free margin available for trading |
| `ACCOUNT_MARGIN_LEVEL` | double | Margin level percentage |
| `ACCOUNT_PROFIT` | double | Current profit on all positions |
| `ACCOUNT_CREDIT` | double | Credit amount |
| `ACCOUNT_LEVERAGE` | long | Account leverage (e.g., 100 for 1:100) |
| `ACCOUNT_CURRENCY` | string | Account currency (e.g., "USD") |

### Rust Account Info Reception

**Lines 338-348: Account Info Update**

```rust
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
```

**Lines 449-466: Account Info Display**

```rust
ui.collapsing("Account Info", |ui| {
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
```

---

## Complete Data Structures

### JSON Tick Data Format (PUB/SUB Port 5555)

```json
{
    "symbol": "XAUUSDc",
    "bid": 2650.55,
    "ask": 2650.75,
    "time": 1706284800,
    "volume": 100,
    "balance": 10000.00,
    "equity": 10150.25,
    "margin": 500.00,
    "free_margin": 9650.25,
    "min_lot": 0.01,
    "max_lot": 100.00,
    "lot_step": 0.01,
    "positions": [
        {
            "ticket": 12345678,
            "type": "BUY",
            "volume": 0.10,
            "price": 2645.50,
            "profit": 50.50
        },
        {
            "ticket": 12345679,
            "type": "SELL",
            "volume": 0.05,
            "price": 2655.00,
            "profit": -25.00
        }
    ],
    "orders": [
        {
            "ticket": 87654321,
            "type": "BUY LIMIT",
            "volume": 0.05,
            "price": 2600.00
        },
        {
            "ticket": 87654322,
            "type": "SELL STOP",
            "volume": 0.10,
            "price": 2700.00
        }
    ]
}
```

### JSON Order Request Format (REQ/REP Port 5556)

**Market Order Request**:
```json
{
    "type": "market_buy",
    "symbol": "XAUUSDc",
    "volume": 0.01,
    "price": 0.0,
    "ticket": 0
}
```

**Pending Order Request**:
```json
{
    "type": "limit_buy",
    "symbol": "XAUUSDc",
    "volume": 0.05,
    "price": 2600.00,
    "ticket": 0
}
```

**Close Position Request**:
```json
{
    "type": "close_position",
    "symbol": "XAUUSDc",
    "volume": 0.0,
    "price": 0.0,
    "ticket": 12345678
}
```

**History Download Request**:
```json
{
    "type": "download_history",
    "symbol": "XAUUSDc",
    "volume": 0.0,
    "price": 0.0,
    "ticket": 0,
    "timeframe": "M1",
    "start": "2024.01.01",
    "end": "2024.01.31",
    "mode": "OHLC",
    "request_id": 1
}
```

### JSON Order Response Format

**Success Response**:
```json
{
    "success": true,
    "ticket": 12345678
}
```

**Failure Response**:
```json
{
    "success": false,
    "error": "Error 10019: Not enough money"
}
```

**History Download Success Response**:
```json
{
    "success": true,
    "message": "1000 records||CSV_DATA||Time,Open,High,Low,Close,TickVol,Spread|NL|2024.01.01 00:00,2650.50,2651.00,2650.00,2650.75,100,3|NL|..."
}
```

---

## ZeroMQ Layer Details

### Socket Patterns

```mermaid
flowchart TB
    subgraph PUB_SUB["PUB/SUB Pattern (Port 5555)"]
        direction LR
        PUB[Publisher<br/>ZmqPublisher.mq5]
        SUB1[Subscriber 1<br/>Rust App]
        SUB2[Subscriber 2<br/>Other Apps]
        
        PUB -->|Broadcast| SUB1
        PUB -->|Broadcast| SUB2
    end
    
    subgraph REQ_REP["REQ/REP Pattern (Port 5556)"]
        direction LR
        REQ[Request<br/>Rust App]
        REP[Reply<br/>ZmqPublisher.mq5]
        
        REQ <-->|Synchronous| REP
    end
    
    style PUB fill:#ccffcc,stroke:#00cc00,stroke-width:2px
    style REP fill:#ffe6cc,stroke:#ff9900,stroke-width:2px
```

### Socket Configuration

**PUB Socket (EA Side)**:
```mql5
g_publisher = new CZmq();
g_publisher.Init(ZMQ_PUB);
g_publisher.Bind("tcp://0.0.0.0:5555");  // Bind to all interfaces
g_publisher.Send(json, true);            // Non-blocking send
```

**SUB Socket (Rust Side)**:
```rust
let mut socket = zeromq::SubSocket::new();
socket.connect("tcp://127.0.0.1:5555").await;  // Connect to localhost
socket.subscribe("").await;                     // Subscribe to all messages
let msg = socket.recv().await;                  // Blocking receive
```

**REP Socket (EA Side)**:
```mql5
g_responder = new CZmq();
g_responder.Init(ZMQ_REP);
g_responder.Bind("tcp://0.0.0.0:5556");     // Bind to all interfaces
string request = g_responder.Receive(true);  // Non-blocking receive
g_responder.Send(response, false);           // Blocking send (REP pattern)
```

**REQ Socket (Rust Side)**:
```rust
let mut socket = zeromq::ReqSocket::new();
socket.connect("tcp://127.0.0.1:5556").await;  // Connect to localhost
socket.send(json_request.into()).await;        // Blocking send
let msg = socket.recv().await;                  // Blocking receive
```

---

## Async Task Management

### Tokio Runtime Architecture

```mermaid
flowchart TB
    subgraph TOKIO["Tokio Async Runtime"]
        MAIN[tokio::main]
        
        subgraph TASKS["Spawned Tasks"]
            TICK_TASK[Tick Subscriber Task<br/>Lines 731-763]
            ORDER_TASK[Order Handler Task<br/>Lines 768-835]
        end
        
        subgraph CHANNELS["MPSC Channels"]
            TICK_CH[Tick Channel<br/>capacity: 100]
            ORDER_CH[Order Channel<br/>capacity: 10]
            RESP_CH[Response Channel<br/>capacity: 10]
        end
    end
    
    subgraph EGUI["eframe GUI (Blocking)"]
        APP[Mt5ChartApp::update]
    end
    
    MAIN --> TICK_TASK
    MAIN --> ORDER_TASK
    MAIN --> EGUI
    
    TICK_TASK --> TICK_CH
    ORDER_TASK <--> ORDER_CH
    ORDER_TASK <--> RESP_CH
    
    TICK_CH --> APP
    APP --> ORDER_CH
    RESP_CH --> APP
    
    style TOKIO fill:#e6f3ff,stroke:#0066cc,stroke-width:2px
    style EGUI fill:#ffe6f0,stroke:#cc0066,stroke-width:2px
```

### Tick Subscriber Task (Lines 731-763)

```rust
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
```

### Order Handler Task (Lines 768-835)

```rust
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
        
        // Send request (blocking in REQ/REP pattern)
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
        
        // Wait for response (blocking in REQ/REP pattern)
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
```

---

## File Structure & Dependencies

### Complete Directory Structure

```
SUM3API/
├── MQL5/
│   ├── Experts/
│   │   └── ZmqPublisher.mq5           # Main EA (451 lines, 19 KB)
│   ├── Include/
│   │   └── Zmq/
│   │       └── Zmq.mqh                # ZMQ wrapper (145 lines, 4 KB)
│   └── Libraries/
│       ├── libzmq.dll                 # ZeroMQ native library
│       └── libsodium.dll              # Crypto library (ZMQ dependency)
│
└── Rustmt5-chart/
    ├── Cargo.toml                     # Rust dependencies
    ├── Cargo.lock                     # Dependency lock file (117 KB)
    ├── src/
    │   └── main.rs                    # Main application (853 lines, 35 KB)
    ├── output/                        # CSV output directory
    │   ├── History_*.csv              # Downloaded historical data
    │   └── Live_*.csv                 # Live recorded tick data
    └── target/                        # Build artifacts
        ├── debug/                     # Debug build
        └── release/                   # Release build

```

### Dependency Graph

```mermaid
flowchart TB
    subgraph MQL5_DEPS["MQL5 Dependencies"]
        EA[ZmqPublisher.mq5]
        ZMQ_MQH[Zmq.mqh]
        TRADE_MQH[Trade.mqh<br/>MT5 Built-in]
        LIBZMQ[libzmq.dll]
        LIBSODIUM[libsodium.dll]
    end
    
    subgraph RUST_DEPS["Rust Dependencies"]
        MAIN[main.rs]
        EFRAME[eframe 0.27]
        EGUI[egui 0.27]
        EGUI_PLOT[egui_plot 0.27]
        SERDE[serde 1.0]
        SERDE_JSON[serde_json 1.0]
        TOKIO[tokio 1.x]
        ZEROMQ[zeromq 0.3]
        CHRONO[chrono 0.4]
    end
    
    EA --> ZMQ_MQH
    EA --> TRADE_MQH
    ZMQ_MQH --> LIBZMQ
    LIBZMQ --> LIBSODIUM
    
    MAIN --> EFRAME
    MAIN --> EGUI_PLOT
    MAIN --> SERDE
    MAIN --> SERDE_JSON
    MAIN --> TOKIO
    MAIN --> ZEROMQ
    MAIN --> CHRONO
    EFRAME --> EGUI
    
    style EA fill:#e6f3ff,stroke:#0066cc,stroke-width:2px
    style MAIN fill:#ffe6f0,stroke:#cc0066,stroke-width:2px
```

---

## Summary

This document provides a complete end-to-end technical specification of the MQL5 ↔ ZeroMQ ↔ Rust trading system, including:

**Security Architecture**: Credential-free design with session inheritance  
**Account Information Flow**: From MT5 API to Rust GUI  
**Complete Data Structures**: JSON formats and Rust/MQL5 types  
**Communication Patterns**: PUB/SUB and REQ/REP with sequence diagrams  
**Async Task Management**: Tokio runtime and channel architecture  
**Micro-level Implementation**: Line-by-line code references  
**File Structure**: Complete dependency graph  

**Key Security Advantage**: Unlike MT5's Python API which requires explicit credentials in code, this system leverages MT5's authenticated session, eliminating credential exposure entirely.

---

