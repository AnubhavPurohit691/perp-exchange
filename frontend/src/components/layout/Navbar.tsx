"use client"
import Link from 'next/link';
import { Button } from '@/components/ui/button';
import { Hexagon, User, LogOut } from 'lucide-react';
import { useAuthStore } from '@/lib/store';
import { useRouter } from 'next/navigation';

export function Navbar() {
    const { user, logout } = useAuthStore();
    const router = useRouter();

    const handleLogout = () => {
        logout();
        router.push('/');
    };

    return (
        <nav className="fixed top-0 w-full z-50 border-b border-white/10 bg-black/50 backdrop-blur-md">
            <div className="container mx-auto px-4 h-16 flex items-center justify-between">
                <Link href="/" className="flex items-center gap-2">
                    <Hexagon className="w-8 h-8 text-white fill-white" />
                    <span className="text-xl font-bold tracking-tighter text-white">PerpX</span>
                </Link>

                <div className="hidden md:flex items-center gap-6">
                    <Link href="/exchange" className="text-sm font-medium text-zinc-400 hover:text-white transition-colors">
                        Exchange
                    </Link>
                    <Link href="/learn" className="text-sm font-medium text-zinc-400 hover:text-white transition-colors">
                        Learn
                    </Link>
                    <Link href="/developers" className="text-sm font-medium text-zinc-400 hover:text-white transition-colors">
                        API
                    </Link>
                </div>

                <div className="flex items-center gap-4">
                    {user ? (
                        <div className="flex items-center gap-4">
                            <span className="text-sm text-zinc-300 font-medium flex items-center gap-2">
                                <User className="w-4 h-4" />
                                {user.name}
                            </span>
                            <Button
                                variant="ghost"
                                size="sm"
                                onClick={handleLogout}
                                className="text-zinc-400 hover:text-red-400 hover:bg-red-400/10"
                            >
                                <LogOut className="w-4 h-4 mr-2" />
                                Logout
                            </Button>
                        </div>
                    ) : (
                        <>
                            <Link href="/login">
                                <Button variant="ghost" className="text-zinc-400 hover:text-white">
                                    Log in
                                </Button>
                            </Link>
                            <Link href="/signup">
                                <Button className="bg-white text-black hover:bg-zinc-200">
                                    Sign up
                                </Button>
                            </Link>
                        </>
                    )}
                </div>
            </div>
        </nav>
    );
}
