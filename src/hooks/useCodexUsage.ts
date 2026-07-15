import { useCallback, useEffect, useState } from "react";
import { fetchUsageSnapshot, type UsageSnapshot } from "../lib/usage";

export function useCodexUsage(enabled: boolean) {
  const [snapshot, setSnapshot] = useState<UsageSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(enabled);

  const refresh = useCallback(async () => {
    if (!enabled) return;
    setLoading(true);
    try {
      const next = await fetchUsageSnapshot();
      setSnapshot(next);
      setError(null);
    } catch (reason) {
      const message = typeof reason === "string" ? reason : "暂时无法连接 Codex app-server。";
      setError(message);
    } finally {
      setLoading(false);
    }
  }, [enabled]);

  useEffect(() => {
    if (!enabled) return;
    void refresh();
    const timer = window.setInterval(() => void refresh(), 60_000);
    return () => window.clearInterval(timer);
  }, [enabled, refresh]);

  return { snapshot, error, loading, refresh };
}
