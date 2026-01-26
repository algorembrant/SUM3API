# MT5 Tick Volume & X-Axis Formulation

## X-Axis Time Representation
The chart X-axis now represents **real-time** timestamps rather than a fixed interval index.
-   **Method**: `egui_plot` is fed `(timestamp, value)` pairs.
-   **Precision**: Seconds (Unix timestamp).
-   **Visual**: Formatted as `HH:MM:SS` on the axis.

## Tick Volume Logic
In MetaTrader 5 (MT5), tick volume serves as a proxy for market activity in decentralized markets like forex and indices, where true traded volume data isn't available.

### Calculation
It is calculated as the **total number of price changes (known as ticks)** that occur within the timeframe of each bar on the chart.
-   Each tick represents an update to the bid or ask price received from the broker's data feed.
-   **1-Minute Chart Example**: The tick volume for a bar is the count of individual price updates that happened in that specific minute.
-   This count accumulates over the bar's formation.

### Architecture Implementation
1.  **MQL5 Source**: `SymbolInfoTick` provides the accumulated `tick.volume` for the current session/bar (depending on exact MQL5 property used, usually `last_tick.volume` or `iVolume`).
    -   *Current Implementation*: We use `tick.volume` from `SymbolInfoTick`. Note that `MqlTick.volume` is often the volume of the *last deal* (for exchange markets) or tick volume (for Forex). For Forex, we might need to rely on `iVolume` or `TimeCurrent` accumulation if `MqlTick.volume` is 0.
    -   *Adjustment*: If `MqlTick.volume` provides the *tick volume* (ticks since start of bar) for Forex symbols, we display that. If it provides *Last Deal Volume* (Exchange), we display that. The chart effectively plots the stream of these volume updates.

2.  **Rust Visualization**:
    -   We store a history of `(time, volume)` tuples.
    -   The Volume Chart plots these as bars, aligning strictly with the time axis of the price chart.
