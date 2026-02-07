import { Pool } from "pg";
import { createClient } from "redis";

const redisUrl = process.env.REDIS_URL ?? "redis://127.0.0.1/";
const tradesKey = process.env.TRADES_KEY ?? "trades:btc";
const timescaleUrl =
  process.env.TIMESCALE_URL ??
  process.env.DATABASE_URL ??
  "postgres://postgres:postgres@127.0.0.1:5432/postgres";
const defaultSymbol = process.env.DEFAULT_SYMBOL ?? tradesKey.split(":").pop() ?? "btc";
const candleTimeframes = [1, 5, 10] as const;

type RawTrade = {
  symbol?: string;
  price?: string | number;
  quantity?: string | number;
  buyuser?: string;
  selluser?: string;
  timestamp?: string;
};

const redisClient = createClient({ url: redisUrl });
redisClient.on("error", (err: unknown) => {
  // Keep errors visible; reconnect handled by client.
  console.error("redis error", err);
});

const db = new Pool({ connectionString: timescaleUrl });
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

await initSchema();
await redisClient.connect();
console.log(`listening for trades on ${tradesKey} via ${redisUrl}`);
console.log(`writing trades + candles to ${timescaleUrl}`);

while (true) {
  // BLPOP blocks until a trade is available.
  const res = await redisClient.blPop(tradesKey, 0);
  if (!res) continue;

  const raw = res.element;
  try {
    const trade = normalizeTrade(raw);
    await writeTradeAndCandles(trade, raw);
  } catch (err) {
    console.error("failed to process trade", { raw, err });
  }
}

async function initSchema() {
  await db.query("CREATE EXTENSION IF NOT EXISTS timescaledb");
  await db.query(`
    CREATE TABLE IF NOT EXISTS trades (
      id BIGSERIAL PRIMARY KEY,
      symbol TEXT NOT NULL,
      price NUMERIC(30, 10) NOT NULL,
      quantity NUMERIC(30, 10) NOT NULL,
      buy_user TEXT,
      sell_user TEXT,
      ts TIMESTAMPTZ NOT NULL,
      raw JSONB NOT NULL,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    )
  `);

  try {
    await db.query("SELECT create_hypertable('trades', 'ts', if_not_exists => TRUE)");
  } catch {
    await db.query("SELECT create_hypertable('trades', by_range('ts'), if_not_exists => TRUE)");
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

function normalizeTrade(raw: string) {
  const payload = JSON.parse(raw) as RawTrade;
  const symbol = (payload.symbol ?? defaultSymbol).toLowerCase();

  const price = toDecimalString(payload.price, "price");
  const quantity = toDecimalString(payload.quantity, "quantity");

  const ts = payload.timestamp ? new Date(payload.timestamp) : new Date();
  if (Number.isNaN(ts.getTime())) {
    throw new Error(`invalid timestamp: ${payload.timestamp}`);
  }

  return {
    symbol,
    price,
    quantity,
    buyUser: payload.buyuser ?? null,
    sellUser: payload.selluser ?? null,
    timestamp: ts,
  };
}

async function writeTradeAndCandles(
  trade: {
    symbol: string;
    price: string;
    quantity: string;
    buyUser: string | null;
    sellUser: string | null;
    timestamp: Date;
  },
  raw: string,
) {
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

    for (const minutes of candleTimeframes) {
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

function toDecimalString(value: string | number | undefined, field: string) {
  if (value === undefined || value === null) {
    throw new Error(`missing ${field}`);
  }
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) {
    throw new Error(`invalid ${field}: ${value}`);
  }
  return String(value);
}

function bucketStart(ts: Date, minutes: number) {
  const sizeMs = minutes * 60 * 1000;
  const bucketMs = Math.floor(ts.getTime() / sizeMs) * sizeMs;
  return new Date(bucketMs).toISOString();
}
