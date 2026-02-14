"use client"
import React, { useEffect } from 'react';
import { Navbar } from '@/components/layout/Navbar';
import { Chart } from '@/components/exchange/Chart';
import { OrderBook } from '@/components/exchange/OrderBook';
import { TradeForm } from '@/components/exchange/TradeForm';
import { PositionsTable } from '@/components/exchange/Positions'; // Fixed import name
import { useRouter } from 'next/navigation';
import { useAuthStore } from '@/lib/store';
import { fetchMe } from '@/lib/api';

export default function ExchangePage() {
    const router = useRouter();
    const { token, setAuth, logout } = useAuthStore();

    useEffect(() => {
        if (!token) {
            router.push('/login');
            return;
        }

        fetchMe()
            .then(user => {
                setAuth(token, user);
            })
            .catch(() => {
                logout();
                router.push('/login');
            });
    }, [token, router, setAuth, logout]);

    return (
        <div className="min-h-screen bg-black text-white selection:bg-indigo-500/30 selection:text-indigo-200 font-sans">
            <Navbar />

            <main className="pt-20 px-4 pb-4 h-[calc(100vh-80px)] container mx-auto max-w-[1920px]">
                <div className="grid grid-cols-1 lg:grid-cols-4 gap-4 h-full">

                    {/* Left Column: OrderBook */}
                    <div className="lg:col-span-1 flex flex-col h-full overflow-hidden">
                        <div className="flex-1 min-h-[300px] bg-zinc-900/20 rounded-lg border border-white/5 overflow-hidden">
                            <OrderBook />
                        </div>
                    </div>

                    {/* Center Column: Chart & Positions */}
                    <div className="lg:col-span-2 flex flex-col gap-4 h-full">
                        {/* 1. Chart Section */}
                        <div className="flex-1 min-h-[400px] border border-white/5 rounded-lg bg-zinc-900/20 backdrop-blur-sm overflow-hidden relative group">
                            <div className="absolute inset-0 bg-gradient-to-t from-black/50 to-transparent pointer-events-none z-0" />
                            <Chart />
                        </div>

                        {/* 2. Positions Section */}
                        <div className="h-[300px] overflow-auto border border-white/5 rounded-lg bg-zinc-900/20 backdrop-blur-sm">
                            <PositionsTable />
                        </div>
                    </div>

                    {/* Right Column: TradeForm */}
                    <div className="lg:col-span-1 flex flex-col h-full">
                        <div className="flex-none">
                            <TradeForm />
                        </div>
                    </div>

                </div>
            </main>
        </div>
    );
}
