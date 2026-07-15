import { invoke } from "@tauri-apps/api/core";

export type SnapshotStatus = "ready" | "signed_out" | "unsupported";

export interface UsageSnapshot {
  status: SnapshotStatus;
  message: string | null;
  source: string;
  updatedAt: string;
  codexVersion: string | null;
  account: AccountView | null;
  rateLimits: RateLimitsView | null;
  tokenUsage: TokenUsageView | null;
}

export interface AccountView {
  authType: "chatgpt" | "apiKey" | "amazonBedrock" | string;
  planType: string | null;
}

export interface RateLimitWindowView {
  usedPercent: number;
  remainingPercent: number;
  windowDurationMins: number | null;
  resetsAt: number | null;
}

export interface CreditsView {
  hasCredits: boolean;
  unlimited: boolean;
  balance: string | null;
}

export interface RateLimitBucketView {
  limitId: string | null;
  limitName: string | null;
  primary: RateLimitWindowView | null;
  secondary: RateLimitWindowView | null;
  credits: CreditsView | null;
  planType: string | null;
  rateLimitReachedType: string | null;
}

export interface RateLimitsView {
  selected: RateLimitBucketView;
  buckets: RateLimitBucketView[];
  resetCreditsAvailable: string | null;
}

export interface TokenUsageView {
  todayTokens: string | null;
  last7DaysTokens: string | null;
  lifetimeTokens: string | null;
  peakDailyTokens: string | null;
  longestRunningTurnSec: string | null;
  currentStreakDays: string | null;
  longestStreakDays: string | null;
  dailyBuckets: Array<{ startDate: string; tokens: string }> | null;
}

export function isTauriRuntime(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export async function fetchUsageSnapshot(): Promise<UsageSnapshot> {
  return invoke<UsageSnapshot>("hexu_usage_snapshot");
}

const planLabels: Record<string, string> = {
  free: "FREE",
  go: "GO",
  plus: "PLUS",
  pro: "PRO",
  prolite: "PRO LITE",
  team: "TEAM",
  self_serve_business_usage_based: "BUSINESS",
  business: "BUSINESS",
  enterprise_cbp_usage_based: "ENTERPRISE",
  enterprise: "ENTERPRISE",
  edu: "EDU",
  unknown: "未知套餐",
};

export function formatPlanLabel(planType: string | null | undefined): string {
  if (!planType) return "账户";
  return planLabels[planType] ?? planType.replaceAll("_", " ").toUpperCase();
}

export function formatResetTime(timestamp: number | null): string {
  if (!timestamp) return "重置时间未知";
  const reset = new Date(timestamp * 1000);
  const differenceMs = reset.getTime() - Date.now();
  if (differenceMs <= 0) return "正在重置";
  const totalMinutes = Math.max(1, Math.round(differenceMs / 60_000));
  if (totalMinutes < 24 * 60) {
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;
    if (hours === 0) return `${minutes}分钟后重置`;
    if (minutes === 0) return `${hours}小时后重置`;
    return `${hours}小时${minutes}分后重置`;
  }
  return `${new Intl.DateTimeFormat("zh-CN", {
    weekday: "short",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(reset)} 重置`;
}

export function formatUpdatedAt(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "刚刚更新";
  const seconds = Math.max(0, Math.round((Date.now() - date.getTime()) / 1000));
  if (seconds < 45) return "刚刚更新";
  if (seconds < 120) return "1分钟前更新";
  return `${Math.floor(seconds / 60)}分钟前更新`;
}

export function formatTokenMetric(value: string | null | undefined): {
  value: string;
  unit: string;
} {
  if (!value) return { value: "—", unit: "" };
  try {
    const tokens = BigInt(value);
    if (tokens >= 10_000n) {
      const tenths = (tokens + 500n) / 1_000n;
      const integer = tenths / 10n;
      const decimal = tenths % 10n;
      return {
        value: decimal === 0n ? integer.toString() : `${integer}.${decimal}`,
        unit: "万",
      };
    }
    return { value: tokens.toLocaleString("zh-CN"), unit: "Token" };
  } catch {
    return { value: "—", unit: "" };
  }
}
