"use client"
import Link from 'next/link';
import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import { ArrowRight, BarChart2, ShieldCheck, Zap } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Navbar } from '@/components/layout/Navbar';
import { useAuthStore } from '@/lib/store';
import { fetchMe } from '@/lib/api';

export default function LandingPage() {
  const router = useRouter();
  const { token, logout } = useAuthStore();

  useEffect(() => {
    if (token) {
      fetchMe()
        .then(() => router.push('/exchange'))
        .catch(() => logout());
    }
  }, [token, router, logout]);

  return (
    <div className="min-h-screen bg-black text-white selection:bg-white selection:text-black font-sans">
      <Navbar />

      {/* Background Gradients */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute top-[-20%] left-[20%] w-[500px] h-[500px] bg-blue-500/20 rounded-full blur-[120px]" />
        <div className="absolute bottom-[-20%] right-[10%] w-[500px] h-[500px] bg-purple-500/10 rounded-full blur-[120px]" />
      </div>

      {/* Hero Section */}
      <section className="relative pt-32 pb-20 md:pt-48 md:pb-32 px-4 container mx-auto text-center z-10">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease: "easeOut" }}
        >
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-white/10 bg-white/5 backdrop-blur-sm text-xs font-medium text-zinc-400 mb-8">
            <span className="flex h-2 w-2 rounded-full bg-green-500 animate-pulse"></span>
            v1.0 is now live on Mainnet
          </div>

          <h1 className="text-5xl md:text-7xl font-bold tracking-tight mb-6 bg-clip-text text-transparent bg-gradient-to-b from-white to-white/50">
            Professional Grade <br />
            <span className="text-white">Perpetual Exchange</span>
          </h1>

          <p className="text-lg md:text-xl text-zinc-400 max-w-2xl mx-auto mb-10 leading-relaxed">
            Trade with sub-millisecond latency, deep liquidity, and up to 100x leverage.
            Built for traders who demand precision and speed.
          </p>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link href="/exchange">
              <Button size="lg" className="h-12 px-8 text-base bg-white text-black hover:bg-zinc-200">
                Start Trading <ArrowRight className="ml-2 w-4 h-4" />
              </Button>
            </Link>
            <Link href="/learn">
              <Button variant="outline" size="lg" className="h-12 px-8 text-base border-white/10 text-white hover:bg-white/5">
                View Documentation
              </Button>
            </Link>
          </div>
        </motion.div>
      </section>

      {/* Features Grid */}
      <section className="py-20 bg-black/50 border-t border-white/5 relative z-10">
        <div className="container mx-auto px-4">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <FeatureCard
              icon={<Zap className="w-6 h-6 text-blue-400" />}
              title="Lowest Latency"
              description="Our custom matching engine processes orders in <1ms, ensuring you catch every market move."
            />
            <FeatureCard
              icon={<ShieldCheck className="w-6 h-6 text-green-400" />}
              title="Ironclad Security"
              description="Assets are secured with multi-sig cold storage and real-time risk monitoring systems."
            />
            <FeatureCard
              icon={<BarChart2 className="w-6 h-6 text-purple-400" />}
              title="Advanced Charting"
              description="Professional-grade tooling with TradingView integration for precise technical analysis."
            />
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-12 border-t border-white/5 text-center text-zinc-600 text-sm relative z-10">
        <p>© 2024 PerpX. All rights reserved.</p>
      </footer>
    </div>
  );
}

function FeatureCard({ icon, title, description }: { icon: React.ReactNode, title: string, description: string }) {
  return (
    <div className="p-8 rounded-2xl bg-zinc-900/50 border border-white/5 hover:border-white/10 transition-colors group">
      <div className="mb-4 p-3 bg-white/5 rounded-xl w-fit group-hover:bg-white/10 transition-colors">
        {icon}
      </div>
      <h3 className="text-xl font-semibold mb-2 text-white">{title}</h3>
      <p className="text-zinc-400 leading-relaxed">
        {description}
      </p>
    </div>
  )
}
