import { useEffect, useState } from "react";
import { useAppStore } from "@/stores/appStore";
import { useTranslation } from "react-i18next";
import { RefreshCw, CheckCircle2, AlertCircle } from "lucide-react";
import type { SettingsTab } from "@/App";
import { getCcSwitchStatus, syncCcSwitch, type CcSwitchStatus } from "@/lib/tauri";
import {
  CostOverview,
  ToolBreakdown,
  TrendChart,
  ModelBreakdown,
  SessionList,
} from "./Dashboard";
import { SavingsInsights } from "./SavingsInsights";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

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
    <div className="flex flex-col items-center justify-center py-12 gap-3">
      <svg
        width="48"
        height="48"
        viewBox="0 0 64 64"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        className="text-muted-foreground/30"
      >
        <rect x="8" y="36" width="10" height="20" rx="2" fill="currentColor" opacity="0.3" />
        <rect x="22" y="24" width="10" height="32" rx="2" fill="currentColor" opacity="0.5" />
        <rect x="36" y="16" width="10" height="40" rx="2" fill="currentColor" opacity="0.3" />
        <rect x="50" y="8" width="10" height="48" rx="2" fill="currentColor" opacity="0.5" />
        <path d="M4 60H62" stroke="currentColor" strokeWidth="2" strokeLinecap="round" opacity="0.3" />
      </svg>
      <p className="text-sm text-muted-foreground/70">{message}</p>
    </div>
  );
}

/** CC Switch connection status badge */
function CcSwitchBadge({ status, onSync }: { status: CcSwitchStatus | null; onSync: () => void }) {
  const { t } = useTranslation();
  const [syncing, setSyncing] = useState(false);

  if (!status) return null;

  const handleSync = async () => {
    setSyncing(true);
    try {
      await onSync();
    } finally {
      setSyncing(false);
    }
  };

  return (
    <div className="flex items-center gap-3 px-4 py-2.5 rounded-lg border bg-card/50">
      <div className="flex items-center gap-2">
        {status.detected ? (
          <CheckCircle2 className="w-4 h-4 text-green-500" />
        ) : (
          <AlertCircle className="w-4 h-4 text-amber-500" />
        )}
        <span className="text-sm font-medium">CC Switch</span>
        {status.detected && (
          <span className="text-xs text-muted-foreground">
            {t("dashboard.records_count", { count: status.recordCount })}
          </span>
        )}
      </div>
      {status.detected && (
        <Button
          variant="ghost"
          size="sm"
          onClick={handleSync}
          disabled={syncing}
          className="h-7 px-2 gap-1.5 text-xs"
        >
          <RefreshCw className={`w-3 h-3 ${syncing ? "animate-spin" : ""}`} />
          {syncing ? t("dashboard.syncing") : t("dashboard.sync")}
        </Button>
      )}
    </div>
  );
}

export function Dashboard({ onNavigateToSettings }: {
  onNavigateToSettings?: (tab: SettingsTab) => void;
}) {
  const { refresh, loading, error, summary } = useAppStore();
  const { t } = useTranslation();
  const [ccSwitchStatus, setCcSwitchStatus] = useState<CcSwitchStatus | null>(null);

  useEffect(() => {
    refresh();
    loadCcSwitchStatus();
  }, [refresh]);

  async function loadCcSwitchStatus() {
    try {
      const status = await getCcSwitchStatus();
      setCcSwitchStatus(status);
    } catch {
      // Ignore errors
    }
  }

  async function handleSync() {
    await syncCcSwitch();
    await refresh();
    await loadCcSwitchStatus();
  }

  const isFirstLoad = loading && summary === null;

  return (
    <div className="p-6 pb-10">
      <div className="max-w-5xl mx-auto space-y-6">
        {/* Header with CC Switch status */}
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold tracking-tight">{t("dashboard.overview")}</h2>
            <p className="text-sm text-muted-foreground mt-1">{t("dashboard.subtitle")}</p>
          </div>
          <CcSwitchBadge status={ccSwitchStatus} onSync={handleSync} />
        </div>

        {error && (
          <div className="rounded-lg border border-destructive/30 bg-destructive/5 px-4 py-3 text-sm text-destructive flex items-center justify-between">
            <span>{error}</span>
            <button onClick={refresh} className="underline text-xs ml-3 shrink-0 font-medium">
              {t("common.retry")}
            </button>
          </div>
        )}

        {isFirstLoad ? (
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <SkeletonCard />
              <SkeletonCard />
            </div>
            <SkeletonCard className="min-h-[280px]" />
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <SkeletonCard />
              <SkeletonCard />
            </div>
          </div>
        ) : (
          <div className="space-y-6">
            {/* Savings Insights */}
            <SavingsInsights onNavigateToSettings={onNavigateToSettings} />

            {/* Main metrics */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <CostOverview />
              <ToolBreakdown />
            </div>

            {/* Trend */}
            <TrendChart />

            {/* Details */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <ModelBreakdown />
              <SessionList />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
