use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

const HTTP_BACKEND_URL: &str = "http://localhost:3000";

#[derive(Debug, Serialize)]
struct UserRequest {
    name: String,
}

#[derive(Debug, Deserialize)]
struct UserData {
    userid: String,
    name: String,
    #[allow(dead_code)]
    balance: Decimal,
    #[allow(dead_code)]
    quantity: Decimal,
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    #[serde(rename = "Ok")]
    ok: Option<UserData>,
    #[serde(rename = "Err")]
    err: Option<String>,
}

#[derive(Debug, Serialize)]
struct OrderRequest {
    price: Decimal,
    symbol: String,
    quantity: Decimal,
    ordertype: String,
    userid: String,
    leverage: Decimal,
}

struct TradingBot {
    client: reqwest::Client,
    base_url: String,
}

impl TradingBot {
    fn new(base_url: &str) -> Self {
        TradingBot {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
        }
    }

    async fn check_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/user", self.base_url);
        let response = self.client.get(&url).send().await;

        match response {
            Ok(resp) => {
                if resp.status().as_u16() == 405
                    || resp.status().is_client_error()
                    || resp.status().is_server_error()
                {
                    Ok(())
                } else {
                    Err("Unexpected response from server".into())
                }
            }
            Err(e) => Err(format!(
                "Cannot connect to http-backend at {}. Make sure it's running.\nError: {}",
                self.base_url, e
            )
            .into()),
        }
    }

    async fn create_user(&self, name: &str) -> Result<UserData, Box<dyn std::error::Error>> {
        let url = format!("{}/user", self.base_url);
        let user_req = UserRequest {
            name: name.to_string(),
        };

        let response = self.client.post(&url).json(&user_req).send().await?;

        if response.status().is_success() {
            let result: UserResponse = response.json().await?;
            match result.ok {
                Some(user) => {
                    println!("[BOT] Created user: {} (ID: {})", user.name, user.userid);
                    Ok(user)
                }
                None => {
                    let err_msg = result.err.unwrap_or_else(|| "Unknown error".to_string());
                    Err(format!("Failed to create user: {}", err_msg).into())
                }
            }
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to create user: {}", error_text).into())
        }
    }

    async fn create_order(
        &self,
        price: Decimal,
        symbol: &str,
        quantity: Decimal,
        ordertype: &str,
        userid: &str,
        leverage: Decimal,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/orderbook", self.base_url);
        let order_req = OrderRequest {
            price,
            symbol: symbol.to_string(),
            quantity,
            ordertype: ordertype.to_string(),
            userid: userid.to_string(),
            leverage,
        };

        let response = self.client.post(&url).json(&order_req).send().await?;

        if response.status().is_success() {
            // Response is either {"Ok": "message"} or {"Err": "error"}
            let json: serde_json::Value = response.json().await?;
            if let Some(ok_msg) = json.get("Ok").and_then(|v| v.as_str()) {
                println!(
                    "[BOT] Order placed: {} {} {} @ {} with {}x leverage (User: {})",
                    ordertype.to_uppercase(),
                    quantity,
                    symbol,
                    price,
                    leverage,
                    &userid[..8]
                );
                Ok(ok_msg.to_string())
            } else if let Some(err_msg) = json.get("Err").and_then(|v| v.as_str()) {
                Err(format!("Order creation failed: {}", err_msg).into())
            } else {
                Err("Unexpected response format".into())
            }
        } else {
            let error_text = response.text().await?;
            Err(format!("HTTP error creating order: {}", error_text).into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Trading Bot: Liquidation Test Script ===\n");
    println!("Target: {}\n", HTTP_BACKEND_URL);

    let bot = TradingBot::new(HTTP_BACKEND_URL);

    // Check if server is running
    println!("[BOT] Checking connection to http-backend...");
    match bot.check_connection().await {
        Ok(_) => println!("[BOT] Connected successfully!\n"),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            eprintln!("\nPlease start the http-backend first:");
            eprintln!("  cargo run --package http-backend");
            return Err(e);
        }
    }

    // Step 1: Create users first
    println!("=== Step 1: Creating Users ===");
    let user1 = bot.create_user("Trader1").await?;
    let user2 = bot.create_user("Trader2").await?;
    let user3 = bot.create_user("Trader3").await?;
    let user4 = bot.create_user("Trader4").await?;
    let user5 = bot.create_user("Trader5").await?;
    println!();

    // Step 2: Create orders that will match and create positions
    println!("=== Step 2: Creating Trades (Orders that will match) ===");

    // Trade 1: User1 buys from User2 (creates LONG for User1, SHORT for User2)
    println!("\n[Trade 1] User1 buying from User2...");
    let _ = bot
        .create_order(
            Decimal::from_str("50000.0")?,
            "btc",
            Decimal::from_str("1.0")?,
            "buy",
            &user1.userid,
            Decimal::from_str("10.0")?, // 10x leverage
        )
        .await?;

    sleep(Duration::from_millis(300)).await;

    let _ = bot
        .create_order(
            Decimal::from_str("50000.0")?,
            "btc",
            Decimal::from_str("1.0")?,
            "sell",
            &user2.userid,
            Decimal::from_str("20.0")?, // 20x leverage
        )
        .await?;

    sleep(Duration::from_millis(500)).await;

    // Trade 2: User3 creates high leverage LONG position (likely to liquidate)
    println!("\n[Trade 2] User3 creating high-risk LONG position...");
    let _ = bot
        .create_order(
            Decimal::from_str("50000.0")?,
            "btc",
            Decimal::from_str("2.0")?,
            "buy",
            &user3.userid,
            Decimal::from_str("50.0")?, // 50x leverage - very risky
        )
        .await?;

    sleep(Duration::from_millis(300)).await;

    let _ = bot
        .create_order(
            Decimal::from_str("50000.0")?,
            "btc",
            Decimal::from_str("2.0")?,
            "sell",
            &user4.userid,
            Decimal::from_str("15.0")?,
        )
        .await?;

    sleep(Duration::from_millis(500)).await;

    // Trade 3: User5 creates another high leverage position
    println!("\n[Trade 3] User5 creating high-risk SHORT position...");
    let _ = bot
        .create_order(
            Decimal::from_str("50000.0")?,
            "btc",
            Decimal::from_str("1.5")?,
            "sell",
            &user5.userid,
            Decimal::from_str("50.0")?, // 50x leverage - very risky
        )
        .await?;

    sleep(Duration::from_millis(300)).await;

    let _ = bot
        .create_order(
            Decimal::from_str("50000.0")?,
            "btc",
            Decimal::from_str("1.5")?,
            "buy",
            &user1.userid,
            Decimal::from_str("10.0")?,
        )
        .await?;

    sleep(Duration::from_millis(500)).await;

    println!("\n=== Positions Created ===");
    println!("User1: LONG 2.5 BTC @ 50000 (10x leverage)");
    println!("User2: SHORT 1.0 BTC @ 50000 (20x leverage)");
    println!("User3: LONG 2.0 BTC @ 50000 (50x leverage) - HIGH RISK");
    println!("User4: SHORT 2.0 BTC @ 50000 (15x leverage)");
    println!("User5: SHORT 1.5 BTC @ 50000 (50x leverage) - HIGH RISK");
    println!();

    // Step 3: Wait for funding rates and potential liquidations
    println!("=== Step 3: Monitoring for Liquidations ===");
    println!("Funding rates are applied every 5 seconds.");
    println!("Liquidations occur when:");
    println!("  - Mark price moves against position");
    println!("  - Margin ratio falls below 1% (maintenance margin)");
    println!("  - Funding rate payments drain equity");
    println!("\nHigh-risk positions (User3 LONG, User5 SHORT) are likely to liquidate");
    println!("if mark price moves significantly against them.");
    println!("\nWaiting 20 seconds to observe liquidations...");
    println!("(Check http-backend logs for liquidation events)\n");

    for i in 1..=4 {
        sleep(Duration::from_secs(5)).await;
        println!(
            "[BOT] Waiting... {} seconds elapsed (funding rate applied {} times)",
            i * 5,
            i
        );
    }

    println!("\n=== Step 4: Creating Additional Trades ===");

    // Create more trades to test matching engine
    let user6 = bot.create_user("Trader6").await?;

    println!("\n[Trade 4] Creating additional matching orders...");
    let _ = bot
        .create_order(
            Decimal::from_str("49950.0")?,
            "btc",
            Decimal::from_str("0.5")?,
            "sell",
            &user6.userid,
            Decimal::from_str("12.0")?,
        )
        .await?;

    sleep(Duration::from_millis(300)).await;

    let _ = bot
        .create_order(
            Decimal::from_str("50000.0")?,
            "btc",
            Decimal::from_str("0.5")?,
            "buy",
            &user2.userid,
            Decimal::from_str("20.0")?,
        )
        .await?;

    sleep(Duration::from_millis(500)).await;

    println!("\n=== Summary ===");
    println!("Bot script completed!");
    println!("\nCreated:");
    println!("  - 6 users");
    println!("  - Multiple trades that established positions");
    println!("  - High leverage positions that may liquidate");
    println!("\nMonitor http-backend logs for:");
    println!("  - LIQUIDATION EVENT messages (when positions liquidate)");
    println!("  - Funding rate applications (every 5 seconds)");
    println!("  - Trade executions");
    println!("\nHigh-risk positions (50x leverage) are most likely to liquidate");
    println!("when mark price moves against them.");

    Ok(())
}
