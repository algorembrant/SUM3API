# MT5 ZeroMQ Chart System - Architecture Diagram

This diagram illustrates the complete data flow and component integration of the MT5-Chart system, including the **DLL layer** that enables ZeroMQ communication in MQL5.

## System Architecture

```mermaid
flowchart TB
    subgraph MT5["MetaTrader 5 Platform"]
        direction TB
        subgraph MQL5Layer["MQL5 Layer"]
            EA["ZmqPublisher.mq5<br/>(Expert Advisor)"]
            Header["Zmq.mqh<br/>(ZMQ Wrapper Class)"]
        end
        
        subgraph DLLLayer["Native DLL Layer"]
            LibZmq["libzmq.dll<br/>(ZeroMQ Native Library)"]
        end
        
        EA -->|"#include"| Header
        Header -->|"#import"| LibZmq
    end

    subgraph RustApp["mt5-chart (Rust Application)"]
        direction TB
        Main["main.rs"]
        
        subgraph Crates["Rust Dependencies"]
            ZmqRust["zeromq crate<br/>(ZMQ Subscriber)"]
            Eframe["eframe + egui<br/>(GUI Framework)"]
            EguiPlot["egui_plot<br/>(Chart Rendering)"]
            Tokio["tokio<br/>(Async Runtime)"]
            Serde["serde + serde_json<br/>(JSON Parsing)"]
        end
        
        Main --> ZmqRust
        Main --> Eframe
        Main --> EguiPlot
        Main --> Tokio
        Main --> Serde
    end

    subgraph Network["ZeroMQ Transport"]
        TCP["tcp://127.0.0.1:5555"]
    end

    LibZmq -->|"PUB Socket<br/>Bind"| TCP
    TCP -->|"SUB Socket<br/>Connect"| ZmqRust
```

## Detailed Component Interaction

```mermaid
sequenceDiagram
    participant MT5 as MetaTrader 5
    participant EA as ZmqPublisher.mq5
    participant MQH as Zmq.mqh (CZmq Class)
    participant DLL as libzmq.dll
    participant ZMQ as ZeroMQ Socket
    participant Rust as mt5-chart (Rust)
    participant GUI as egui Chart

    Note over MT5,GUI: Initialization Phase
    MT5->>EA: OnInit()
    EA->>MQH: new CZmq()
    MQH->>DLL: zmq_ctx_new()
    DLL-->>MQH: context handle
    MQH->>DLL: zmq_socket(context, ZMQ_PUB)
    DLL-->>MQH: socket handle
    EA->>MQH: Bind("tcp://0.0.0.0:5555")
    MQH->>DLL: zmq_bind(socket, endpoint)
    DLL-->>ZMQ: PUB socket listening

    Rust->>Rust: Start tokio runtime
    Rust->>ZMQ: connect("tcp://127.0.0.1:5555")
    Rust->>ZMQ: subscribe("")

    Note over MT5,GUI: Runtime - Tick Data Flow
    loop Every Market Tick
        MT5->>EA: OnTick()
        EA->>EA: SymbolInfoTick(_Symbol, tick)
        EA->>EA: Create JSON {"symbol", "bid", "ask", "time"}
        EA->>MQH: Send(json)
        MQH->>DLL: zmq_send(socket, data, len, flags)
        DLL->>ZMQ: Publish message
        ZMQ->>Rust: Receive message
        Rust->>Rust: Parse JSON to TickData
        Rust->>Rust: mpsc::send(tick)
        Rust->>GUI: Update chart data
        GUI->>GUI: Render Bid/Ask lines
    end

    Note over MT5,GUI: Shutdown Phase
    MT5->>EA: OnDeinit()
    EA->>MQH: Shutdown()
    MQH->>DLL: zmq_close(socket)
    MQH->>DLL: zmq_ctx_term(context)
```

## File Structure & Dependencies

```mermaid
graph LR
    subgraph Repository["SUM3API Repository"]
        subgraph MQL5Dir["MQL5/"]
            ExpertsDir["Experts/"]
            IncludeDir["Include/"]
            LibrariesDir["Libraries/ (required)"]
            
            ExpertsDir --> ZmqPub["ZmqPublisher.mq5"]
            IncludeDir --> ZmqDir["Zmq/"]
            ZmqDir --> ZmqMqh["Zmq.mqh"]
            LibrariesDir --> LibZmqDll["libzmq.dll"]
        end
        
        subgraph Mt5ChartDir["mt5-chart/"]
            CargoToml["Cargo.toml"]
            SrcDir["src/"]
            SrcDir --> MainRs["main.rs"]
        end
        
        README["README.md"]
    end
    
    ZmqPub -.->|"includes"| ZmqMqh
    ZmqMqh -.->|"imports"| LibZmqDll
```

## DLL Import Details

The `Zmq.mqh` header file imports the following functions from `libzmq.dll`:

| Function | Purpose |
|----------|---------|
| `zmq_ctx_new()` | Create new ZMQ context |
| `zmq_ctx_term(context)` | Terminate ZMQ context |
| `zmq_socket(context, type)` | Create socket of specified type |
| `zmq_close(socket)` | Close a socket |
| `zmq_bind(socket, endpoint)` | Bind socket to address |
| `zmq_connect(socket, endpoint)` | Connect socket to address |
| `zmq_send(socket, buf, len, flags)` | Send message |
| `zmq_recv(socket, buf, len, flags)` | Receive message |
| `zmq_errno()` | Get last error code |

> [!IMPORTANT]
> The `libzmq.dll` must be placed in the `MQL5/Libraries/` folder of your MetaTrader 5 data directory for the Expert Advisor to function.

## Data Flow Summary

```mermaid
flowchart LR
    A["MT5 Market Tick"] --> B["ZmqPublisher.mq5"]
    B --> C["CZmq.Send()"]
    C --> D["libzmq.dll"]
    D --> E["TCP Socket<br/>:5555"]
    E --> F["Rust zeromq crate"]
    F --> G["JSON Parser"]
    G --> H["mpsc Channel"]
    H --> I["egui Chart"]
    
    style D fill:#ff6b6b,stroke:#333,color:#fff
    style E fill:#4ecdc4,stroke:#333,color:#fff
