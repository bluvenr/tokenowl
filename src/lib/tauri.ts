import { invoke } from '@tauri-apps/api/core';

// ============ Usage Commands ============

export interface UsageSummary {
  total_cost_usd: number;
  total_tokens: number;
  input_tokens: number;
  output_tokens: number;
  cache_tokens: number;
  session_count: number;
  request_count: number;
}

export interface ModelUsage {
  model: string;
  cost_usd: number;
  total_tokens: number;
  request_count: number;
  percentage: number;
}

export interface ProviderUsage {
  provider_name: string;
  cost_usd: number;
  total_tokens: number;
  request_count: number;
  avg_latency_ms: number;
  failure_rate: number;
  percentage: number;
}

export interface TrendPoint {
  timestamp: string;
  cost_usd: number;
  total_tokens: number;
}

export interface SessionSummary {
  id: string;
  timestamp: string;
  model: string;
  provider_name: string | null;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  cost_usd: number | null;
  status_code: number | null;
  response_time_ms: number | null;
}

export const getUsageSummary = (period: string) =>
  invoke<UsageSummary>('get_usage_summary', { period });

export const getUsageByModel = (period: string) =>
  invoke<ModelUsage[]>('get_usage_by_model', { period });

export const getUsageByProvider = (period: string) =>
  invoke<ProviderUsage[]>('get_usage_by_provider', { period });

export const getUsageTrend = (granularity: string, period: string) =>
  invoke<TrendPoint[]>('get_usage_trend', { granularity, period });

export const getRecentSessions = (limit?: number) =>
  invoke<SessionSummary[]>('get_recent_sessions', { limit });

export interface PeriodComparison {
  cost_change_pct: number | null;
  tokens_change_pct: number | null;
  requests_change_pct: number | null;
  sessions_change_pct: number | null;
}

export const getPeriodComparison = (period: string) =>
  invoke<PeriodComparison>('get_period_comparison', { period });

export interface CostAnomaly {
  date: string;
  cost_usd: number;
  avg_cost: number;
  deviation: number;
  top_provider: string | null;
  top_model: string | null;
}

export interface CostAnomalyReport {
  anomaly_days: CostAnomaly[];
  total_days: number;
  avg_daily_cost: number;
  stddev: number;
  threshold: number;
}

export const getCostAnomalies = (period: string) =>
  invoke<CostAnomalyReport>('get_cost_anomalies', { period });

export interface TokenBreakdown {
  token_type: string;
  cost_usd: number;
  tokens: number;
  percentage: number;
}

export interface ModelAttribution {
  model: string;
  cost_usd: number;
  total_tokens: number;
  token_breakdown: TokenBreakdown[];
  percentage: number;
}

export interface ProviderAttribution {
  provider_name: string;
  cost_usd: number;
  models: ModelAttribution[];
  percentage: number;
}

export const getCostAttribution = (period: string) =>
  invoke<ProviderAttribution[]>('get_cost_attribution', { period });

export interface BudgetBurnRate {
  daily_rate: number;
  daily_spend: number | null;
  daily_limit: number | null;
  daily_days_remaining: number | null;
  weekly_spend: number | null;
  weekly_limit: number | null;
  weekly_days_remaining: number | null;
  monthly_spend: number | null;
  monthly_limit: number | null;
  monthly_days_remaining: number | null;
}

export const getBudgetBurnRate = () =>
  invoke<BudgetBurnRate>('get_budget_burn_rate');

export interface CacheTrendPoint {
  timestamp: string;
  cache_hit_rate: number;
  cache_tokens: number;
  total_tokens: number;
}

export const getCacheTrend = (granularity: string, period: string) =>
  invoke<CacheTrendPoint[]>('get_cache_trend', { granularity, period });

// ============ Budget Commands ============

export interface BudgetConfig {
  daily_limit_usd: number | null;
  weekly_limit_usd: number | null;
  monthly_limit_usd: number | null;
  alert_threshold_pct: number;
  alert_icon_color: boolean;
  alert_system_notify: boolean;
  alert_dashboard_banner: boolean;
}

export interface BudgetAlert {
  triggered: boolean;
  period: string;
  current_cost: number;
  limit: number;
  percentage: number;
  message: string;
}

export interface DbStats {
  total_records: number;
  total_cost_usd: number;
  date_range_start: string | null;
  date_range_end: string | null;
  db_size_bytes: number;
}

export const getBudgetConfig = () =>
  invoke<BudgetConfig>('get_budget_config');

export const updateBudgetConfig = (config: BudgetConfig) =>
  invoke<void>('update_budget_config', { config });

export const checkBudgetAlert = () =>
  invoke<BudgetAlert | null>('check_budget_alert');

export const sendNotification = (title: string, body: string) =>
  invoke<void>('send_notification', { title, body });

export const getDbStats = () =>
  invoke<DbStats>('get_db_stats');

