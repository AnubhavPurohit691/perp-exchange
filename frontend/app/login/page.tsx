"use client"
import React, { useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card, CardHeader, CardContent, CardDescription, CardFooter, CardTitle } from '@/components/ui/card';
import { useAuthStore } from '@/lib/store';
import axios from 'axios';

export default function LoginPage() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [loading, setLoading] = useState(false);
    const router = useRouter();
    const { setAuth } = useAuthStore();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);
        try {
            const res = await axios.post('http://localhost:3000/login', { email, password });
            setAuth(res.data.token, res.data.user);
            router.push('/exchange');
        } catch (e: unknown) {
            if (axios.isAxiosError(e)) {
                alert(e.response?.data || "Login failed");
            } else {
                alert("Login failed");
            }
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="min-h-screen bg-black flex items-center justify-center p-4">
            <Card className="w-full max-w-sm bg-zinc-900 border-zinc-800 text-white">
                <CardHeader>
                    <CardTitle className="text-2xl">Login</CardTitle>
                    <CardDescription className="text-zinc-400">
                        Enter your email below to login to your account.
                    </CardDescription>
                </CardHeader>
                <form onSubmit={handleSubmit}>
                    <CardContent className="space-y-4">
                        <div className="space-y-2">
                            <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">Email</label>
                            <Input
                                type="email"
                                placeholder="m@example.com"
                                className="bg-zinc-950 border-zinc-800 focus:ring-zinc-700"
                                value={email}
                                onChange={(e) => setEmail(e.target.value)}
                            />
                        </div>
                        <div className="space-y-2">
                            <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">Password</label>
                            <Input
                                type="password"
                                className="bg-zinc-950 border-zinc-800 focus:ring-zinc-700"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                            />
                        </div>
                    </CardContent>
                    <CardFooter className="flex flex-col gap-4">
                        <Button className="w-full bg-white text-black hover:bg-zinc-200" disabled={loading}>
                            {loading ? "Signing in..." : "Sign in"}
                        </Button>
                        <p className="text-center text-sm text-zinc-400">
                            Don&apos;t have an account? <Link href="/signup" className="text-white hover:underline">Sign up</Link>
                        </p>
                    </CardFooter>
                </form>
            </Card>
        </div>
    );
}
