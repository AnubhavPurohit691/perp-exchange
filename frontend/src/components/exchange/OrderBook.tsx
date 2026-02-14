"use client"
import React from 'react';
import useSWR from 'swr';
import { fetchOrderbook, OrderbookSnapshot } from '@/lib/api';

export function OrderBook() {
    const { data: snapshot, isLoading } = useSWR<OrderbookSnapshot | null>('orderbook', fetchOrderbook, {
        refreshInterval: 1000,
    });

    if (isLoading && !snapshot) {
        return (
            <div className="w-full h-full flex flex-col bg-zinc-900/50 backdrop-blur-sm border border-white/5 rounded-lg overflow-hidden font-mono text-xs items-center justify-center text-zinc-500">
                Loading Orderbook...
            </div>
        );
    }

    return (
        <div className="w-full h-full flex flex-col bg-zinc-900/50 backdrop-blur-sm border border-white/5 rounded-lg overflow-hidden font-mono text-xs">
            <div className="px-4 py-2 bg-zinc-900 border-b border-white/5 font-semibold text-zinc-400 flex justify-between">
                <span>Order Book</span>
                <span>Spread: {snapshot && snapshot.bids.length > 0 && snapshot.asks.length > 0
                    ? (Number(snapshot.asks[0].price) - Number(snapshot.bids[0].price)).toFixed(2)
                    : '--'}</span>
            </div>

            <div className="flex-1 overflow-y-auto">
                <div className="grid grid-cols-3 px-4 py-1 text-zinc-500 border-b border-white/5">
                    <span className="text-left">Price (USD)</span>
                    <span className="text-right">Size (BTC)</span>
                    <span className="text-right">Total</span>
                </div>

                {/* Asks (Sell Orders) - Red */}
                <div className="flex flex-col-reverse">
                    {snapshot?.asks.slice(0, 15).map((ask, i) => (
                        <div key={i} className="grid grid-cols-3 px-4 py-0.5 hover:bg-white/5 cursor-pointer group">
                            <span className="text-red-400 group-hover:text-red-300">{Number(ask.price).toFixed(2)}</span>
                            <span className="text-right text-zinc-300">{Number(ask.quantity).toFixed(4)}</span>
                            <span className="text-right text-zinc-500">{(Number(ask.price) * Number(ask.quantity)).toFixed(2)}</span>
                        </div>
                    ))}
                </div>

                {/* Current Price Indicator */}
                <div className="py-2 text-center text-lg font-bold text-white border-y border-white/5 my-1 bg-zinc-800/50">
                    {snapshot?.bids[0]?.price ? Number(snapshot.bids[0].price).toFixed(2) : '--'}
                    <span className="text-xs text-zinc-500 ml-2 font-normal">USD</span>
                </div>

                {/* Bids (Buy Orders) - Green */}
                <div>
                    {snapshot?.bids.slice(0, 15).map((bid, i) => (
                        <div key={i} className="grid grid-cols-3 px-4 py-0.5 hover:bg-white/5 cursor-pointer group">
                            <span className="text-green-400 group-hover:text-green-300">{Number(bid.price).toFixed(2)}</span>
                            <span className="text-right text-zinc-300">{Number(bid.quantity).toFixed(4)}</span>
                            <span className="text-right text-zinc-500">{(Number(bid.price) * Number(bid.quantity)).toFixed(2)}</span>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}
