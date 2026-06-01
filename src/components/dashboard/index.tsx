import { useState, useEffect } from "react";
import { useAppStore } from "@/stores/appStore";
import { useTranslation } from "react-i18next";
import { AlertTriangle, ArrowRight, X } from "lucide-react";
import type { MissingModelPrice } from "@/lib/tauri";
import type { SettingsTab } from "@/App";
import {
  CostOverview,
  ToolBreakdown,
  TrendChart,
  ModelBreakdown,
  SessionList,
} from "./Dashboard";
import { Card, CardContent } from "@/components/ui/card";

/** Skeleton shimmer placeholder for cards during initial load */
function SkeletonCard({ className = "" }: { className?: string }) {
  return (
    <Card className={className}>
      <CardContent className="p-5 space-y-3">
        <div className="h-3 w-24 rounded bg-muted animate-pulse" />
        <div className="h-8 w-32 rounded bg-muted animate-pulse" />
        <div className="h-3 w-40 rounded bg-muted animate-pulse" />
        <div className="h-20 w-full rounded bg-muted animate-pulse" />
      </CardContent>
    </Card>
  );
}

/** Empty state illustration — a minimal chart icon with a message */
export function EmptyState({ message }: { message: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-10 gap-3">
      <svg
        width="64"
        height="64"
        viewBox="0 0 64 64"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        className="text-muted-foreground/40"
      >
        <rect x="8" y="36" width="10" height="20" rx="2" fill="currentColor" opacity="0.3" />
        <rect x="22" y="24" width="10" height="32" rx="2" fill="currentColor" opacity="0.5" />
        <rect x="36" y="16" width="10" height="40" rx="2" fill="currentColor" opacity="0.3" />
        <rect x="50" y="8" width="10" height="48" rx="2" fill="currentColor" opacity="0.5" />
        <path d="M4 60H62" stroke="currentColor" strokeWidth="2" strokeLinecap="round" opacity="0.3" />
      </svg>
      <p className="text-sm text-muted-foreground">{message}</p>
    </div>
  );
}

export function Dashboard({ onNavigateToSettings, missingModels = [] }: {
  onNavigateToSettings?: (tab: SettingsTab) => void;
  missingModels?: MissingModelPrice[];
}) {
  const { refresh, loading, error, summary } = useAppStore();
  const { t } = useTranslation();
  const [bannerDismissed, setBannerDismissed] = useState(false);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const isFirstLoad = loading && summary === null;

  return (
    <div className="p-6 pb-10">
      <div className="max-w-5xl mx-auto space-y-4">
        {/* Missing price warning banner */}
        {missingModels.length > 0 && !bannerDismissed && (
          <div className="rounded-lg border border-amber-500/40 bg-amber-500/10 px-4 py-3 flex items-start gap-3">
            <AlertTriangle className="w-4 h-4 text-amber-500 shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-amber-700 dark:text-amber-400">
                {t("dashboard.missing_prices_title", { count: missingModels.length })}
              </p>
              <p className="text-xs text-amber-600/80 dark:text-amber-400/70 mt-1 truncate">
                {missingModels.slice(0, 5).map((m) => m.model).join("、")}
                {missingModels.length > 5 && t("dashboard.missing_prices_more", { count: missingModels.length - 5 })}
              </p>
            </div>
            <button
              onClick={() => onNavigateToSettings?.("pricing")}
              className="shrink-0 inline-flex items-center gap-1 rounded-md bg-amber-500/20 hover:bg-amber-500/30 px-2.5 py-1.5 text-xs font-medium text-amber-700 dark:text-amber-400 transition-colors"
            >
              {t("dashboard.go_set_prices")}
              <ArrowRight className="w-3 h-3" />
            </button>
            <button
              onClick={() => setBannerDismissed(true)}
              className="shrink-0 text-amber-500/60 hover:text-amber-500 transition-colors"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        )}

        {error && (
          <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive flex items-center justify-between">
            <span>{error}</span>
            <button onClick={refresh} className="underline text-xs ml-3 shrink-0">
              {t("common.retry")}
            </button>
          </div>
        )}

        {isFirstLoad ? (
          /* Skeleton layout matching the dashboard structure */
          <>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <SkeletonCard />
              <SkeletonCard />
            </div>
            <SkeletonCard className="min-h-[240px]" />
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <SkeletonCard />
              <SkeletonCard />
            </div>
          </>
        ) : (
          <>
            {/* Row 1: Overview + Tool Breakdown */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <CostOverview />
              <ToolBreakdown />
            </div>

            {/* Row 2: Trend Chart (full width) */}
            <TrendChart />

            {/* Row 3: Model Breakdown + Session List */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <ModelBreakdown />
              <SessionList />
            </div>
          </>
        )}
      </div>
    </div>
  );
}
