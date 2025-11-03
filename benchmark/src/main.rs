use futures::stream::{FuturesUnordered, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use reqwest::Client;
use serde_json::json;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    // Base API endpoint
    let base_url = "http://localhost:3000";
    let user_url = format!("{}/user", base_url);
    let orderbook_url = format!("{}/orderbook", base_url);

    let client = Client::new();

    // Benchmark parameters
    let total_orders = 100_000;
    let concurrent_orders = 1000;
    let leverage = 20.0;

    println!(
        "🚀 Starting benchmark on {}\nOrders: {}, Concurrency: {}\n",
        base_url, total_orders, concurrent_orders
    );

    // Step 1️⃣ — Create one user and extract user ID
    let user_payload = json!({ "name": "benchmark_user" });

    let userid = match client.post(&user_url).json(&user_payload).send().await {
        Ok(resp) => match resp.text().await {
            Ok(id) => {
                println!("✅ User created: {}\n", id);
                id
            }
            Err(_) => panic!("❌ Failed to get user ID from response"),
        },
        Err(e) => panic!("❌ Failed to create user: {}", e),
    };

    // Step 2️⃣ — Prepare benchmark
    let pb = ProgressBar::new(total_orders as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let start_time = Instant::now();
    let mut handles = FuturesUnordered::new();
    let mut latencies: Vec<Duration> = Vec::with_capacity(total_orders);
    let mut success = 0;

    // Step 3️⃣ — Send concurrent order requests
    for i in 0..total_orders {
        let client = client.clone();
        let orderbook_url = orderbook_url.clone();
        let userid = userid.clone();

        let side = if i % 2 == 0 { "Buy" } else { "Sell" };
        let price = 50000.0 + rand::thread_rng().gen_range(-500.0..500.0);
        let quantity = rand::thread_rng().gen_range(0.01..1.0);

        let payload = json!({
            "userid": userid,
            "symbol": "BTCUSDT",
            "side": side,
            "quantity": format!("{:.4}", quantity),
            "trade_price": format!("{:.2}", price),
            "leverage": leverage.to_string(),
        });

        handles.push(async move {
            let start = Instant::now();
            let res = client.post(&orderbook_url).json(&payload).send().await;
            let latency = start.elapsed();
            (res.is_ok(), latency)
        });

        // Limit concurrency
        if handles.len() >= concurrent_orders {
            if let Some((ok, latency)) = handles.next().await {
                if ok {
                    success += 1;
                    latencies.push(latency);
                }
                pb.inc(1);
            }
        }
    }

    // Drain remaining
    while let Some((ok, latency)) = handles.next().await {
        if ok {
            success += 1;
            latencies.push(latency);
        }
        pb.inc(1);
    }

    pb.finish();

    // Step 4️⃣ — Print results
    let elapsed = start_time.elapsed();
    let avg_latency = if latencies.is_empty() {
        Duration::from_millis(0)
    } else {
        latencies.iter().sum::<Duration>() / (latencies.len() as u32)
    };

    let throughput = total_orders as f64 / elapsed.as_secs_f64();

    println!("\n📊 BENCHMARK RESULTS");
    println!("-----------------------------");
    println!("User ID: {}", userid);
    println!("Total requests: {}", total_orders);
    println!("Successful: {}", success);
    println!("Total time: {:.2?}", elapsed);
    println!("Average latency: {:.2?}", avg_latency);
    println!("Throughput: {:.2} req/sec", throughput);
    println!("-----------------------------");
}
