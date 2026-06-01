import { create } from "zustand";
import {
  getUsageSummary,
  getUsageBySource,
  getUsageByModel,
  getUsageTrend,
  getRecentSessions,
  onDataChanged,
  onRescanRequested,
  type UsageSummary,
  type SourceUsage,
  type ModelUsage,
  type TrendPoint,
  type SessionSummary,
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
  bySource: SourceUsage[];
  byModel: ModelUsage[];
  trend: TrendPoint[];
  sessions: SessionSummary[];

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
  bySource: [],
  byModel: [],
  trend: [],
  sessions: [],

  loading: false,
  error: null,
  listenersRegistered: false,

  refresh: async () => {
    const { period, trendGranularity } = get();
    set({ loading: true, error: null });
    try {
      const [summary, bySource, byModel, trend, sessions] = await Promise.all([
        getUsageSummary(period),
        getUsageBySource(period),
        getUsageByModel(period),
        getUsageTrend(trendGranularity, period),
        getRecentSessions(20),
      ]);
      set({ summary, bySource, byModel, trend, sessions, loading: false });
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

    // Listen for file watcher data changes (real-time incremental updates)
    const unlistenData = await onDataChanged(() => {
      get().refresh();
    });

    // Listen for rescan requests from tray menu
    const unlistenRescan = await onRescanRequested(() => {
      get().refresh();
    });

    // Return cleanup function
    return () => {
      unlistenData();
      unlistenRescan();
      set({ listenersRegistered: false });
    };
  },
}));
