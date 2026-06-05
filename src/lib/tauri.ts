import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// ─── Types ──────────────────────────────────────────────────────────

export interface UsageSummary {
  totalCostUsd: number;
  totalTokens: number;
  inputTokens: number;
  outputTokens: number;
  reasoningTokens: number;
  sessionCount: number;
}

export interface ModelUsage {
  model: string;
  source: string;
  costUsd: number;
  totalTokens: number;
  inputTokens: number;
  outputTokens: number;
  reasoningTokens: number;
}

export interface TrendPoint {
  date: string;
  costUsd: number;
  totalTokens: number;
}

export interface SessionSummary {
  sessionId: string;
  source: string;
  model: string;
  costUsd: number;
  totalTokens: number;
  timestamp: string;
  projectPath: string | null;
}

export interface BudgetConfig {
  dailyLimitUsd: number | null;
  weeklyLimitUsd: number | null;
  monthlyLimitUsd: number | null;
  alertThresholdPct: number;
  alertIconColor: boolean;
  alertSystemNotify: boolean;
}

export interface BudgetAlert {
  triggered: boolean;
  message: string;
  currentCostUsd: number;
  limitUsd: number;
  percentage: number;
  period: string;
}

export interface AppSettings {
  language: string;
  downloadSource: string;
  autoStart: boolean;
  theme: string;
  trayDisplay: string;
  telemetryEnabled: boolean;
  crashLogEnabled: boolean;
  updateCheckIntervalHours: number;
}

export interface ModelPricing {
  modelId: string;
  displayName: string;
  source: string;
  inputPerMillion: number;
  outputPerMillion: number;
  cacheWritePerMillion: number | null;
  cacheReadPerMillion: number | null;
  reasoningPerMillion?: number | null;
  priceSource: string;
  hasDefault?: boolean;
  createdAt?: string | null;
}

export interface DbStats {
  recordCount: number;
  sourceCount: number;
  sessionCount: number;
  dbSizeBytes: number;
}

export interface UpdateInfo {
  currentVersion: string;
  newVersion: string;
  notes: string;
  downloadUrl: string;
}

export interface CrashEntry {
  id: string;
  timestamp: string;
  errorType: string;
  message: string;
  stackTrace: string | null;
  appVersion: string;
  osInfo: string;
  context: Record<string, unknown>;
}

// ─── CC Switch Types ────────────────────────────────────────────────

export interface CcSwitchInfo {
  detected: boolean;
  dbPath: string;
  dbExists: boolean;
  recordCount: number;
  lastModified: string | null;
}

export interface CcSwitchStatus {
  detected: boolean;
  dbPath: string;
  dbExists: boolean;
  recordCount: number;
  lastSyncedId: number;
  lastSyncAt: string | null;
}

export interface SyncResult {
  newRecords: number;
  totalRecords: number;
  durationMs: number;
}

// ─── Usage API ──────────────────────────────────────────────────────

export const getUsageSummary = (period: string) =>
  invoke<UsageSummary>("get_usage_summary", { period });

export const getUsageByModel = (period: string) =>
  invoke<ModelUsage[]>("get_usage_by_model", { period });

export const getUsageTrend = (granularity: string, period: string) =>
  invoke<TrendPoint[]>("get_usage_trend", { granularity, period });

export const getRecentSessions = (limit: number = 20) =>
  invoke<SessionSummary[]>("get_recent_sessions", { limit });

// ─── Budget API ─────────────────────────────────────────────────────

export const getBudgetConfig = () =>
  invoke<BudgetConfig>("get_budget_config");

export const updateBudgetConfig = (config: BudgetConfig) =>
  invoke<void>("update_budget_config", { config });

export const checkBudgetAlert = () =>
  invoke<BudgetAlert | null>("check_budget_alert");

// ─── Settings API ───────────────────────────────────────────────────

export const getSettings = () =>
  invoke<AppSettings>("get_settings");

export const updateSettings = (settings: AppSettings) =>
  invoke<void>("update_settings", { settings });

// ─── Pricing API ────────────────────────────────────────────────────

export const getAllPrices = () =>
  invoke<ModelPricing[]>("get_all_prices");

