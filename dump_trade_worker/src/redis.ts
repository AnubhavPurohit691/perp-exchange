import { createClient } from "redis";
import { config } from "./config.ts";

const redisClient = createClient({ url: config.redisUrl });
redisClient.on("error", (err: unknown) => {
  console.error("redis error", err);
});

export async function connectRedis(): Promise<void> {
  await redisClient.connect();
}

export function getRedisClient() {
  return redisClient;
}
