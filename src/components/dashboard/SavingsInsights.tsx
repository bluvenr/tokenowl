import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "@/stores/appStore";
import { formatCost, getSourceColor } from "@/lib/format";
import type { SettingsTab } from "@/App";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Database,
  BarChart3,
  TrendingUp,
  AlertTriangle,
  ChevronDown,
  ChevronUp,
  Zap,
  ArrowRight,
} from "lucide-react";

// ─── Cache Insight Card ───────────────────────────────────────────────

function CacheInsightCard() {
  const { savings } = useAppStore();
  const { t } = useTranslation();

  const cache = savings?.cacheEfficiency ?? [];
  // Only show sources that have cache data
  const sourcesWithCache = cache.filter((c) => c.hitRate !== null);
  const totalSavings = cache.reduce((sum, c) => sum + c.cacheCostSavings, 0);

  // Pick the primary source (highest hit rate)
  const primary = sourcesWithCache.reduce(
    (best, c) =>
      !best || (c.hitRate ?? 0) > (best.hitRate ?? 0) ? c : best,
    null as (typeof cache)[number] | null
  );

  return (
    <Card className="h-full">
      <CardHeader className="flex flex-row items-center gap-2 pb-2">
        <Database className="w-4 h-4 text-blue-500" />
        <CardTitle className="text-xs font-medium text-muted-foreground">
          {t("savings.cache_efficiency")}
        </CardTitle>
      </CardHeader>
      <CardContent className="pt-0">
        {sourcesWithCache.length === 0 ? (
          <div className="text-sm text-muted-foreground py-3">
            {t("savings.cache_no_data")}
          </div>
        ) : (
          <>
            {/* Primary source: large hit rate display */}
            <div className="flex items-baseline gap-2 mb-2">
              <span className="text-2xl font-bold tabular-nums">
                {((primary?.hitRate ?? 0) * 100).toFixed(0)}%
              </span>
              <span className="text-xs text-muted-foreground">
                {primary?.displayName} {t("savings.cache_hit_rate")}
              </span>
            </div>

            {/* All sources list */}
            {sourcesWithCache.length > 1 && (
              <div className="space-y-1 mb-2">
                {sourcesWithCache.map((c) => (
                  <div key={c.source} className="flex items-center gap-1.5 text-xs">
                    <div
                      className="w-2 h-2 rounded-full shrink-0"
                      style={{ backgroundColor: getSourceColor(c.source) }}
                    />
                    <span className="flex-1 truncate text-muted-foreground">
                      {c.displayName}
                    </span>
                    <span className="tabular-nums font-medium">
                      {((c.hitRate ?? 0) * 100).toFixed(0)}%
                    </span>
                  </div>
                ))}
              </div>
            )}

            {/* Savings estimate */}
            {totalSavings > 0 && (
              <div className="text-xs text-green-600 dark:text-green-400 flex items-center gap-1">
                <Zap className="w-3 h-3" />
                {t("savings.cache_saved", { amount: formatCost(totalSavings) })}
              </div>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
}

// ─── Model Insight Card ───────────────────────────────────────────────

function ModelInsightCard() {
  const { savings } = useAppStore();
  const { t } = useTranslation();

  const analysis = savings?.modelAnalysis;
  const insights = analysis?.insights ?? [];
  const topModels = insights.slice(0, 3);
  const concentration = analysis?.concentrationIndex ?? 0;

  const concLabel =
    concentration > 0.7
      ? t("savings.model_high")
      : concentration > 0.4
        ? t("savings.model_medium")
        : t("savings.model_low");

  const concColor =
    concentration > 0.7
      ? "text-red-500"
      : concentration > 0.4
        ? "text-amber-500"
        : "text-green-500";

  return (
    <Card className="h-full">
      <CardHeader className="flex flex-row items-center gap-2 pb-2">
        <BarChart3 className="w-4 h-4 text-purple-500" />
        <CardTitle className="text-xs font-medium text-muted-foreground">
          {t("savings.model_analysis")}
        </CardTitle>
      </CardHeader>
      <CardContent className="pt-0">
        {topModels.length === 0 ? (
          <div className="text-sm text-muted-foreground py-3">
            {t("savings.model_no_data")}
          </div>
        ) : (
          <>
            {/* Top models list */}
            <div className="space-y-1.5 mb-2">
              {topModels.map((m, i) => (
                <div key={`${m.source}-${m.model}`} className="flex items-center gap-1.5">
                  <span className="text-[10px] font-bold text-muted-foreground w-3.5 shrink-0">
                    #{i + 1}
                  </span>
                  <div
                    className="w-2 h-2 rounded-full shrink-0"
                    style={{ backgroundColor: getSourceColor(m.source) }}
                  />
                  <span className="text-xs flex-1 truncate">{m.model}</span>
                  <span className="text-xs font-medium tabular-nums shrink-0">
                    {formatCost(m.costUsd)}
                  </span>
                  <span className="text-[10px] text-muted-foreground w-8 text-right tabular-nums shrink-0">
                    {m.costSharePct.toFixed(0)}%
                  </span>
                </div>
              ))}
            </div>

            {/* Top model details */}
            {topModels[0] && (
              <div className="flex gap-3 text-[10px] text-muted-foreground mb-2">
                <span>
                  {t("savings.model_per_session", {
                    cost: formatCost(topModels[0].costPerSession),
                  })}
                </span>
                <span>
                  {t("savings.model_per_m_tokens", {
                    cost: formatCost(topModels[0].costPerMillionTokens),
                  })}
                </span>
              </div>
            )}

            {/* Concentration indicator */}
            <div className="flex items-center gap-1.5">
              <span className="text-[10px] text-muted-foreground">
                {t("savings.model_concentration")}:
              </span>
              <span className={`text-[10px] font-semibold ${concColor}`}>
                {concLabel}
              </span>
              {concentration > 0.7 && analysis?.topCostModel && (
                <span className="text-[10px] text-muted-foreground truncate">
                  — {t("savings.model_concentration_hint", {
                    model: analysis.topCostModel.model,
                  })}
                </span>
              )}
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}

// ─── Forecast Card ────────────────────────────────────────────────────

function ForecastCard({ onNavigateToSettings }: { onNavigateToSettings?: (tab: SettingsTab) => void }) {
  const { savings } = useAppStore();
  const { t } = useTranslation();

  const fc = savings?.forecast;

  if (!fc) {
    return (
      <Card className="h-full">
        <CardHeader className="flex flex-row items-center gap-2 pb-2">
          <TrendingUp className="w-4 h-4 text-emerald-500" />
          <CardTitle className="text-xs font-medium text-muted-foreground">
            {t("savings.forecast")}
          </CardTitle>
        </CardHeader>
        <CardContent className="pt-0">
          <div className="text-sm text-muted-foreground py-3">
            {t("savings.forecast_no_data")}
          </div>
        </CardContent>
      </Card>
    );
  }

  const pctUsed =
    fc.monthlyLimit && fc.monthlyLimit > 0
      ? Math.min((fc.projectedMonthlyCost / fc.monthlyLimit) * 100, 100)
      : null;

  return (
    <Card className="h-full">
      <CardHeader className="flex flex-row items-center gap-2 pb-2">
        <TrendingUp className="w-4 h-4 text-emerald-500" />
        <CardTitle className="text-xs font-medium text-muted-foreground">
          {t("savings.forecast")}
        </CardTitle>
      </CardHeader>
      <CardContent className="pt-0">
        {/* Projected cost */}
        <div className="flex items-baseline gap-2 mb-1">
          <span className="text-2xl font-bold tabular-nums">
            {formatCost(fc.projectedMonthlyCost)}
          </span>
          <span className="text-xs text-muted-foreground">
            {t("savings.forecast_projected")}
          </span>
        </div>

        {/* Daily average + week change */}
        <div className="flex gap-3 text-xs text-muted-foreground mb-2">
          <span>
            {t("savings.forecast_daily_avg")}: {formatCost(fc.dailyAvgCost)}
          </span>
          {fc.weekOverWeekChangePct !== null && (
            <span
              className={
                fc.weekOverWeekChangePct > 0
                  ? "text-red-500"
                  : fc.weekOverWeekChangePct < 0
                    ? "text-green-500"
                    : ""
              }
            >
              {t("savings.forecast_week_change", {
                pct:
                  (fc.weekOverWeekChangePct > 0 ? "+" : "") +
                  fc.weekOverWeekChangePct.toFixed(0) +
                  "%",
              })}
            </span>
          )}
        </div>

        {/* Budget progress bar */}
        {pctUsed !== null && fc.monthlyLimit ? (
          <>
            <div className="w-full h-1.5 rounded-full bg-muted overflow-hidden mb-1">
              <div
                className={`h-full rounded-full transition-all ${
                  pctUsed >= 90
                    ? "bg-red-500"
                    : pctUsed >= 70
                      ? "bg-amber-500"
                      : "bg-green-500"
                }`}
                style={{ width: `${pctUsed}%` }}
              />
            </div>
            <div className="flex justify-between text-[10px] text-muted-foreground">
              <span>
                {formatCost(fc.dailyAvgCost * fc.daysElapsed)} / {formatCost(fc.monthlyLimit)}
              </span>
              <span>{pctUsed.toFixed(0)}%</span>
            </div>

            {/* Over budget warning */}
            {fc.projectedOverBudget && fc.budgetExhaustionDays !== null && (
              <div className="text-xs text-red-500 mt-1.5 flex items-center gap-1">
                <AlertTriangle className="w-3 h-3 shrink-0" />
                {t("savings.forecast_over_budget", {
                  days: fc.budgetExhaustionDays,
                })}
              </div>
            )}
            {!fc.projectedOverBudget && (
              <div className="text-xs text-green-600 dark:text-green-400 mt-1.5">
                {t("savings.forecast_on_track")}
              </div>
            )}
          </>
        ) : (
          <div className="flex items-center gap-2 mt-1">
            <span className="text-xs text-muted-foreground">
              {t("savings.forecast_no_limit")}
            </span>
            {onNavigateToSettings && (
              <button
                onClick={() => onNavigateToSettings("budget")}
                className="inline-flex items-center gap-1 text-xs font-medium text-blue-600 dark:text-blue-400 hover:underline shrink-0"
              >
                {t("savings.forecast_go_set")}
                <ArrowRight className="w-3 h-3" />
              </button>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// ─── Anomaly Card ─────────────────────────────────────────────────────

function AnomalyCard() {
  const { savings } = useAppStore();
  const { t } = useTranslation();

  const report = savings?.anomalyReport;
  const anomalies = report?.anomalies ?? [];

  return (
    <Card className="h-full">
      <CardHeader className="flex flex-row items-center gap-2 pb-2">
        <AlertTriangle className="w-4 h-4 text-amber-500" />
        <CardTitle className="text-xs font-medium text-muted-foreground">
          {t("savings.anomaly")}
        </CardTitle>
      </CardHeader>
      <CardContent className="pt-0">
        {anomalies.length === 0 ? (
          <div className="text-sm text-muted-foreground py-3">
            {t("savings.anomaly_none")}
          </div>
        ) : (
          <>
            {/* Summary */}
            <div className="flex items-baseline gap-2 mb-2">
              <span className="text-2xl font-bold tabular-nums">{anomalies.length}</span>
              <span className="text-xs text-muted-foreground">
                {t("savings.anomaly_detected", { count: anomalies.length })}
              </span>
            </div>

            {/* Top anomalies list */}
            <div className="space-y-1">
              {anomalies.slice(0, 4).map((a) => (
                <div
                  key={a.date}
                  className="flex items-center gap-1.5 text-xs"
                >
                  <span className="text-muted-foreground w-16 shrink-0 tabular-nums">
                    {a.date.slice(5)}
                  </span>
                  <span className="font-medium tabular-nums">
                    {formatCost(a.costUsd)}
                  </span>
                  <span className="text-red-500 text-[10px] font-semibold tabular-nums">
                    {t("savings.anomaly_spike", {
                      factor: a.deviationFactor.toFixed(1),
                    })}
                  </span>
                  {a.source && (
                    <span className="text-[10px] text-muted-foreground truncate">
                      {a.source}
                    </span>
                  )}
                </div>
              ))}
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}

// ─── Main Container ───────────────────────────────────────────────────

export function SavingsInsights({ onNavigateToSettings }: { onNavigateToSettings?: (tab: SettingsTab) => void }) {
  const { savings } = useAppStore();
  const { t } = useTranslation();
  const [collapsed, setCollapsed] = useState(false);

  // Don't render if no data at all
  if (!savings) return null;

  const hasAnyData =
    savings.cacheEfficiency.length > 0 ||
    savings.modelAnalysis.insights.length > 0 ||
    savings.anomalyReport.anomalies.length > 0 ||
    savings.forecast.dailyAvgCost > 0;

  if (!hasAnyData) return null;

  return (
    <div>
      {/* Section header */}
      <div className="flex items-center gap-2 mb-3">
        <Zap className="w-4 h-4 text-amber-500" />
        <h2 className="text-sm font-semibold">{t("savings.title")}</h2>
        <button
          onClick={() => setCollapsed(!collapsed)}
          className="ml-auto text-xs text-muted-foreground hover:text-foreground flex items-center gap-0.5 transition-colors"
        >
          {collapsed ? (
            <>
              {t("savings.expand")} <ChevronDown className="w-3 h-3" />
            </>
          ) : (
            <>
              {t("savings.collapse")} <ChevronUp className="w-3 h-3" />
            </>
          )}
        </button>
      </div>

      {/* Cards grid */}
      {!collapsed && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <CacheInsightCard />
          <ModelInsightCard />
          <ForecastCard onNavigateToSettings={onNavigateToSettings} />
          <AnomalyCard />
        </div>
      )}
    </div>
  );
}
