import type { NormalizedTrade, RawTrade } from "../types.ts";

export function toDecimalString(
  value: string | number | undefined,
  field: string,
): string {
  if (value === undefined || value === null) {
    throw new Error(`missing ${field}`);
  }
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) {
    throw new Error(`invalid ${field}: ${value}`);
  }
  return String(value);
}

export function bucketStart(ts: Date, minutes: number): string {
  const sizeMs = minutes * 60 * 1000;
  const bucketMs = Math.floor(ts.getTime() / sizeMs) * sizeMs;
  return new Date(bucketMs).toISOString();
}

export function normalizeTrade(
  raw: string,
  defaultSymbol: string,
): NormalizedTrade {
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
