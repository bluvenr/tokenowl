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

export interface SourceUsage {
  source: string;
  displayName: string;
  costUsd: number;
  totalTokens: number;
  percentage: number;
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
  priceSyncIntervalHours: number;
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

export interface SourceConfig {
  source: string;
  enabled: boolean;
  customPath: string | null;
  available: boolean;
  status: string;
}

export interface SourceStatus {
  source: string;
  displayName: string;
  available: boolean;
  enabled: boolean;
  recordCount: number;
  lastError: string | null;
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

// ─── Usage API ──────────────────────────────────────────────────────

export const getUsageSummary = (period: string) =>
  invoke<UsageSummary>("get_usage_summary", { period });

export const getUsageBySource = (period: string) =>
  invoke<SourceUsage[]>("get_usage_by_source", { period });

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

export const getSourceConfigs = () =>
  invoke<SourceConfig[]>("get_source_configs");

export const updateSourceConfig = (
  source: string,
  enabled: boolean,
  customPath: string | null
) => invoke<void>("update_source_config", { source, enabled, customPath });

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

export const recalculateCosts = (modelId: string) =>
  invoke<number>("recalculate_costs", { modelId });

export const countModelRecords = (modelId: string) =>
  invoke<number>("count_model_records", { modelId });

// ─── Export API ─────────────────────────────────────────────────────

export const exportUsageCsv = (period: string) =>
  invoke<string>("export_usage_csv", { period });

export const exportUsageJson = (period: string) =>
  invoke<string>("export_usage_json", { period });

// ─── Scan API ───────────────────────────────────────────────────────

export const rescan = () =>
  invoke<number>("rescan");

export const getSourceStatus = () =>
  invoke<SourceStatus[]>("get_source_status");

export interface MissingModelPrice {
  model: string;
  source: string;
}

export const getModelsMissingPrices = () =>
  invoke<MissingModelPrice[]>("get_models_without_prices");

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

export const rebuildTrayMenu = (openText: string, rescanText: string, quitText: string) =>
  invoke<void>("rebuild_tray_menu", { openText, rescanText, quitText });

// ─── Crash Log API ────────────────────────────────────────────────────

export const getCrashLogs = () =>
  invoke<CrashEntry[]>("get_crash_logs");

export const clearCrashLogs = () =>
  invoke<number>("clear_crash_logs");

export const getCrashIssueUrl = (id: string) =>
  invoke<string>("get_crash_issue_url", { id });

// ─── Events ─────────────────────────────────────────────────────────

/** Listen for data changes from the file watcher (real-time updates) */
export const onDataChanged = (callback: () => void) =>
  listen("tokenowl:data-changed", () => callback());

/** Listen for rescan requests from tray menu */
export const onRescanRequested = (callback: () => void) =>
  listen("tokenowl:rescan-requested", () => callback());
