import { create } from "zustand";
import {
  getUsageSummary,
  getUsageByModel,
  getUsageTrend,
  getRecentSessions,
  getSavingsAnalysis,
  onSyncRequested,
  type UsageSummary,
  type ModelUsage,
  type TrendPoint,
  type SessionSummary,
  type SavingsAnalysis,
} from "@/lib/tauri";

interface AppState {
  // Period selection
  period: "today" | "week" | "month";
  setPeriod: (p: "today" | "week" | "month") => void;

  // Trend granularity
  trendGranularity: "hourly" | "daily" | "weekly";
  setTrendGranularity: (g: "hourly" | "daily" | "weekly") => void;

  // Usage data
  summary: UsageSummary | null;
  byModel: ModelUsage[];
  trend: TrendPoint[];
  sessions: SessionSummary[];

  // Savings analysis data
  savings: SavingsAnalysis | null;

  // Loading state
  loading: boolean;
  error: string | null;

  // Refresh all data
  refresh: () => Promise<void>;

  // Refresh only trend data (when granularity changes)
  refreshTrend: () => Promise<void>;

  // Register event listeners (call once on app mount)
  initEventListeners: () => Promise<(() => void) | undefined>;
  listenersRegistered: boolean;
}

export const useAppStore = create<AppState>((set, get) => ({
  period: "today",
  setPeriod: (p) => {
    set({ period: p });
    get().refresh();
  },

  trendGranularity: "daily",
  setTrendGranularity: (g) => {
    set({ trendGranularity: g });
    get().refreshTrend();
  },

  summary: null,
  byModel: [],
  trend: [],
  sessions: [],
  savings: null,

  loading: false,
  error: null,
  listenersRegistered: false,

  refresh: async () => {
    const { period, trendGranularity } = get();
    set({ loading: true, error: null });
    try {
      const [summary, byModel, trend, sessions, savings] = await Promise.all([
        getUsageSummary(period),
        getUsageByModel(period),
        getUsageTrend(trendGranularity, period),
        getRecentSessions(20),
        getSavingsAnalysis(period),
      ]);
      set({ summary, byModel, trend, sessions, savings, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  refreshTrend: async () => {
    const { period, trendGranularity } = get();
    try {
      const trend = await getUsageTrend(trendGranularity, period);
      set({ trend });
    } catch (e) {
      console.error("Failed to refresh trend:", e);
    }
  },

  initEventListeners: async () => {
    const { listenersRegistered } = get();
    if (listenersRegistered) return undefined;

    set({ listenersRegistered: true });

    // Listen for sync requests from tray menu
    const unlistenSync = await onSyncRequested(() => {
      get().refresh();
    });

    // Return cleanup function
    return () => {
      unlistenSync();
      set({ listenersRegistered: false });
    };
  },
}));
