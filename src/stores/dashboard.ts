import { create } from 'zustand';
import {
  type UsageSummary,
  type ModelUsage,
  type ProviderUsage,
  type TrendPoint,
  type SessionSummary,
  type BudgetConfig,
  type BudgetAlert,
  type DbStats,
  type CcSwitchStatus,
  type SavingsAnalysis,
  type SyncResult,
  type PeriodComparison,
  type CostAnomalyReport,
  type ProviderAttribution,
  type BudgetBurnRate,
  type CacheTrendPoint,
  getUsageSummary,
  getUsageByModel,
  getUsageByProvider,
  getUsageTrend,
  getRecentSessions,
  getBudgetConfig,
  checkBudgetAlert,
  getDbStats,
  getCcSwitchStatus,
  syncCcSwitch,
  getSavingsAnalysis,
  getPeriodComparison,
  getCostAnomalies,
  getCostAttribution,
  getBudgetBurnRate,
  getCacheTrend,
} from '@/lib/tauri';
import { PERIODS, GRANULARITIES } from '@/lib/constants';

interface DashboardState {
  // Period selection
  period: string;
  granularity: string;
  initialized: boolean;

  // Data
  summary: UsageSummary | null;
  modelUsage: ModelUsage[];
  providerUsage: ProviderUsage[];
  trend: TrendPoint[];
  sessions: SessionSummary[];
  budgetConfig: BudgetConfig | null;
  budgetAlert: BudgetAlert | null;
  dbStats: DbStats | null;
  ccSwitchStatus: CcSwitchStatus | null;
  savingsAnalysis: SavingsAnalysis | null;
  periodComparison: PeriodComparison | null;
  costAnomalyReport: CostAnomalyReport | null;
  costAttribution: ProviderAttribution[];
  budgetBurnRate: BudgetBurnRate | null;
  cacheTrend: CacheTrendPoint[];

  // UI state
  loading: boolean;
  syncing: boolean;
  error: string | null;

  // Actions
  setPeriod: (period: string) => void;
  setGranularity: (granularity: string) => void;
  initializePeriod: (defaultPeriod: string) => void;
  fetchDashboardData: () => Promise<void>;
  syncData: () => Promise<SyncResult | null>;
  refreshStatus: () => Promise<void>;
}

export const useDashboardStore = create<DashboardState>((set, get) => ({
  // Initial state
  period: PERIODS.WEEK,
  granularity: GRANULARITIES.DAILY,
  initialized: false,
  summary: null,
  modelUsage: [],
  providerUsage: [],
  trend: [],
  sessions: [],
  budgetConfig: null,
  budgetAlert: null,
  dbStats: null,
  ccSwitchStatus: null,
  savingsAnalysis: null,
  periodComparison: null,
  costAnomalyReport: null,
  costAttribution: [],
  budgetBurnRate: null,
  cacheTrend: [],
  loading: false,
  syncing: false,
  error: null,

  setPeriod: (period: string) => {
    set({ period });
    get().fetchDashboardData();
  },

  setGranularity: (granularity: string) => {
    set({ granularity });
    const { period } = get();
    Promise.all([
      getUsageTrend(granularity, period),
      getCacheTrend(granularity, period),
    ])
      .then(([trend, cacheTrend]) => set({ trend, cacheTrend }))
      .catch(console.error);
  },

  initializePeriod: (defaultPeriod: string) => {
    const { initialized } = get();
    if (!initialized) {
      set({ period: defaultPeriod, initialized: true });
    }
  },

  fetchDashboardData: async () => {
    set({ loading: true, error: null });
    const { period, granularity } = get();

    try {
      const [summary, modelUsage, providerUsage, trend, sessions, budgetConfig, budgetAlert, dbStats, ccSwitchStatus, savingsAnalysis, periodComparison, costAnomalyReport, costAttribution, budgetBurnRate, cacheTrend] =
        await Promise.all([
          getUsageSummary(period),
          getUsageByModel(period),
          getUsageByProvider(period),
          getUsageTrend(granularity, period),
          getRecentSessions(10),
          getBudgetConfig(),
          checkBudgetAlert(),
          getDbStats(),
          getCcSwitchStatus(),
          getSavingsAnalysis(period),
          getPeriodComparison(period),
          getCostAnomalies(period),
          getCostAttribution(period),
          getBudgetBurnRate(),
          getCacheTrend(granularity, period),
        ]);

      set({
        summary,
        modelUsage,
        providerUsage,
        trend,
        sessions,
        budgetConfig,
        budgetAlert,
        dbStats,
        ccSwitchStatus,
        savingsAnalysis,
        periodComparison,
        costAnomalyReport,
        costAttribution,
        budgetBurnRate,
        cacheTrend,
        loading: false,
      });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  syncData: async () => {
    set({ syncing: true });
    try {
      const result = await syncCcSwitch();
      await get().fetchDashboardData();
      set({ syncing: false });
      return result;
    } catch (err) {
      set({ error: String(err), syncing: false });
      return null;
    }
  },

  refreshStatus: async () => {
    try {
      const [ccSwitchStatus, budgetAlert] = await Promise.all([
        getCcSwitchStatus(),
        checkBudgetAlert(),
      ]);
      set({ ccSwitchStatus, budgetAlert });
    } catch (err) {
      console.error('Failed to refresh status:', err);
    }
  },
}));
