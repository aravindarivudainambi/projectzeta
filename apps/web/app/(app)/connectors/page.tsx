"use client";

import { useEffect, useState } from "react";
import { useAuth } from "@/contexts/AuthContext";
import {
  listConnectors,
  getNotionOAuthUrl,
  type ConnectorInfo,
} from "@api-client";

export default function ConnectorsPage() {
  const { token } = useAuth();
  const [connectors, setConnectors] = useState<ConnectorInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    if (!token) return;
    listConnectors(token)
      .then(setConnectors)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [token]);

  const handleConnectNotion = async () => {
    if (!token) return;
    try {
      const { redirect_url } = await getNotionOAuthUrl(token);
      window.location.href = redirect_url;
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to start Notion OAuth");
    }
  };

  if (loading) {
    return (
      <main className="p-8 text-zinc-400">Loading connectors...</main>
    );
  }

  return (
    <main className="p-8 max-w-3xl mx-auto">
      <h1 className="text-2xl font-semibold mb-2">Connectors</h1>
      <p className="text-sm text-zinc-500 mb-8">
        Connect external services to use their tools in your agent workflows.
      </p>
      {error && (
        <p className="text-sm text-red-600 mb-4">{error}</p>
      )}
      <div className="grid gap-4">
        {connectors.map((c) => (
          <div
            key={c.name}
            className="flex items-center justify-between rounded-xl border bg-white p-5 shadow-sm"
          >
            <div>
              <h2 className="font-medium text-zinc-900">{c.display_name}</h2>
              <p className="text-xs text-zinc-500 mt-1">
                {c.connected ? "Connected" : "Not connected"}
              </p>
            </div>
            <div>
              {c.connected ? (
                <span className="rounded-full bg-emerald-50 border border-emerald-200 px-3 py-1 text-xs font-semibold text-emerald-700">
                  Connected
                </span>
              ) : c.name === "notion" ? (
                <button
                  onClick={handleConnectNotion}
                  className="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-700"
                >
                  Connect Notion
                </button>
              ) : (
                <span className="text-xs text-zinc-400">Coming soon</span>
              )}
            </div>
          </div>
        ))}
      </div>
    </main>
  );
}
