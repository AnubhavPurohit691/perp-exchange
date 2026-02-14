import axios from 'axios';
import { useAuthStore } from './store';

const API_URL = 'http://localhost:3000';

const api = axios.create({
    baseURL: API_URL,
});

api.interceptors.request.use((config) => {
    const token = useAuthStore.getState().token;
    if (token) {
        config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
});

export interface Candle {
    bucket_start: string;
    open: string;
    high: string;
    low: string;
    close: string;
    volume: string;
}

export interface OrderbookSnapshot {
    bids: { price: string; quantity: string }[];
    asks: { price: string; quantity: string }[];
}

export interface Position {
    positionid: string;
    symbol: string;
    side: 'Buy' | 'Sell';
    quantity: string;
    entry_price: string;
    liquidation_price: string;
    margin: string;
    leverage: string;
    unrealized_pnl: string;
}

export const fetchCandles = async (symbol: string = 'btc', timeframe: string = '1m') => {
    try {
        const response = await api.get<Candle[]>('/candles', {
            params: { symbol, timeframe, limit: 1000 },
        });
        if (Array.isArray(response.data)) {
            return response.data;
        }
        return [];
    } catch (e) {
        console.error("Error fetching candles", e);
        return [];
    }
};

export const fetchOrderbook = async () => {
    try {
        const response = await api.get<OrderbookSnapshot>('/orderbook');
        const data = response.data;
        if (data && Array.isArray(data.bids) && Array.isArray(data.asks)) {
            return data;
        }
        return null;
    } catch (e) {
        console.error("Error fetching orderbook", e);
        return null; // Return null so SWR data is null, component handles null checks
    }
};

export const fetchPositions = async () => {
    try {
        const response = await api.get<Position[]>('/openpositions');
        if (Array.isArray(response.data)) {
            return response.data;
        }
        return [];
    } catch (e) {
        console.error("Error fetching positions", e);
        return [];
    }
};



export const fetchMe = async () => {
    const response = await api.get('/me');
    return response.data;
}

export const placeOrder = async (price: number, quantity: number, side: 'buy' | 'sell', leverage: number) => {
    const response = await api.post('/orderbook', {
        price,
        quantity,
        ordertype: side,
        leverage
    });
    return response.data;
};

export const closePosition = async (positionid: string) => {
    const response = await api.post('/closeposition', { positionid });
    return response.data;
}
