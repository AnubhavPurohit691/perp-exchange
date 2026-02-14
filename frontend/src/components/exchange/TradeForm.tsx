"use client"
import React, { useState } from 'react';
import { useAuthStore } from '@/lib/store';
import { placeOrder } from '@/lib/api';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card } from '@/components/ui/card';
import { AxiosError } from 'axios';
// import { toast } from 'sonner';

export function TradeForm() {
    const { user, token } = useAuthStore();
    const [side, setSide] = useState<'buy' | 'sell'>('buy');
    const [price, setPrice] = useState('50000');
    const [quantity, setQuantity] = useState('0.1');
    const [leverage, setLeverage] = useState(1);
    const [placing, setPlacing] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!token) {
            alert("Please login first");
            return;
        }
        setPlacing(true);
        try {
            await placeOrder(Number(price), Number(quantity), side, leverage);
            alert("Order Placed Successfully");
        } catch (err) {
            let message = "Unknown error";
            if (err instanceof AxiosError) {
                // Handle Axios error structure
                if (typeof err.response?.data === 'string') {
                    message = err.response.data;
                } else if (err.response?.data && typeof err.response.data === 'object') {
                    message = (err.response.data as { message?: string }).message || JSON.stringify(err.response.data);
                } else {
                    message = err.message;
                }
            } else if (err instanceof Error) {
                message = err.message;
            }
            alert("Order Failed: " + message);
        } finally {
            setPlacing(false);
        }
    };

    if (!user) {
        return (
            <div className="h-full flex items-center justify-center p-6 border border-white/5 rounded-lg bg-zinc-900/20 backdrop-blur-md">
                <div className="text-center">
                    <p className="text-zinc-400 mb-4">Connect wallet or login to trade</p>
                    <Button variant="default" className="w-full">Log In</Button>
                </div>
            </div>
        );
    }

    return (
        <Card className="h-full bg-zinc-900/50 border-white/5 backdrop-blur-md text-white">
            <div className="flex border-b border-white/5">
                <button
                    className={`flex-1 py-3 text-sm font-medium transition-colors ${side === 'buy' ? 'text-green-400 border-b-2 border-green-400 bg-green-400/5' : 'text-zinc-400 hover:text-white'}`}
                    onClick={() => setSide('buy')}
                >
                    Buy / Long
                </button>
                <button
                    className={`flex-1 py-3 text-sm font-medium transition-colors ${side === 'sell' ? 'text-red-400 border-b-2 border-red-400 bg-red-400/5' : 'text-zinc-400 hover:text-white'}`}
                    onClick={() => setSide('sell')}
                >
                    Sell / Short
                </button>
            </div>

            <form onSubmit={handleSubmit} className="p-4 space-y-4">
                <div className="space-y-2">
                    <label className="text-xs text-zinc-500 uppercase">Order Type</label>
                    <div className="flex gap-2">
                        <Button type="button" variant="secondary" size="sm" className="flex-1 bg-zinc-800 text-white hover:bg-zinc-700">Limit</Button>
                        <Button type="button" variant="ghost" size="sm" className="flex-1 text-zinc-400 hover:text-white">Market</Button>
                    </div>
                </div>

                <div className="space-y-2">
                    <label className="text-xs text-zinc-500 uppercase">Price (USDT)</label>
                    <div className="relative">
                        <Input
                            type="number"
                            className="bg-zinc-950 border-zinc-800 focus:ring-zinc-700 text-right pr-12"
                            value={price}
                            onChange={(e) => setPrice(e.target.value)}
                        />
                        <span className="absolute right-3 top-2.5 text-zinc-500 text-sm">USDT</span>
                    </div>
                </div>

                <div className="space-y-2">
                    <label className="text-xs text-zinc-500 uppercase">Amount (BTC)</label>
                    <div className="relative">
                        <Input
                            type="number"
                            className="bg-zinc-950 border-zinc-800 focus:ring-zinc-700 text-right pr-12"
                            value={quantity}
                            onChange={(e) => setQuantity(e.target.value)}
                        />
                        <span className="absolute right-3 top-2.5 text-zinc-500 text-sm">BTC</span>
                    </div>
                </div>

                <div className="space-y-4 py-2">
                    <div className="flex justify-between items-center">
                        <label className="text-xs text-zinc-500 uppercase">Leverage</label>
                        <span className="text-xs font-mono text-zinc-300 bg-zinc-800 px-2 py-0.5 rounded">{leverage}x</span>
                    </div>
                    <input
                        type="range"
                        min="1"
                        max="100"
                        step="1"
                        value={leverage}
                        onChange={(e) => setLeverage(Number(e.target.value))}
                        className="w-full h-1 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-white"
                    />
                    <div className="flex justify-between text-xs text-zinc-600 font-mono">
                        <span>1x</span>
                        <span>25x</span>
                        <span>50x</span>
                        <span>75x</span>
                        <span>100x</span>
                    </div>
                </div>

                <div className="pt-4 border-t border-white/5 space-y-2 text-xs text-zinc-400">
                    <div className="flex justify-between">
                        <span>Margin Required</span>
                        <span className="text-white">{(Number(price) * Number(quantity) / leverage).toFixed(2)} USDT</span>
                    </div>
                    <div className="flex justify-between">
                        <span>Available Balance</span>
                        <span className="text-white">{Number(user.balance).toFixed(2)} USDT</span>
                    </div>
                </div>

                <Button
                    disabled={placing}
                    className={`w-full h-12 text-base font-bold ${side === 'buy' ? 'bg-green-500 hover:bg-green-600 text-black' : 'bg-red-500 hover:bg-red-600 text-white'}`}
                >
                    {placing ? 'Placing Order...' : (side === 'buy' ? 'Buy / Long BTC' : 'Sell / Short BTC')}
                </Button>
            </form>
        </Card>
    );
}
