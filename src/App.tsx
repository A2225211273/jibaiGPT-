import { useEffect, useId, useMemo, useState, type ReactNode } from "react";
import { useCodexUsage } from "./hooks/useCodexUsage";
import {
  formatPlanLabel,
  formatResetTime,
  formatTokenMetric,
  formatUpdatedAt,
  isTauriRuntime,
  type UsageSnapshot,
} from "./lib/usage";

type Theme = "light" | "dark";
type View = "board" | "desktop" | "attached" | "conversation";

const Icon = ({ children, size = 18 }: { children: ReactNode; size?: number }) => (
  <svg
    aria-hidden="true"
    className="icon"
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    {children}
  </svg>
);

const icons = {
  clock: (
    <Icon>
      <circle cx="12" cy="12" r="8.5" />
      <path d="M12 7.5v5l3.2 1.8" />
    </Icon>
  ),
  calendar: (
    <Icon>
      <rect x="4" y="5.5" width="16" height="14" rx="3" />
      <path d="M8 3.5v4M16 3.5v4M4 9.5h16" />
    </Icon>
  ),
  tokens: (
    <Icon>
      <ellipse cx="12" cy="6" rx="7.5" ry="3" />
      <path d="M4.5 6v6c0 1.7 3.4 3 7.5 3s7.5-1.3 7.5-3V6M4.5 12v6c0 1.7 3.4 3 7.5 3s7.5-1.3 7.5-3v-6" />
    </Icon>
  ),
  chart: (
    <Icon>
      <path d="M5 19V12M10 19V8M15 19V5M20 19V10" />
    </Icon>
  ),
  bell: (
    <Icon>
      <path d="M6.5 16.5h11l-1.5-2.3V10a4 4 0 0 0-8 0v4.2l-1.5 2.3Z" />
      <path d="M10 19h4" />
    </Icon>
  ),
  settings: (
    <Icon size={20}>
      <circle cx="12" cy="12" r="3" />
      <path d="M19 12a7 7 0 0 0-.1-1.2l2-1.5-2-3.4-2.4 1A8 8 0 0 0 14.4 5L14 2.5h-4L9.6 5a8 8 0 0 0-2.1 1.9l-2.4-1-2 3.4 2 1.5A7 7 0 0 0 5 12c0 .4 0 .8.1 1.2l-2 1.5 2 3.4 2.4-1A8 8 0 0 0 9.6 19l.4 2.5h4l.4-2.5a8 8 0 0 0 2.1-1.9l2.4 1 2-3.4-2-1.5c.1-.4.1-.8.1-1.2Z" />
    </Icon>
  ),
  sun: (
    <Icon>
      <circle cx="12" cy="12" r="3.5" />
      <path d="M12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M18.7 5.3l-1.4 1.4M6.7 17.3l-1.4 1.4" />
    </Icon>
  ),
  moon: (
    <Icon>
      <path d="M20 15.3A8 8 0 0 1 8.7 4a8.3 8.3 0 1 0 11.3 11.3Z" />
    </Icon>
  ),
  check: (
    <Icon size={18}>
      <path d="m6.5 12 3.4 3.4L18 7.8" />
    </Icon>
  ),
};

function BrandMark({ compact = false }: { compact?: boolean }) {
  return (
    <svg
      aria-label="禾序序列波标识"
      className={compact ? "brand-mark is-compact" : "brand-mark"}
      viewBox="0 0 64 64"
      role="img"
    >
      <rect width="64" height="64" rx="19" fill="currentColor" opacity="0.1" />
      <path d="M12 22c8-9 15-9 23 0s15 9 22 0" />
      <path d="M9 32c10-10 18-10 27 0s16 10 22 0" />
      <path d="M12 42c8-9 15-9 23 0s15 9 22 0" />
    </svg>
  );
}

