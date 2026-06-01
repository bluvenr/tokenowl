import { useAppStore } from "@/stores/appStore";
import { formatCost, formatTokens, getSourceColor } from "@/lib/format";
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
  CartesianGrid,
} from "recharts";

// ─── Cost Overview ──────────────────────────────────────────────────

export function CostOverview() {
  const { summary, period, setPeriod } = useAppStore();
  const { t } = useTranslation();

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-sm font-medium text-muted-foreground">
          {t("dashboard.overview")}
        </CardTitle>
        <div className="flex gap-1">
          {(["today", "week", "month"] as const).map((p) => (
            <button
              key={p}
              onClick={() => setPeriod(p)}
              className={`px-2.5 py-1 text-xs rounded-md transition-colors ${
                period === p
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-muted"
              }`}
            >
              {t(`period.${p}`)}
            </button>
          ))}
        </div>
      </CardHeader>
      <CardContent>
        <div className="text-3xl font-bold tracking-tight">
          {summary ? formatCost(summary.totalCostUsd) : "$0.00"}
        </div>
        <div className="mt-2 flex gap-4 text-xs text-muted-foreground">
          <span>{summary ? formatTokens(summary.totalTokens) : "0"} {t("dashboard.tokens")}</span>
          <span>{summary?.sessionCount ?? 0} {t("dashboard.sessions")}</span>
        </div>
        {summary && summary.totalTokens > 0 && (
          <div className="mt-3 flex gap-2 text-xs text-muted-foreground">
            <span>In: {formatTokens(summary.inputTokens)}</span>
            <span>Out: {formatTokens(summary.outputTokens)}</span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// ─── Tool Breakdown (PieChart) ──────────────────────────────────────

export function ToolBreakdown() {
  const { bySource } = useAppStore();
  const { t } = useTranslation();

  const chartData = bySource.map((s) => ({
    name: s.displayName,
    value: s.costUsd,
    color: getSourceColor(s.source),
    percentage: s.percentage,
  }));

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium text-muted-foreground">
          {t("dashboard.tool_breakdown")}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {bySource.length === 0 ? (
          <EmptyState message={t("dashboard.no_data")} />
        ) : (
          <div className="flex items-center gap-4">
            <div className="w-32 h-32">
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie
                    data={chartData}
                    dataKey="value"
                    cx="50%"
                    cy="50%"
                    innerRadius={30}
                    outerRadius={55}
                    strokeWidth={2}
                    stroke="var(--color-background)"
                  >
                    {chartData.map((entry, index) => (
                      <Cell key={index} fill={entry.color} />
                    ))}
                  </Pie>
                </PieChart>
              </ResponsiveContainer>
            </div>
            <div className="flex-1 space-y-1.5">
              {bySource.map((s) => (
                <div key={s.source} className="flex items-center gap-2">
                  <div
                    className="w-2.5 h-2.5 rounded-full shrink-0"
                    style={{ backgroundColor: getSourceColor(s.source) }}
                  />
                  <span className="text-sm flex-1 truncate">{s.displayName}</span>
                  <span className="text-sm font-medium tabular-nums">{formatCost(s.costUsd)}</span>
                  <span className="text-xs text-muted-foreground w-10 text-right tabular-nums">
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

// ─── Trend Chart (AreaChart with granularity switch) ────────────────

export function TrendChart() {
  const { trend, trendGranularity, setTrendGranularity } = useAppStore();
  const { t } = useTranslation();

  const displayData = trend.map((p) => ({
    ...p,
    displayDate: p.date.length > 10 ? p.date.slice(5) : p.date,
  }));

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-sm font-medium text-muted-foreground">
          {t("dashboard.trend")}
        </CardTitle>
        <div className="flex gap-1">
          {([
            { key: "hourly", label: t("period.hourly", "Hourly") },
            { key: "daily", label: t("period.daily", "Daily") },
            { key: "weekly", label: t("period.weekly", "Weekly") },
          ] as const).map((g) => (
            <button
              key={g.key}
              onClick={() => setTrendGranularity(g.key)}
              className={`px-2.5 py-1 text-xs rounded-md transition-colors ${
                trendGranularity === g.key
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-muted"
              }`}
            >
              {g.label}
            </button>
          ))}
        </div>
      </CardHeader>
      <CardContent>
        {displayData.length === 0 ? (
          <EmptyState message={t("dashboard.no_data")} />
        ) : (
          <div className="h-48">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={displayData} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
                <defs>
                  <linearGradient id="costGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="var(--color-primary)" stopOpacity={0.3} />
                    <stop offset="100%" stopColor="var(--color-primary)" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border)" vertical={false} />
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
                  tickFormatter={(v: number) => formatCost(v)}
                  width={50}
                />
                <Tooltip
                  content={({ active, payload }) => {
                    if (active && payload?.[0]) {
                      const data = payload[0].payload;
                      return (
                        <div className="rounded-lg border bg-popover p-2 text-xs shadow-md">
                          <div className="font-medium">{data.date}</div>
                          <div className="text-muted-foreground">
                            {formatCost(data.costUsd)} · {formatTokens(data.totalTokens)}
                          </div>
                        </div>
                      );
                    }
                    return null;
                  }}
                />
                <Area
                  type="monotone"
                  dataKey="costUsd"
                  stroke="var(--color-primary)"
                  strokeWidth={2}
                  fill="url(#costGradient)"
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

  const chartData = byModel.slice(0, 8).map((m) => ({
    name: m.model.length > 20 ? m.model.slice(0, 18) + "..." : m.model,
    cost: m.costUsd,
    tokens: m.totalTokens,
    source: m.source,
  }));

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium text-muted-foreground">
          {t("dashboard.model_detail")}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {byModel.length === 0 ? (
          <EmptyState message={t("dashboard.no_data")} />
        ) : (
          <div className="h-52">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={chartData} layout="vertical" margin={{ top: 0, right: 4, bottom: 0, left: 0 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border)" horizontal={false} />
                <XAxis
                  type="number"
                  tick={{ fontSize: 10, fill: "var(--color-muted-foreground)" }}
                  tickLine={false}
                  axisLine={false}
                  tickFormatter={(v: number) => formatCost(v)}
                />
                <YAxis
                  type="category"
                  dataKey="name"
                  tick={{ fontSize: 10, fill: "var(--color-muted-foreground)" }}
                  tickLine={false}
                  axisLine={false}
                  width={120}
                />
                <Tooltip
                  content={({ active, payload }) => {
                    if (active && payload?.[0]) {
                      const data = payload[0].payload;
                      return (
                        <div className="rounded-lg border bg-popover p-2 text-xs shadow-md">
                          <div className="font-medium">{data.name}</div>
                          <div className="text-muted-foreground">
                            {formatCost(data.cost)} · {formatTokens(data.tokens)}
                          </div>
                        </div>
                      );
                    }
                    return null;
                  }}
                />
                <Bar dataKey="cost" fill="var(--color-chart-1)" radius={[0, 4, 4, 0]} barSize={18} />
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
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium text-muted-foreground">
          {t("dashboard.recent_sessions")}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {sessions.length === 0 ? (
          <EmptyState message={t("dashboard.no_data")} />
        ) : (
          <div className="space-y-0.5 max-h-56 overflow-y-auto">
            {sessions.slice(0, 15).map((s) => (
              <div
                key={`${s.sessionId}-${s.source}`}
                className="flex items-center gap-2 py-1.5 px-2 rounded-md hover:bg-muted/50 transition-colors"
              >
                <div
                  className="w-1.5 h-1.5 rounded-full shrink-0"
                  style={{ backgroundColor: getSourceColor(s.source) }}
                />
                <span className="text-xs text-muted-foreground w-20 shrink-0 truncate">
                  {s.source.replace("_", " ")}
                </span>
                <span className="text-sm flex-1 truncate">{s.model}</span>
                <span className="text-sm font-medium tabular-nums">{formatCost(s.costUsd)}</span>
                <span className="text-xs text-muted-foreground tabular-nums w-14 text-right">
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
