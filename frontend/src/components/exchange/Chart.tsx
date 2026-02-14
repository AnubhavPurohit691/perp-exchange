import { createChart, ColorType, IChartApi, ISeriesApi, CandlestickSeries, UTCTimestamp } from 'lightweight-charts';
import React, { useEffect, useRef, useMemo } from 'react';
import useSWR from 'swr';
import { fetchCandles, Candle } from '@/lib/api';

export function Chart() {
    const chartContainerRef = useRef<HTMLDivElement>(null);
    const chartRef = useRef<IChartApi | null>(null);
    const seriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);

    const { data: candles, error, isLoading } = useSWR<Candle[]>('candles', () => fetchCandles('btc', '1m'), {
        refreshInterval: 1000,
        dedupingInterval: 1000,
    });

    // Formatting data using useMemo to avoid recalculation on every render
    const formattedData = useMemo(() => {
        if (!candles) return [];

        const data = candles.map(c => ({
            time: (new Date(c.bucket_start).getTime() / 1000) as UTCTimestamp,
            open: parseFloat(c.open),
            high: parseFloat(c.high),
            low: parseFloat(c.low),
            close: parseFloat(c.close),
        }));

        // Sort by time
        data.sort((a, b) => (a.time as number) - (b.time as number));

        // Remove duplicates
        const uniqueData = [];
        const times = new Set();
        for (const d of data) {
            if (!times.has(d.time)) {
                uniqueData.push(d);
                times.add(d.time);
            }
        }
        return uniqueData;
    }, [candles]);

    // Initialize Chart
    useEffect(() => {
        if (!chartContainerRef.current) return;

        const chart = createChart(chartContainerRef.current, {
            layout: {
                background: { type: ColorType.Solid, color: 'transparent' },
                textColor: '#d1d5db',
            },
            grid: {
                vertLines: { color: 'rgba(42, 46, 57, 0.1)' },
                horzLines: { color: 'rgba(42, 46, 57, 0.1)' },
            },
            width: chartContainerRef.current.clientWidth,
            height: 400,
            timeScale: {
                timeVisible: true,
                secondsVisible: false,
                borderColor: 'rgba(42, 46, 57, 0.1)',
            },
            rightPriceScale: {
                borderColor: 'rgba(42, 46, 57, 0.1)',
            },
        });
        chartRef.current = chart;

        const newSeries = chart.addSeries(CandlestickSeries, {
            upColor: '#26a69a',
            downColor: '#ef5350',
            borderVisible: false,
            wickUpColor: '#26a69a',
            wickDownColor: '#ef5350',
        });
        seriesRef.current = newSeries;

        // ResizeObserver for more robust resizing
        const resizeObserver = new ResizeObserver((entries) => {
            if (!chartRef.current) return;
            for (const entry of entries) {
                const { width, height } = entry.contentRect;
                chartRef.current.applyOptions({ width, height });
            }
        });
        resizeObserver.observe(chartContainerRef.current);

        return () => {
            resizeObserver.disconnect();
            chart.remove();
        };
    }, []);

    // Update Data
    useEffect(() => {
        if (seriesRef.current && formattedData.length > 0) {
            seriesRef.current.setData(formattedData);
        }
    }, [formattedData]);

    return (
        <div className="relative w-full h-[400px] bg-black border border-zinc-800 rounded-lg overflow-hidden flex flex-col">
            <div className="absolute top-4 left-4 z-10 flex items-center gap-2 pointer-events-none">
                <span className="text-white font-bold bg-zinc-900/80 backdrop-blur px-2 py-1 rounded border border-white/5">BTC/USD</span>
                <span className="text-zinc-400 text-xs bg-zinc-900/50 px-2 py-1 rounded">1M</span>
            </div>

            {isLoading && !candles && (
                <div className="absolute inset-0 flex items-center justify-center text-zinc-500 bg-black/50 z-20 backdrop-blur-sm">
                    <div className="animate-pulse">Loading Chart...</div>
                </div>
            )}

            {error && (
                <div className="absolute inset-0 flex items-center justify-center text-red-500 bg-black/50 z-20 backdrop-blur-sm">
                    Failed to load chart data
                </div>
            )}

            <div ref={chartContainerRef} className="w-full h-full" />
        </div>
    );
}
