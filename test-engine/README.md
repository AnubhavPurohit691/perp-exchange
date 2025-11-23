# Trading Bot - Test Engine

A trading bot script that tests the http-backend by creating users, placing orders, and setting up trades that lead to liquidation.

## How It Works

1. **Creates Users**: Uses `POST /user` to create multiple traders
2. **Gets User IDs**: Receives userid from each user creation response
3. **Places Orders**: Uses `POST /orderbook` with the userid to create orders
4. **Creates Trades**: Orders match and execute, creating positions
5. **Triggers Liquidations**: High leverage positions are created that will liquidate when mark price moves

## Usage

### Prerequisites

1. Make sure the http-backend is running on `http://localhost:3000`
   ```bash
   cargo run --package http-backend
   ```

### Running the Bot

Run the trading bot:
```bash
cargo run --package test-engine
```

Or use the test script:
```bash
./test-engine/test.sh
```

## Bot Flow

### Step 1: Create Users
- Creates 6 traders (Trader1 through Trader6)
- Each user gets a unique userid

### Step 2: Create Trades
The bot creates matching buy/sell orders that execute trades:

- **Trade 1**: User1 (LONG 1.0 BTC @ 50000, 10x) vs User2 (SHORT 1.0 BTC @ 50000, 20x)
- **Trade 2**: User3 (LONG 2.0 BTC @ 50000, 50x) vs User4 (SHORT 2.0 BTC @ 50000, 15x)
- **Trade 3**: User5 (SHORT 1.5 BTC @ 50000, 50x) vs User1 (LONG 1.5 BTC @ 50000, 10x)
- **Trade 4**: User6 (SHORT 0.5 BTC @ 49950, 12x) vs User2 (LONG 0.5 BTC @ 50000, 20x)

### Step 3: Monitor Liquidations
- Waits 20 seconds for funding rates and potential liquidations
- High leverage positions (50x) are most likely to liquidate

## Positions Created

- **User1**: LONG 2.5 BTC @ 50000 (10x leverage)
- **User2**: SHORT 1.5 BTC @ 50000 (20x leverage)
- **User3**: LONG 2.0 BTC @ 50000 (50x leverage) - **HIGH RISK**
- **User4**: SHORT 2.0 BTC @ 50000 (15x leverage)
- **User5**: SHORT 1.5 BTC @ 50000 (50x leverage) - **HIGH RISK**
- **User6**: SHORT 0.5 BTC @ 49950 (12x leverage)

## Observing Results

**Bot output shows:**
- User creation confirmations
- Order placement confirmations
- Trade execution messages
- Waiting periods for liquidations

**http-backend server logs show:**
- `LIQUIDATION EVENT` messages when positions are liquidated
- Funding rate applications (every 5 seconds)
- Trade executions

## Liquidation Triggers

Positions liquidate when:
1. Mark price moves against the position
2. Margin ratio falls below 1% (maintenance margin)
3. Funding rate payments drain equity below threshold

High-risk positions (50x leverage) will liquidate quickly if price moves against them.

