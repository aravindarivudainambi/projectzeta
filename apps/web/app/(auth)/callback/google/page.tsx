"use client";

import { useEffect, useState, Suspense } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { useAuth } from "@/contexts/AuthContext";
import { exchangeGoogleCode } from "@api-client";

function GoogleCallbackInner() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const { token, isLoading } = useAuth();
  const [status, setStatus] = useState<"processing" | "success" | "error">(
    "processing",
  );
  const [errorMsg, setErrorMsg] = useState("");

  useEffect(() => {
    if (isLoading) return;

    const code = searchParams.get("code");
    if (!code) {
      setStatus("error");
      setErrorMsg("No authorization code received from Google.");
      return;
    }
    if (!token) {
      setStatus("error");
      setErrorMsg("Not authenticated. Please log in first.");
      return;
    }

    exchangeGoogleCode(token, code)
      .then(() => {
        setStatus("success");
        setTimeout(() => router.push("/connectors"), 1500);
      })
      .catch((e) => {
        setStatus("error");
        setErrorMsg(e instanceof Error ? e.message : "Code exchange failed.");
      });
  }, [searchParams, token, isLoading, router]);

  return (
    <main className="min-h-screen flex items-center justify-center bg-zinc-50">
      <div className="text-center">
        {(status === "processing" || isLoading) && (
          <p className="text-zinc-500">Connecting Google Workspace...</p>
        )}
        {status === "success" && (
          <p className="text-emerald-600 font-semibold">
            Google Workspace connected! Redirecting...
          </p>
        )}
        {status === "error" && !isLoading && (
          <div>
            <p className="text-red-600 mb-2">{errorMsg}</p>
            <a href="/connectors" className="text-sm text-indigo-600 underline">
              Back to connectors
            </a>
          </div>
        )}
      </div>
    </main>
  );
}

export default function GoogleCallbackPage() {
  return (
    <Suspense
      fallback={
        <main className="min-h-screen flex items-center justify-center bg-zinc-50">
          <p className="text-zinc-500">Loading...</p>
        </main>
      }
    >
      <GoogleCallbackInner />
    </Suspense>
  );
}