function SequenceWave({ level = 72 }: { level?: number }) {
  const width = Math.max(10, Math.min(100, level));
  const gradientId = `wave-${useId().replace(/:/g, "")}`;
  return (
    <div className="sequence-wave" aria-hidden="true">
      <svg viewBox="0 0 170 52" preserveAspectRatio="none">
        <defs>
          <linearGradient id={gradientId} x1="0" x2="1">
            <stop offset="0" stopColor="currentColor" stopOpacity="0.2" />
            <stop offset={`${width}%`} stopColor="currentColor" stopOpacity="1" />
            <stop offset={`${width}%`} stopColor="currentColor" stopOpacity="0.18" />
          </linearGradient>
        </defs>
        {[11, 20, 29, 38].map((y, index) => (
          <path
            key={y}
            d={`M2 ${y} C 36 ${y - 18 + index * 2}, 58 ${y + 18 - index}, 88 ${y} S 137 ${y - 15 + index}, 168 ${y}`}
            stroke={`url(#${gradientId})`}
            strokeWidth="2"
            fill="none"
          />
        ))}
      </svg>
    </div>
  );
}

function WindowDots() {
  return (
    <div className="window-dots" aria-hidden="true">
      <span>—</span>
      <span>□</span>
      <span>×</span>
    </div>
  );
}

function UsageRow({
  kind,
  label,
  remaining,
  reset,
}: {
  kind: "short" | "long";
  label: string;
  remaining: number | null;
  reset: string;
}) {
  return (
    <section className={remaining === null ? "usage-row is-unavailable" : "usage-row"} aria-label={`${label}${remaining === null ? "暂无数据" : `剩余${remaining}%`}`}>
      <div className="usage-copy">
        <div className="usage-label">
          <span className="icon-badge">{kind === "short" ? icons.clock : icons.calendar}</span>
          <span>{label}</span>
        </div>
        <div className="usage-value">
          <span>剩余</span>
          <strong>{remaining === null ? "—" : `${remaining}%`}</strong>
        </div>
        <div className="reset-copy">{reset}</div>
      </div>
      <div className="usage-visual">
        {remaining === null ? <span className="usage-placeholder">等待真实数据</span> : <SequenceWave level={remaining} />}
        <div className="linear-progress" aria-hidden="true">
          <span style={{ width: `${remaining ?? 0}%` }} />
        </div>
      </div>
    </section>
  );
}

type SummaryItem = { icon: ReactNode; label: string; value: string; unit: string; warning?: boolean };

const summaryItems: SummaryItem[] = [
  { icon: icons.tokens, label: "今日 Token", value: "128", unit: "万" },
  { icon: icons.chart, label: "近 7 天", value: "642", unit: "万" },
  { icon: icons.clock, label: "已工作", value: "58", unit: "分钟" },
  { icon: icons.bell, label: "距离提醒", value: "2", unit: "分钟", warning: true },
];

function SummaryMetrics({ compact = false, items = summaryItems }: { compact?: boolean; items?: SummaryItem[] }) {
  return (
    <div className={compact ? "summary-metrics is-compact" : "summary-metrics"}>
      {items.map((item) => (
        <div className={item.warning ? "summary-item is-warning" : "summary-item"} key={item.label}>
          <span className="summary-icon">{item.icon}</span>
          <span className="summary-label">{item.label}</span>
          <span className="summary-number">
            <strong>{item.value}</strong>
            <small>{item.unit}</small>
          </span>
        </div>
      ))}
    </div>
  );
}

type StatusTone = "good" | "warning" | "error" | "neutral";

interface DesktopViewModel {
  planLabel: string;
  statusTitle: string;
  statusMessage: string | null;
  statusTone: StatusTone;
  shortRemaining: number | null;
  shortReset: string;
  longRemaining: number | null;
  longReset: string;
  summary: SummaryItem[];
  sourceText: string;
}

const demoViewModel: DesktopViewModel = {
  planLabel: "PLUS",
  statusTitle: "工作余量充足",
  statusMessage: null,
  statusTone: "good",
  shortRemaining: 72,
  shortReset: "3小时12分后重置",
  longRemaining: 41,
  longReset: "周二 10:00 重置",
  summary: summaryItems,
  sourceText: "数据来自 Codex · 刚刚更新",
};

function runtimeSummary(snapshot: UsageSnapshot | null): SummaryItem[] {
  const today = formatTokenMetric(snapshot?.tokenUsage?.todayTokens);
  const lastWeek = formatTokenMetric(snapshot?.tokenUsage?.last7DaysTokens);
  return [
    { icon: icons.tokens, label: "今日 Token", value: today.value, unit: today.unit },
    { icon: icons.chart, label: "近 7 天", value: lastWeek.value, unit: lastWeek.unit },
    { icon: icons.clock, label: "已工作", value: "—", unit: "" },
    { icon: icons.bell, label: "距离提醒", value: "—", unit: "", warning: false },
  ];
}

