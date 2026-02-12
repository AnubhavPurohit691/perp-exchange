/**
 * Trade payload and normalized trade types.
 */

export type RawTrade = {
  symbol?: string;
  price?: string | number;
  quantity?: string | number;
  buyuser?: string;
  selluser?: string;
  timestamp?: string;
};

export type NormalizedTrade = {
  symbol: string;
  price: string;
  quantity: string;
  buyUser: string | null;
  sellUser: string | null;
  timestamp: Date;
};