export const getCustomPrices = () =>
  invoke<ModelPricing[]>("get_custom_prices");

export const updateCustomPrice = (price: ModelPricing) =>
  invoke<void>("update_custom_price", { price });

export const resetCustomPrice = (modelId: string) =>
  invoke<void>("reset_custom_price", { modelId });

export const deleteCustomPrice = (modelId: string) =>
  invoke<void>("delete_custom_price", { modelId });

export const countModelRecords = (modelId: string) =>
  invoke<number>("count_model_records", { modelId });

// ─── Export API ─────────────────────────────────────────────────────

export const exportUsageCsv = (period: string) =>
  invoke<string>("export_usage_csv", { period });

export const exportUsageJson = (period: string) =>
  invoke<string>("export_usage_json", { period });

// ─── CC Switch API ──────────────────────────────────────────────────

export const getCcSwitchStatus = () =>
  invoke<CcSwitchStatus>("get_ccswitch_status");

export const syncCcSwitch = () =>
  invoke<SyncResult>("sync_ccswitch");

export const getCcSwitchDbPath = () =>
  invoke<string>("get_ccswitch_db_path");

// ─── Notification API ───────────────────────────────────────────────

export const sendNotification = (title: string, body: string) =>
  invoke<void>("send_notification", { title, body });

// ─── Database Stats API ─────────────────────────────────────────────

export const getDbStats = () =>
  invoke<DbStats>("get_db_stats");

// ─── Remote Services API ──────────────────────────────────────────────

export const getAppVersion = () =>
  invoke<string>("get_app_version");

export const checkForUpdate = () =>
  invoke<UpdateInfo | null>("check_for_update");

// ─── Tray API ─────────────────────────────────────────────────────────

export const rebuildTrayMenu = (openText: string, syncText: string, quitText: string) =>
  invoke<void>("rebuild_tray_menu", { openText, syncText, quitText });

// ─── Crash Log API ────────────────────────────────────────────────────

export const getCrashLogs = () =>
  invoke<CrashEntry[]>("get_crash_logs");

export const clearCrashLogs = () =>
  invoke<number>("clear_crash_logs");

export const getCrashIssueUrl = (id: string) =>
  invoke<string>("get_crash_issue_url", { id });

// ─── Savings Analysis API ────────────────────────────────────────────

export interface CacheEfficiency {
  source: string;
  displayName: string;
  totalCacheRead: number;
  totalCacheCreation: number;
  totalInput: number;
  hitRate: number | null;
  cacheCostSavings: number;
}

export interface ModelUsageInsight {
  model: string;
  source: string;
  costUsd: number;
  totalTokens: number;
  sessionCount: number;
  costPerSession: number;
  costPerMillionTokens: number;
  costSharePct: number;
}

export interface ModelAnalysis {
  insights: ModelUsageInsight[];
  topCostModel: ModelUsageInsight | null;
  topCostSharePct: number;
  concentrationIndex: number;
}

export interface CostForecast {
  dailyAvgCost: number;
  projectedMonthlyCost: number;
  daysRemaining: number;
  daysElapsed: number;
  monthlyLimit: number | null;
  projectedOverBudget: boolean;
  budgetExhaustionDays: number | null;
  weekOverWeekChangePct: number | null;
}

export interface CostAnomaly {
  date: string;
  costUsd: number;
  dailyAvg: number;
  deviationFactor: number;
  source: string | null;
}

export interface AnomalyReport {
  anomalies: CostAnomaly[];
  dailyAvgCost: number;
  dailyStdDev: number;
  thresholdFactor: number;
}

export interface SavingsAnalysis {
  cacheEfficiency: CacheEfficiency[];
  modelAnalysis: ModelAnalysis;
  forecast: CostForecast;
  anomalyReport: AnomalyReport;
}

export const getSavingsAnalysis = (period: string) =>
  invoke<SavingsAnalysis>("get_savings_analysis", { period });

// ─── Events ─────────────────────────────────────────────────────────

/** Listen for sync requests from tray menu */
export const onSyncRequested = (callback: () => void) =>
  listen("tokenowl:sync-requested", () => callback());