function toDesktopViewModel(
  runtime: boolean,
  snapshot: UsageSnapshot | null,
  loading: boolean,
  error: string | null,
): DesktopViewModel {
  if (!runtime) return demoViewModel;
  if (error) {
    return {
      planLabel: "未连接",
      statusTitle: "暂时无法读取用量",
      statusMessage: error,
      statusTone: "error",
      shortRemaining: null,
      shortReset: "请检查 Codex CLI",
      longRemaining: null,
      longReset: "稍后可重新读取",
      summary: runtimeSummary(null),
      sourceText: "Codex app-server · 读取失败",
    };
  }
  if (!snapshot) {
    return {
      planLabel: "连接中",
      statusTitle: loading ? "正在读取真实用量" : "等待 Codex 数据",
      statusMessage: "禾序只通过本机 app-server 获取账户状态。",
      statusTone: "neutral",
      shortRemaining: null,
      shortReset: "正在读取",
      longRemaining: null,
      longReset: "正在读取",
      summary: runtimeSummary(null),
      sourceText: "正在连接 Codex app-server",
    };
  }
  if (snapshot.status !== "ready") {
    return {
      planLabel: snapshot.status === "signed_out" ? "未登录" : "不支持",
      statusTitle: snapshot.status === "signed_out" ? "请先登录 Codex" : "当前登录方式不支持",
      statusMessage: snapshot.message,
      statusTone: snapshot.status === "signed_out" ? "neutral" : "warning",
      shortRemaining: null,
      shortReset: "没有可显示的账户额度",
      longRemaining: null,
      longReset: "不会使用本地 Token 估算",
      summary: runtimeSummary(snapshot),
      sourceText: `${snapshot.source} · ${formatUpdatedAt(snapshot.updatedAt)}`,
    };
  }

  const selected = snapshot.rateLimits?.selected;
  const shortRemaining = selected?.primary ? Math.round(selected.primary.remainingPercent) : null;
  const longRemaining = selected?.secondary ? Math.round(selected.secondary.remainingPercent) : null;
  const lowest = Math.min(shortRemaining ?? 100, longRemaining ?? 100);
  const reached = Boolean(selected?.rateLimitReachedType);
  const statusTone: StatusTone = reached ? "error" : lowest <= 20 ? "warning" : "good";
  const statusTitle = reached ? "账户额度暂不可用" : lowest <= 20 ? "用量余量偏低" : "工作余量充足";

  return {
    planLabel: formatPlanLabel(snapshot.account?.planType ?? selected?.planType),
    statusTitle,
    statusMessage: snapshot.message,
    statusTone,
    shortRemaining,
    shortReset: formatResetTime(selected?.primary?.resetsAt ?? null),
    longRemaining,
    longReset: formatResetTime(selected?.secondary?.resetsAt ?? null),
    summary: runtimeSummary(snapshot),
    sourceText: `${snapshot.source} · ${formatUpdatedAt(snapshot.updatedAt)}`,
  };
}

