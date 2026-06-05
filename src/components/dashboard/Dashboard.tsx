import { useState } from "react";
import { useAppStore } from "@/stores/appStore";
import { formatCost, formatTokens, getModelColor, getSourceColor } from "@/lib/format";
import { useTranslation } from "react-i18next";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "./index";
import {
  PieChart,
  Pie,
  Cell,
  AreaChart,
  Area,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

type Dimension = "cost" | "tokens";

function SegmentToggle({ value, onChange, options }: { 
  value: string; 
  onChange: (v: string) => void;
  options: { key: string; label: string }[];
}) {
  return (
    <div className="flex gap-0.5 rounded-lg bg-muted/60 p-0.5">
      {options.map((opt) => (
        <button
          key={opt.key}
          onClick={() => onChange(opt.key)}
          className={`px-2.5 py-1 text-[11px] font-medium rounded-md transition-all ${
            value === opt.key
              ? "bg-background text-foreground shadow-sm"
              : "text-muted-foreground hover:text-foreground/80"
          }`}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}

// ─── Cost Overview ──────────────────────────────────────────────────

export function CostOverview() {
  const { summary, period, setPeriod } = useAppStore();
  const { t } = useTranslation();

  const periodOptions = [
    { key: "today", label: t("period.today") },
    { key: "week", label: t("period.week") },
    { key: "month", label: t("period.month") },
  ];

  return (
    <Card className="border-border/60">
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
          {t("dashboard.total_cost")}
        </CardTitle>
        <SegmentToggle value={period} onChange={(v) => setPeriod(v as "today" | "week" | "month")} options={periodOptions} />
      </CardHeader>
      <CardContent className="pt-2">
        <div className="text-4xl font-bold tracking-tight">
          {summary ? formatCost(summary.totalCostUsd) : "$0.00"}
        </div>
        <div className="mt-3 flex items-center gap-4 text-xs text-muted-foreground">
          <div className="flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 rounded-full bg-blue-400" />
            <span>{summary ? formatTokens(summary.totalTokens) : "0"} {t("dashboard.tokens")}</span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 rounded-full bg-purple-400" />
            <span>{summary?.sessionCount ?? 0} {t("dashboard.sessions")}</span>
          </div>
        </div>
        {summary && summary.totalTokens > 0 && (
          <div className="mt-4 pt-4 border-t border-border/40 grid grid-cols-2 gap-4">
            <div>
              <div className="text-[10px] text-muted-foreground uppercase tracking-wide">{t("dashboard.lbl_in")}</div>
              <div className="text-sm font-semibold mt-0.5">{formatTokens(summary.inputTokens)}</div>
            </div>
            <div>
              <div className="text-[10px] text-muted-foreground uppercase tracking-wide">{t("dashboard.lbl_out")}</div>
              <div className="text-sm font-semibold mt-0.5">{formatTokens(summary.outputTokens)}</div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// ─── Model Summary (PieChart) ───────────────────────────────────────

export function ToolBreakdown() {
  const { byModel } = useAppStore();
  const { t } = useTranslation();
  const [dim, setDim] = useState<Dimension>("cost");

  const totalCostAll = byModel.reduce((sum, m) => sum + m.costUsd, 0);
  const totalTokensAll = byModel.reduce((sum, m) => sum + m.totalTokens, 0);

  const chartData = byModel.slice(0, 5).map((m, i) => ({
    name: m.model.length > 24 ? m.model.slice(0, 22) + "..." : m.model,
    value: dim === "cost" ? m.costUsd : m.totalTokens,
    color: getModelColor(m.model, i),
    costUsd: m.costUsd,
    totalTokens: m.totalTokens,
    percentage: dim === "cost"
      ? (totalCostAll > 0 ? (m.costUsd / totalCostAll) * 100 : 0)
      : (totalTokensAll > 0 ? (m.totalTokens / totalTokensAll) * 100 : 0),
  }));

  const dimOptions = [
    { key: "cost", label: t("dashboard.cost") },
    { key: "tokens", label: t("dashboard.tokens_dim") },
  ];

  return (
    <Card className="border-border/60">
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
          {t("dashboard.model_breakdown")}
        </CardTitle>
        <SegmentToggle value={dim} onChange={(v) => setDim(v as Dimension)} options={dimOptions} />
      </CardHeader>
      <CardContent className="pt-2">
        {byModel.length === 0 ? (
          <EmptyState message={t("dashboard.no_data")} />
        ) : (
          <div className="flex items-center gap-5">
            <div className="w-28 h-28 shrink-0">
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie
                    data={chartData}
                    dataKey="value"
                    cx="50%"
                    cy="50%"
                    innerRadius={32}
                    outerRadius={52}
                    strokeWidth={0}
                    paddingAngle={2}
                  >
                    {chartData.map((entry, index) => (
                      <Cell key={index} fill={entry.color} />
                    ))}
                  </Pie>
                </PieChart>
              </ResponsiveContainer>
            </div>
            <div className="flex-1 space-y-2 min-w-0">
              {chartData.map((s) => (
                <div key={s.name} className="flex items-center gap-2.5">
                  <div
                    className="w-2 h-2 rounded-full shrink-0"
                    style={{ backgroundColor: s.color }}
                  />
                  <span className="text-xs flex-1 truncate text-foreground/90">{s.name}</span>
                  <span className="text-xs font-medium tabular-nums">
                    {dim === "cost" ? formatCost(s.costUsd) : formatTokens(s.totalTokens)}
                  </span>
                  <span className="text-[10px] text-muted-foreground w-8 text-right tabular-nums">
                    {s.percentage.toFixed(0)}%
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// ─── Trend Chart (AreaChart) ────────────────────────────────────────

export function TrendChart() {
  const { trend, trendGranularity, setTrendGranularity } = useAppStore();
  const { t } = useTranslation();
  const [dim, setDim] = useState<Dimension>("cost");

  const displayData = trend.map((p) => ({
    ...p,
    displayDate: p.date.length > 10 ? p.date.slice(5) : p.date,
  }));

  const formatAxis = (v: number) => dim === "cost" ? formatCost(v) : formatTokens(v);

  const dimOptions = [
    { key: "cost", label: t("dashboard.cost") },
    { key: "tokens", label: t("dashboard.tokens_dim") },
  ];

  const granularityOptions = [
    { key: "hourly", label: t("period.hourly", "Hourly") },
    { key: "daily", label: t("period.daily", "Daily") },
    { key: "weekly", label: t("period.weekly", "Weekly") },
  ];

  return (
    <Card className="border-border/60">
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
          {t("dashboard.trend")}
        </CardTitle>
        <div className="flex items-center gap-3">
          <SegmentToggle value={dim} onChange={(v) => setDim(v as Dimension)} options={dimOptions} />
          <SegmentToggle value={trendGranularity} onChange={(v) => setTrendGranularity(v as "hourly" | "daily" | "weekly")} options={granularityOptions} />
        </div>
      </CardHeader>
      <CardContent className="pt-2">
        {displayData.length === 0 ? (
          <EmptyState message={t("dashboard.no_data")} />
        ) : (
          <div className="h-56">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={displayData} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
                <defs>
                  <linearGradient id="costGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="var(--color-primary)" stopOpacity={0.2} />
                    <stop offset="100%" stopColor="var(--color-primary)" stopOpacity={0} />
                  </linearGradient>
                  <linearGradient id="tokenGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="var(--color-chart-2, #22c55e)" stopOpacity={0.2} />
                    <stop offset="100%" stopColor="var(--color-chart-2, #22c55e)" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <XAxis
                  dataKey="displayDate"
                  tick={{ fontSize: 10, fill: "var(--color-muted-foreground)" }}
                  tickLine={false}
                  axisLine={false}
                  interval="preserveStartEnd"
                />
                <YAxis
                  tick={{ fontSize: 10, fill: "var(--color-muted-foreground)" }}
                  tickLine={false}
                  axisLine={false}
                  tickFormatter={formatAxis}
                  width={50}
                />
                <Tooltip
                  content={({ active, payload }) => {
                    if (active && payload?.[0]) {
                      const data = payload[0].payload;
                      return (
                        <div className="rounded-lg border bg-popover p-2.5 text-xs shadow-lg">
                          <div className="font-medium mb-1">{data.date}</div>
                          <div className="flex items-center gap-2 text-muted-foreground">
                            <span>{formatCost(data.costUsd)}</span>
                            <span className="text-border">·</span>
                            <span>{formatTokens(data.totalTokens)}</span>
                          </div>
                        </div>
                      );
                    }
                    return null;
                  }}
                />
                <Area
                  type="monotone"
                  dataKey={dim === "cost" ? "costUsd" : "totalTokens"}
                  stroke={dim === "cost" ? "var(--color-primary)" : "var(--color-chart-2, #22c55e)"}
                  strokeWidth={2}
                  fill={dim === "cost" ? "url(#costGradient)" : "url(#tokenGradient)"}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// ─── Model Breakdown (BarChart) ─────────────────────────────────────

export function ModelBreakdown() {
  const { byModel } = useAppStore();
  const { t } = useTranslation();
  const [dim, setDim] = useState<Dimension>("cost");

  const chartData = byModel.slice(0, 6).map((m, i) => ({
    name: m.model.length > 24 ? m.model.slice(0, 22) + "..." : m.model,
    cost: m.costUsd,
    tokens: m.totalTokens,
    color: getModelColor(m.model, i),
  }));

  const formatAxis = (v: number) => dim === "cost" ? formatCost(v) : formatTokens(v);

  const dimOptions = [
    { key: "cost", label: t("dashboard.cost") },
    { key: "tokens", label: t("dashboard.tokens_dim") },
  ];

  return (
    <Card className="border-border/60">
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
          {t("dashboard.model_detail")}
        </CardTitle>
        <SegmentToggle value={dim} onChange={(v) => setDim(v as Dimension)} options={dimOptions} />
      </CardHeader>
      <CardContent className="pt-2">
        {byModel.length === 0 ? (
          <EmptyState message={t("dashboard.no_data")} />
        ) : (
          <div className="h-52">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={chartData} layout="vertical" margin={{ top: 0, right: 8, bottom: 0, left: 0 }}>
                <XAxis
                  type="number"
                  tick={{ fontSize: 10, fill: "var(--color-muted-foreground)" }}
                  tickLine={false}
                  axisLine={false}
                  tickFormatter={formatAxis}
                />
                <YAxis
                  type="category"
                  dataKey="name"
                  tick={{ fontSize: 10, fill: "var(--color-muted-foreground)" }}
                  tickLine={false}
                  axisLine={false}
                  width={110}
                />
                <Tooltip
                  cursor={{ fill: "var(--color-muted)", opacity: 0.3 }}
                  content={({ active, payload }) => {
                    if (active && payload?.[0]) {
                      const data = payload[0].payload;
                      return (
                        <div className="rounded-lg border bg-popover p-2.5 text-xs shadow-lg">
                          <div className="font-medium mb-1">{data.name}</div>
                          <div className="flex items-center gap-2 text-muted-foreground">
                            <span>{formatCost(data.cost)}</span>
                            <span className="text-border">·</span>
                            <span>{formatTokens(data.tokens)}</span>
                          </div>
                        </div>
                      );
                    }
                    return null;
                  }}
                />
                <Bar
                  dataKey={dim === "cost" ? "cost" : "tokens"}
                  fill="var(--color-chart-1)"
                  radius={[0, 4, 4, 0]}
                  barSize={16}
                />
              </BarChart>
            </ResponsiveContainer>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// ─── Session List ───────────────────────────────────────────────────

export function SessionList() {
  const { sessions } = useAppStore();
  const { t } = useTranslation();

  return (
    <Card className="border-border/60">
      <CardHeader className="pb-2">
        <CardTitle className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
          {t("dashboard.recent_sessions")}
        </CardTitle>
      </CardHeader>
      <CardContent className="pt-2">
        {sessions.length === 0 ? (
          <EmptyState message={t("dashboard.no_data")} />
        ) : (
          <div className="space-y-1 max-h-52 overflow-y-auto">
            {sessions.slice(0, 12).map((s) => (
              <div
                key={`${s.sessionId}-${s.source}`}
                className="flex items-center gap-2.5 py-2 px-2.5 rounded-lg hover:bg-muted/40 transition-colors"
              >
                <div
                  className="w-2 h-2 rounded-full shrink-0"
                  style={{ backgroundColor: getSourceColor(s.source) }}
                />
                <span className="text-[10px] text-muted-foreground w-16 shrink-0 truncate uppercase tracking-wide">
                  {s.source.replace("_", " ")}
                </span>
                <span className="text-sm flex-1 truncate font-medium">{s.model}</span>
                <span className="text-sm font-semibold tabular-nums">{formatCost(s.costUsd)}</span>
                <span className="text-[10px] text-muted-foreground tabular-nums w-12 text-right">
                  {formatTokens(s.totalTokens)}
                </span>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
