"use client";

import { useEffect, type ReactNode } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuth } from "@/contexts/AuthContext";

export default function AppLayout({ children }: { children: ReactNode }) {
  const { token, user, isLoading, logout } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!isLoading && !token) {
      router.replace("/login");
    }
  }, [isLoading, token, router]);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center text-zinc-400">
        Loading...
      </div>
    );
  }

  if (!token) return null;

  return (
    <div className="min-h-screen">
      <nav className="h-12 border-b bg-white flex items-center justify-between px-6 text-sm">
        <div className="flex gap-6">
          <Link
            href="/dashboard"
            className="font-medium text-zinc-700 hover:text-zinc-900"
          >
            Dashboard
          </Link>
          <Link
            href="/connectors"
            className="font-medium text-zinc-700 hover:text-zinc-900"
          >
            Connectors
          </Link>
          <Link
            href="/agents"
            className="font-medium text-zinc-700 hover:text-zinc-900"
          >
            Agents
          </Link>
          <Link
            href="/marketplace"
            className="font-medium text-zinc-700 hover:text-zinc-900"
          >
            Marketplace
          </Link>
        </div>
        <div className="flex items-center gap-4">
          <span className="text-xs text-zinc-400">
            {user?.user_id?.slice(0, 8)}
          </span>
          <button
            onClick={() => {
              logout();
              router.push("/login");
            }}
            className="text-xs text-zinc-500 hover:text-zinc-900"
          >
            Sign out
          </button>
        </div>
      </nav>
      <section>{children}</section>
    </div>
  );
}