function DesktopPanel({
  onSettings,
  runtime = false,
  snapshot = null,
  loading = false,
  error = null,
  onRefresh,
}: {
  onSettings: () => void;
  runtime?: boolean;
  snapshot?: UsageSnapshot | null;
  loading?: boolean;
  error?: string | null;
  onRefresh?: () => void;
}) {
  const view = toDesktopViewModel(runtime, snapshot, loading, error);
  return (
    <article className="desktop-panel">
      <header className="panel-header">
        <div className="brand-lockup">
          <BrandMark />
          <div>
            <div className="brand-title">禾序</div>
            <div className="brand-subtitle">Codex 用量助手</div>
          </div>
        </div>
        <div className="panel-head-actions">
          <span className="live-pill"><i /> {view.planLabel} · 实时</span>
          <button className="icon-button" type="button" aria-label="打开提醒设置" onClick={onSettings}>
            {icons.settings}
          </button>
        </div>
        <WindowDots />
      </header>

      <section className={`status-block is-${view.statusTone}`}>
        <div>
          <span className="section-eyebrow">今日状态</span>
          <div className="status-title"><span className="status-check">{icons.check}</span>{view.statusTitle}</div>
          {view.statusMessage ? <div className="status-message">{view.statusMessage}</div> : null}
        </div>
        <div className="status-wave"><SequenceWave level={view.shortRemaining ?? 0} /></div>
      </section>

      <UsageRow kind="short" label="短周期" remaining={view.shortRemaining} reset={view.shortReset} />
      <UsageRow kind="long" label="长周期" remaining={view.longRemaining} reset={view.longReset} />
      <SummaryMetrics items={view.summary} />

      <footer className="panel-footer">
        <div className="footer-source">
          {runtime ? (
            <button className="source-button" type="button" onClick={onRefresh} disabled={loading}>
              <i className={loading ? "source-dot is-loading" : "source-dot"} />{view.sourceText}
            </button>
          ) : <span><i className="source-dot" />{view.sourceText}</span>}
        </div>
        <div className="footer-bottom">
          <span className="creator"><i className="sprout">⌁</i>沐青禾开发</span>
          <div className="display-mode" aria-label="显示方式演示">
            <button type="button">贴附</button>
            <button className="is-active" type="button">桌面</button>
          </div>
        </div>
      </footer>
    </article>
  );
}

function AttachedRail() {
  return (
    <section className="workspace-demo" aria-label="贴附模式演示">
      <div className="workspace-titlebar">
        <span className="workspace-menu">☰</span>
        <WindowDots />
      </div>
      <div className="workspace-sidebar">
        <span className="skeleton wide" />
        <span className="skeleton" />
        <span className="skeleton short" />
        <span className="skeleton" />
        <span className="skeleton short" />
      </div>
      <div className="workspace-content">
        <span className="skeleton wide" />
        <span className="skeleton line" />
        <span className="skeleton line medium" />
        <span className="skeleton line" />
        <span className="skeleton line short" />
        <div className="workspace-composer"><span /><span /></div>
      </div>
      <aside className="attached-rail">
        <div className="rail-top">
          <BrandMark compact />
          <span className="rail-live" />
        </div>
        <div className="rail-metric">
          <span>短</span>
          <strong>72</strong>
          <SequenceWave level={72} />
        </div>
        <div className="rail-metric">
          <span>长</span>
          <strong>41</strong>
          <SequenceWave level={41} />
        </div>
        <div className="rail-metric is-time">
          <span>工</span>
          <strong>58</strong>
          <small>分</small>
        </div>
        <button type="button" aria-label="贴附设置">{icons.settings}</button>
      </aside>
      <div className="mode-label attached-label">贴附模式 <span>不注入 Codex</span></div>
    </section>
  );
}

function ConversationCard() {
  return (
    <section className="conversation-wrap" aria-label="会话内卡片演示">
      <div className="mode-label conversation-label">会话内卡片 <span>结构化结果</span></div>
      <article className="conversation-card">
        <header>
          <div className="conversation-brand"><BrandMark compact /><strong>禾序</strong><span>Codex 用量助手</span></div>
          <span className="live-pill compact"><i /> PLUS · 实时</span>
          <span className="conversation-source">数据来自 Codex · 刚刚更新</span>
          <button type="button" aria-label="更多操作">•••</button>
        </header>
        <div className="conversation-grid">
          <div className="mini-usage">
            <span>{icons.clock} 短周期</span>
            <strong>剩余 <b>72%</b></strong>
            <small>3小时12分后重置</small>
            <div className="linear-progress"><span style={{ width: "72%" }} /></div>
          </div>
          <div className="mini-usage">
            <span>{icons.calendar} 长周期</span>
            <strong>剩余 <b>41%</b></strong>
            <small>周二 10:00 重置</small>
            <div className="linear-progress"><span style={{ width: "41%" }} /></div>
          </div>
          <SummaryMetrics compact />
        </div>
      </article>
      <p>这是插件在会话中返回的结构化用量结果，不是虚构的官方侧边栏插槽。</p>
    </section>
  );
}

