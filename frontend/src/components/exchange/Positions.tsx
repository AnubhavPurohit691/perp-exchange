"use client"
import React, { useState } from 'react';
import useSWR, { mutate } from 'swr';
import { fetchPositions, closePosition, Position } from '@/lib/api';
import { Button } from '@/components/ui/button';

export function PositionsTable() {
    const { data: positions = [], isLoading } = useSWR<Position[]>('positions', fetchPositions, {
        refreshInterval: 2000,
        fallbackData: [], // Default to empty array
    });
    const [closing, setClosing] = useState<string | null>(null);

    const handleClose = async (id: string) => {
        setClosing(id);
        try {
            await closePosition(id);
            await mutate('positions');
        } catch (e) {
            console.error(e);
            alert("Failed to close position");
        } finally {
            setClosing(null);
        }
    }

    return (
        <div className="w-full bg-zinc-900/50 border border-white/5 rounded-lg overflow-hidden backdrop-blur-md">
            <div className="px-4 py-3 border-b border-white/5 font-semibold text-zinc-400">
                Open Positions
            </div>
            <table className="w-full text-sm text-left text-zinc-400">
                <thead className="text-xs text-zinc-500 bg-zinc-950/50 uppercase">
                    <tr>
                        <th className="px-4 py-3">Symbol</th>
                        <th className="px-4 py-3">Side</th>
                        <th className="px-4 py-3 text-right">Size</th>
                        <th className="px-4 py-3 text-right">Entry Price</th>
                        <th className="px-4 py-3 text-right">Liq. Price</th>
                        <th className="px-4 py-3 text-right">Leverage</th>
                        <th className="px-4 py-3 text-right">Action</th>
                    </tr>
                </thead>
                <tbody>
                    {!positions || positions.length === 0 ? (
                        <tr>
                            <td colSpan={7} className="px-4 py-8 text-center text-zinc-600">
                                {isLoading ? "Loading..." : "No open positions"}
                            </td>
                        </tr>
                    ) : (
                        positions.map((p) => (
                            <tr key={p.positionid} className="border-b border-white/5 hover:bg-white/5 transition-colors">
                                <td className="px-4 py-3 font-medium text-white">{p.symbol.toUpperCase()}</td>
                                <td className={`px-4 py-3 ${p.side === 'Buy' ? 'text-green-400' : 'text-red-400'}`}>
                                    {p.side.toUpperCase()}
                                </td>
                                <td className="px-4 py-3 text-right">{Number(p.quantity).toFixed(4)}</td>
                                <td className="px-4 py-3 text-right text-white">{Number(p.entry_price).toFixed(2)}</td>
                                <td className="px-4 py-3 text-right text-orange-400">{Number(p.liquidation_price).toFixed(2)}</td>
                                <td className="px-4 py-3 text-right">{p.leverage}x</td>
                                <td className="px-4 py-3 text-right">
                                    <Button
                                        variant="destructive"
                                        size="sm"
                                        className="h-7 text-xs bg-red-500/20 hover:bg-red-500 text-red-400 hover:text-white border-none"
                                        onClick={() => handleClose(p.positionid)}
                                        disabled={closing === p.positionid}
                                    >
                                        {closing === p.positionid ? 'Closing...' : 'Close'}
                                    </Button>
                                </td>
                            </tr>
                        ))
                    )}
                </tbody>
            </table>
        </div>
    );
}
