⚡ Performance Benchmark

The core strength of PerpX lies in its raw performance, powered by a custom asynchronous engine optimized for concurrent workloads.
The following benchmark was conducted on a Lenovo loq laptop:

Duration: 2 minutes

Requests: 612,540

Throughput: ~7,418 requests/sec

Avg. Latency: ~0.17 ms

Failures: 0

CPU Usage: ~68%

Memory Usage: ~11 MB

These figures demonstrate the engine’s ability to handle massive trading volumes with sub-millisecond latency and minimal overhead, making it ideal for real-time trading environments.

🏗 Architecture & Concurrency

PerpX is built around modern concurrency primitives for deterministic performance and predictable scaling.

Multithreaded Engine Core — Separate threads handle user management, order matching, and liquidation, isolating workloads efficiently.

Pinned Worker Threads — Each critical thread is manually pinned to system cores for deterministic latency and throughput.

Async Communication Layer — Internal components communicate through Tokio MPSC channels and oneshot responses for high-throughput message passing.

Lock-Minimized Design — Shared-nothing architecture for the critical path; no global mutexes on the order matching layer.

Backpressure Safety — Channels are bounded, ensuring controlled memory growth during bursts.

🧩 Current Implementation

The backend currently supports a robust set of core exchange systems:

⚙️ Matching Engine

Built on top of BTreeMap-based order books for O(log n) insertion and matching.

Maintains price-time priority for fair matching.

Handles partial and full order executions.

💰 Margin & Position System

Isolated Margin Model: Each position operates independently, avoiding cross-user contagion.

Dynamic Liquidation Logic: Automatically liquidates undercollateralized positions based on live mark prices.

Leverage Support: Each position can use individual leverage settings.

🔥 Liquidation Engine

Continuously monitors all positions.

Triggers liquidation when margin ratio < maintenance threshold.

Ensures market stability by closing risky positions first.

💸 Funding Rate Engine

Periodically calculates funding payments between longs and shorts.

Uses the difference between Mark Price and Index Price (from Binance) to maintain price parity with the spot market.

Longs pay shorts or vice versa, depending on price imbalance.

🌐 High-Performance HTTP + WebSocket Layer

Built using Axum + Tokio for ultra-low latency.

/user endpoints handle user creation and balance setup.

/orderbook handles placing, canceling, and matching orders.

WebSocket stream pushes real-time order book, funding updates, and liquidation events.

🧮 Price Sources
Index Price

Aggregated from Binance Spot Market (e.g., BTCUSDT).

Represents the fair global price reference.

Mark Price

Derived from Binance Futures Mark Price feed.

Used internally for margin calculation, liquidation, and funding rate settlement.

Smoothens volatility and prevents manipulation during thin liquidity.

🚧 What’s Next (Contributions Welcome!)

While the backend engine is stable and optimized, some features are still under development or open to contributors:

Frontend Interface — Interactive web UI for traders.

Persistent Storage — Database integration for trades and account histories.

Advanced Orders — Stop-limit, iceberg, and trigger-based orders.

Enhanced Risk Module — Cross-margin and portfolio-level margining.

🚀 Getting Started
Prerequisites

Rust (latest stable)

Cargo build tool

Installation
git clone https://github.com/anubhavpurohit/perp-exchange
cd perpx-backend

Running the Server
cargo run --release

The server will start listening on localhost:8080 and begin accepting HTTP + WebSocket connections.

⚔️ Benchmarking

A dedicated benchmarking client is included to stress test user creation and order matching performance.

cd benchmark-client
cargo run --release

This tool rapidly creates users, opens WebSocket connections, and places synthetic orders to test throughput and latency in real-time.

🛠 Tech Stack

Language: Rust

Runtime: Tokio

Networking: Axum (HTTP) + Tungstenite (WebSocket)

Concurrency: MPSC + Oneshot channels

Data Structures: BTreeMap for order books

Serialization: Serde + JSON

Metrics: Custom benchmarking client for load and latency profiling
