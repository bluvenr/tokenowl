import { useEffect } from "react";
import { useAppStore } from "@/stores/appStore";
import { useTranslation } from "react-i18next";
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

export function Dashboard() {
  const { refresh, loading, error, summary } = useAppStore();
  const { t } = useTranslation();

  useEffect(() => {
    refresh();
  }, [refresh]);

  const isFirstLoad = loading && summary === null;

  return (
    <div className="p-6 pb-10">
      <div className="max-w-5xl mx-auto space-y-4">
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