// ============ Export Commands ============

export const exportUsageCsv = (period: string) =>
  invoke<void>('export_usage_csv', { period });

export const exportUsageJson = (period: string) =>
  invoke<void>('export_usage_json', { period });

// ============ Settings Commands ============

export interface AppSettings {
  language: string;
  download_source: string;
  auto_start: boolean;
  theme: string;
  tray_display: string;
  telemetry_enabled: boolean;
  crash_log_enabled: boolean;
  anomaly_threshold: number;
  forecast_method: string;
  data_retention_days: number;
  daily_digest_enabled: boolean;
  daily_digest_time: string;
  weekly_digest_enabled: boolean;
  update_check_interval_hours: number;
  price_sync_interval_hours: number;
  default_period: string;
}

export interface ModelPricing {
  model_id: string;
  display_name: string;
  input_per_million: number;
  output_per_million: number;
  cache_write_per_million: number | null;
  cache_read_per_million: number | null;
  source: 'builtin' | 'remote' | 'custom';
}

export interface CustomPrice {
  model_id: string;
  input_per_million: number;
  output_per_million: number;
  cache_write_per_million: number | null;
  cache_read_per_million: number | null;
}

export const getSettings = () =>
  invoke<AppSettings>('get_settings');

export const updateSettings = (settings: AppSettings) =>
  invoke<void>('update_settings', { settings });

export const setAutostart = (enabled: boolean) =>
  invoke<void>('set_autostart', { enabled });

export const isAutostartEnabled = () =>
  invoke<boolean>('is_autostart_enabled');

export const getCustomPrices = () =>
  invoke<CustomPrice[]>('get_custom_prices');

export const updateCustomPrice = (price: CustomPrice) =>
  invoke<void>('update_custom_price', { price });

export const deleteCustomPrice = (modelId: string) =>
  invoke<void>('delete_custom_price', { modelId });

export const resetCustomPrice = (modelId: string) =>
  invoke<void>('reset_custom_price', { modelId });

export const getAllPrices = () =>
  invoke<ModelPricing[]>('get_all_prices');

export const countModelRecords = () =>
  invoke<[string, number][]>('count_model_records');

export const getModelsWithoutPrices = () =>
  invoke<string[]>('get_models_without_prices');

export const quitApp = () => invoke<void>('quit_app');

export const showTrayPopup = () => invoke<void>('show_tray_popup');

export const updateTrayMenu = (language: string) =>
  invoke<void>('update_tray_menu', { language });

// ============ CC Switch Commands ============

export interface CcSwitchStatus {
  detected: boolean;
  db_path: string | null;
  custom_db_path: string | null;
  proxy_running: boolean;
  total_records: number;
  last_sync_time: string | null;
  sync_interval_secs: number;
  provider_count: number;
  success_rate: number;
}

export interface SyncResult {
  new_records: number;
  skipped_duplicates: number;
  errors: number;
  sync_duration_ms: number;
}

export const getCcSwitchStatus = () =>
  invoke<CcSwitchStatus>('get_ccswitch_status');

export const syncCcSwitch = () =>
  invoke<SyncResult>('sync_ccswitch');

export const getCcSwitchDbPath = () =>
  invoke<string | null>('get_ccswitch_db_path');

export const ccswitchUpdateSyncConfig = (syncIntervalSecs: number) =>
  invoke<void>('ccswitch_update_sync_config', { syncIntervalSecs });

export const ccswitchSetDbPath = (path: string | null) =>
  invoke<void>('ccswitch_set_db_path', { path });

// ============ Savings Commands ============

export interface SavingsAnalysis {
  cache_hit_rate: number;
  cache_savings_usd: number;
  herfindahl_index: number;
  model_concentration: string;
  monthly_forecast_usd: number;
  forecast_confidence: number;
  recommendations: string[];
}

export const getSavingsAnalysis = (period: string) =>
  invoke<SavingsAnalysis>('get_savings_analysis', { period });

// ============ Remote Commands ============

export interface AppVersion {
  current: string;
  latest: string | null;
  update_available: boolean;
  release_url: string | null;
  changelog: string | null;
}

export interface CrashLogEntry {
  id: string;
  timestamp: string;
  error_type: string;
  message: string;
  backtrace: string | null;
}

export const getAppVersion = () =>
  invoke<AppVersion>('get_app_version');

export const checkForUpdate = () =>
  invoke<AppVersion>('check_for_update');

export const fetchRemoteConfig = () =>
  invoke<Record<string, unknown>>('fetch_remote_config');

export const getCrashLogs = () =>
  invoke<CrashLogEntry[]>('get_crash_logs');

export const deleteCrashLog = (id: string) =>
  invoke<void>('delete_crash_log', { id });

export const clearCrashLogs = () =>
  invoke<void>('clear_crash_logs');

export const getCrashIssueUrl = () =>
  invoke<string>('get_crash_issue_url');
