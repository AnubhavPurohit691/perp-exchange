import { Pool } from "pg";
import { config } from "../config.ts";
import type { NormalizedTrade } from "../types.ts";
import { bucketStart } from "../utils/trade.ts";

const db = new Pool({ connectionString: config.timescaleUrl });
db.on("error", (err: unknown) => {
  console.error("timescale connection error", err);
});

const insertTradeSql = `
INSERT INTO trades (symbol, price, quantity, buy_user, sell_user, ts, raw)
VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb)
`;

const upsertCandleSql = `
INSERT INTO candles (
  symbol, timeframe, bucket_start, open, high, low, close, volume, trade_count, open_ts, close_ts
)
VALUES ($1, $2, $3, $4, $4, $4, $4, $5, 1, $6, $6)
ON CONFLICT (symbol, timeframe, bucket_start)
DO UPDATE
SET
  high = GREATEST(candles.high, EXCLUDED.high),
  low = LEAST(candles.low, EXCLUDED.low),
  open = CASE
    WHEN EXCLUDED.open_ts < candles.open_ts THEN EXCLUDED.open
    ELSE candles.open
  END,
  open_ts = LEAST(candles.open_ts, EXCLUDED.open_ts),
  close = CASE
    WHEN EXCLUDED.close_ts >= candles.close_ts THEN EXCLUDED.close
    ELSE candles.close
  END,
  close_ts = GREATEST(candles.close_ts, EXCLUDED.close_ts),
  volume = candles.volume + EXCLUDED.volume,
  trade_count = candles.trade_count + EXCLUDED.trade_count
`;

export async function initSchema(): Promise<void> {
  await db.query("CREATE EXTENSION IF NOT EXISTS timescaledb");
  await db.query(`
    CREATE TABLE IF NOT EXISTS trades (
      id BIGSERIAL,
      symbol TEXT NOT NULL,
      price NUMERIC(30, 10) NOT NULL,
      quantity NUMERIC(30, 10) NOT NULL,
      buy_user TEXT,
      sell_user TEXT,
      ts TIMESTAMPTZ NOT NULL,
      raw JSONB NOT NULL,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      PRIMARY KEY (id, ts)
    )
  `);

  try {
    await db.query("SELECT create_hypertable('trades', 'ts', if_not_exists => TRUE)");
  } catch (err: unknown) {
    const code = (err as { code?: string })?.code;
    if (code === "TS103") {
      // Table already exists with old PK (id only). Migrate to (id, ts) then retry.
      await db.query("ALTER TABLE trades DROP CONSTRAINT IF EXISTS trades_pkey");
      await db.query("ALTER TABLE trades ADD PRIMARY KEY (id, ts)");
      await db.query("SELECT create_hypertable('trades', 'ts', if_not_exists => TRUE)");
    } else {
      try {
        await db.query("SELECT create_hypertable('trades', by_range('ts'), if_not_exists => TRUE)");
      } catch {
        throw err;
      }
    }
  }

  await db.query(`
    CREATE TABLE IF NOT EXISTS candles (
      symbol TEXT NOT NULL,
      timeframe TEXT NOT NULL,
      bucket_start TIMESTAMPTZ NOT NULL,
      open NUMERIC(30, 10) NOT NULL,
      high NUMERIC(30, 10) NOT NULL,
      low NUMERIC(30, 10) NOT NULL,
      close NUMERIC(30, 10) NOT NULL,
      volume NUMERIC(30, 10) NOT NULL,
      trade_count BIGINT NOT NULL DEFAULT 0,
      open_ts TIMESTAMPTZ NOT NULL,
      close_ts TIMESTAMPTZ NOT NULL,
      PRIMARY KEY (symbol, timeframe, bucket_start)
    )
  `);
}

export async function writeTradeAndCandles(
  trade: NormalizedTrade,
  raw: string,
): Promise<void> {
  const conn = await db.connect();
  try {
    await conn.query("BEGIN");

    await conn.query(insertTradeSql, [
      trade.symbol,
      trade.price,
      trade.quantity,
      trade.buyUser,
      trade.sellUser,
      trade.timestamp.toISOString(),
      raw,
    ]);

    for (const minutes of config.candleTimeframes) {
      await conn.query(upsertCandleSql, [
        trade.symbol,
        `${minutes}m`,
        bucketStart(trade.timestamp, minutes),
        trade.price,
        trade.quantity,
        trade.timestamp.toISOString(),
      ]);
    }

    await conn.query("COMMIT");
  } catch (err) {
    await conn.query("ROLLBACK");
    throw err;
  } finally {
    conn.release();
  }
}
