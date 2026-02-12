import { config } from "./config.ts";
import { initSchema, writeTradeAndCandles } from "./db/index.ts";
import { connectRedis, getRedisClient } from "./redis.ts";
import { normalizeTrade } from "./utils/trade.ts";

await initSchema();
await connectRedis();

const redisClient = getRedisClient();
const { tradesKey, redisUrl, timescaleUrl, defaultSymbol } = config;

console.log(`listening for trades on ${tradesKey} via ${redisUrl}`);
console.log(`writing trades + candles to ${timescaleUrl}`);

while (true) {
  const res = await redisClient.blPop(tradesKey, 0);
  if (!res) continue;

  const raw = res.element;
  try {
    const trade = normalizeTrade(raw, defaultSymbol);
    await writeTradeAndCandles(trade, raw);
  } catch (err) {
    console.error("failed to process trade", { raw, err });
  }
}