function SettingsSheet({ open, onClose }: { open: boolean; onClose: () => void }) {
  return (
    <div className={open ? "settings-backdrop is-open" : "settings-backdrop"} onClick={onClose}>
      <aside className="settings-sheet" onClick={(event) => event.stopPropagation()}>
        <header><div><span>提醒设置</span><small>演示交互</small></div><button onClick={onClose} type="button">×</button></header>
        <label className="setting-row"><span><strong>工作提醒</strong><small>活跃满 60 分钟提醒</small></span><input type="checkbox" defaultChecked /></label>
        <label className="setting-row"><span><strong>休息后重置</strong><small>连续休息 10 分钟</small></span><input type="checkbox" defaultChecked /></label>
        <label className="setting-row"><span><strong>松弛模式</strong><small>默认关闭，可使用幽默文案</small></span><input type="checkbox" /></label>
        <blockquote>“你已经工作了 1小时，再忙也要导一管”</blockquote>
        <p>这句话只会在用户主动启用松弛模式后出现。</p>
      </aside>
    </div>
  );
}

function Toolbar({
  theme,
  setTheme,
  view,
  setView,
}: {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  view: View;
  setView: (view: View) => void;
}) {
  const views: { id: View; label: string }[] = [
    { id: "board", label: "完整稿" },
    { id: "desktop", label: "桌面" },
    { id: "attached", label: "贴附" },
    { id: "conversation", label: "会话" },
  ];
  return (
    <nav className="prototype-toolbar" aria-label="原型视图控制">
      <div className="toolbar-brand"><BrandMark compact /><span><strong>禾序 UI</strong><small>高保真原型 · 演示数据</small></span></div>
      <div className="segmented-control">
        {views.map((item) => (
          <button className={view === item.id ? "is-active" : ""} key={item.id} onClick={() => setView(item.id)} type="button">
            {item.label}
          </button>
        ))}
      </div>
      <button className="theme-toggle" type="button" onClick={() => setTheme(theme === "light" ? "dark" : "light")}>
        {theme === "light" ? icons.moon : icons.sun}
        <span>{theme === "light" ? "暗色" : "亮色"}</span>
      </button>
    </nav>
  );
}

function App() {
  const params = useMemo(() => new URLSearchParams(window.location.search), []);
  const initialTheme = params.get("theme") === "dark" ? "dark" : "light";
  const requestedView = params.get("view");
  const captureMode = params.get("capture") === "1";
  const runtimeMode = params.get("runtime") === "tauri" && isTauriRuntime();
  const initialView: View = runtimeMode ? "desktop" : ["desktop", "attached", "conversation"].includes(requestedView ?? "")
    ? (requestedView as View)
    : "board";
  const [theme, setTheme] = useState<Theme>(initialTheme);
  const [view, setView] = useState<View>(initialView);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const usage = useCodexUsage(runtimeMode);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    document.querySelector('meta[name="theme-color"]')?.setAttribute("content", theme === "dark" ? "#101816" : "#f6f8f7");
  }, [theme]);

  return (
    <main className={`app-shell view-${view}${captureMode ? " capture-mode" : ""}${runtimeMode ? " runtime-mode" : ""}`}>
      <Toolbar theme={theme} setTheme={setTheme} view={view} setView={setView} />
      <div className="board-heading">
        <span>HEXU / UI CONCEPT 01</span>
        <h1>把用量排成节奏，<br />而不是一块警告牌。</h1>
        <p>三段序列波分别承接短周期、长周期与工作时间。数值清楚，但不抢走整个界面。</p>
      </div>
      <div className="design-board">
        <div className="desktop-stage">
          <div className="mode-label desktop-label">桌面悬浮 <span>380 × 520</span></div>
          <DesktopPanel
            onSettings={() => setSettingsOpen(true)}
            runtime={runtimeMode}
            snapshot={usage.snapshot}
            loading={usage.loading}
            error={usage.error}
            onRefresh={() => void usage.refresh()}
          />
        </div>
        <AttachedRail />
        <ConversationCard />
      </div>
      <div className="palette-strip" aria-label="禾序色彩系统">
        <span style={{ background: "#F6F8F7", color: "#1B2725" }}><i />雾白</span>
        <span style={{ background: "#1B2725", color: "#fff" }}><i />墨青</span>
        <span style={{ background: "#28B8A6", color: "#fff" }}><i />禾青</span>
        <span style={{ background: "#CDEEE7", color: "#1B2725" }}><i />浅青</span>
        <span style={{ background: "#E5A84B", color: "#1B2725" }}><i />提醒</span>
      </div>
      <SettingsSheet open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </main>
  );
}

export default App;
