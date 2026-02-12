/**
 * Environment and worker configuration.
 */

export const config = {
  redisUrl: process.env.REDIS_URL ?? "redis://127.0.0.1/",
  tradesKey: process.env.TRADES_KEY ?? "trades:btc",
  timescaleUrl:
    process.env.TIMESCALE_URL ??
    process.env.DATABASE_URL ??
    "postgres://postgres:postgres@127.0.0.1:5432/postgres",
  defaultSymbol:
    process.env.DEFAULT_SYMBOL ??
    (process.env.TRADES_KEY ?? "trades:btc").split(":").pop() ??
    "btc",
  candleTimeframes: [1, 5, 10] as const,
} as const;

export type CandleTimeframeMinutes = (typeof config.candleTimeframes)[number];
