use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use reqwest::Client;
use std::sync::Arc;
use std::time::Instant;
use tokio::task;

#[tokio::main]
async fn main() {
    // Change this to your API endpoint
    let url = Arc::new("http://127.0.0.1:3000/user".to_string());

    let client = Arc::new(Client::new());
    let threads = 10; // number of parallel tasks
    let requests_per_thread = 100; // requests per thread
    let total_requests = threads * requests_per_thread;

    let start = Instant::now();
    let mut handles = Vec::new();

    for i in 0..threads {
        let url = url.clone();
        let client = client.clone();

        let handle = task::spawn(async move {
            // Seed with thread index (for unique RNG per thread)
            let mut rng = StdRng::seed_from_u64(i as u64 + 12345);

            for _ in 0..requests_per_thread {
                let random_name: String = (0..8)
                    .map(|_| (rng.random_range(b'a'..=b'z') as char))
                    .collect();

                let body = serde_json::json!({ "name": random_name });

                let _ = client.post(&*url).json(&body).send().await;
            }
        });

        handles.push(handle);
    }

    for h in handles {
        let _ = h.await;
    }

    let duration = start.elapsed();
    let rps = total_requests as f64 / duration.as_secs_f64();

    println!("Total requests: {}", total_requests);
    println!("Time taken: {:.2?}", duration);
    println!("Requests per second: {:.2}", rps);
}
