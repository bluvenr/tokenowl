/**
 * Application constants
 */

export const APP_NAME = 'TokenOwl';
export const APP_VERSION = '1.0.0';

// Time periods for data queries
export const PERIODS = {
  TODAY: 'today',
  WEEK: 'week',
  MONTH: 'month',
  ALL: 'all',
} as const;

export const PERIOD_LABELS: Record<string, string> = {
  today: '今日',
  week: '本周',
  month: '本月',
  all: '全部',
};

// Trend granularity options
export const GRANULARITIES = {
  HOURLY: 'hourly',
  DAILY: 'daily',
  WEEKLY: 'weekly',
} as const;

export const GRANULARITY_LABELS: Record<string, string> = {
  hourly: '按小时',
  daily: '按天',
  weekly: '按周',
};

// Theme options
export const THEMES = {
  LIGHT: 'light',
  DARK: 'dark',
  SYSTEM: 'system',
} as const;

// Language options
export const LANGUAGES = {
  AUTO: 'auto',
  ZH_CN: 'zh-CN',
  EN: 'en',
} as const;

export const LANGUAGE_LABELS: Record<string, string> = {
  auto: '自动',
  'zh-CN': '简体中文',
  en: 'English',
};

// Default settings
export const DEFAULT_SETTINGS = {
  language: 'auto',
  download_source: 'auto',
  auto_start: false,
  theme: 'system',
  tray_display: 'cost',
  telemetry_enabled: false,
  crash_log_enabled: true,
  anomaly_threshold: 2.5,
  forecast_method: 'linear',
  data_retention_days: 90,
  daily_digest_enabled: false,
  daily_digest_time: '20:00',
  weekly_digest_enabled: false,
  update_check_interval_hours: 4,
  price_sync_interval_hours: 12,
  default_period: 'week',
};

// Chart colors
export const CHART_COLORS = [
  'hsl(220, 70%, 50%)', // blue
  'hsl(160, 70%, 45%)', // green
  'hsl(35, 90%, 55%)',  // orange
  'hsl(280, 70%, 55%)', // purple
  'hsl(350, 80%, 55%)', // red
  'hsl(190, 80%, 45%)', // cyan
  'hsl(45, 90%, 50%)',  // yellow
  'hsl(320, 70%, 50%)', // pink
];

// GitHub links
export const GITHUB_REPO = 'https://github.com/bluvenr/tokenowl';
export const GITHUB_ISSUES = `${GITHUB_REPO}/issues`;
