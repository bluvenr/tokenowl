import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  getUsageSummary,
  getUsageByProvider,
  getBudgetConfig,
  getCcSwitchStatus,
  getPeriodComparison,
  syncCcSwitch,
  quitApp,
  type UsageSummary,
  type ProviderUsage,
  type BudgetConfig,
  type CcSwitchStatus,
  type PeriodComparison,
} from '@/lib/tauri';
import { formatUsd, formatTokens, formatPercent } from '@/lib/format';
import { cn } from '@/lib/utils';
import { useAppStore } from '@/stores/app';
import {
  RefreshCw,
  ExternalLink,
  Settings,
  Zap,
  Wifi,
  WifiOff,
  TrendingUp,
  TrendingDown,
  Minus,
  Check,
  AlertTriangle,
  X,
  LogOut,
  Pin,
  PinOff,
  Minimize2,
} from 'lucide-react';
import { getCurrentWindow, getAllWindows } from '@tauri-apps/api/window';
import { emit } from '@tauri-apps/api/event';
import { LogicalSize } from '@tauri-apps/api/dpi';

const NORMAL_SIZE = new LogicalSize(320, 400);
const MINI_SIZE = new LogicalSize(220, 135);

export function TrayPopup() {
  const { t } = useTranslation();
  const loadSettings = useAppStore((s) => s.loadSettings);
  const [summary, setSummary] = useState<UsageSummary | null>(null);
  const [providers, setProviders] = useState<ProviderUsage[]>([]);
  const [budget, setBudget] = useState<BudgetConfig | null>(null);
  const [ccStatus, setCcStatus] = useState<CcSwitchStatus | null>(null);
  const [comparison, setComparison] = useState<PeriodComparison | null>(null);
  const [syncing, setSyncing] = useState(false);
  const [loading, setLoading] = useState(true);

  // Pin and mini mode state persisted to localStorage
  const [pinned, setPinned] = useState(() => localStorage.getItem('tray_pinned') === 'true');
  const [miniMode, setMiniMode] = useState(() => localStorage.getItem('tray_mini_mode') === 'true');
  const lastMouseDownRef = useRef(0);

  useEffect(() => {
    loadSettings();
    loadData();
    const interval = setInterval(loadData, 30000);
    return () => clearInterval(interval);
  }, []);

  // Persist pin state
  useEffect(() => {
    localStorage.setItem('tray_pinned', String(pinned));
  }, [pinned]);

  // Persist mini mode state
  useEffect(() => {
    localStorage.setItem('tray_mini_mode', String(miniMode));
  }, [miniMode]);

  // Handle mini mode window size and always-on-top
  useEffect(() => {
    const win = getCurrentWindow();
    if (miniMode) {
      win.setSize(MINI_SIZE);
      win.setAlwaysOnTop(true);
      setPinned(true); // force pin on in mini mode
    } else {
      win.setSize(NORMAL_SIZE);
      win.setAlwaysOnTop(false);
    }
  }, [miniMode]);

  // Reload settings and data when tray window gains focus; auto-hide on blur
  useEffect(() => {
    const win = getCurrentWindow();
    const unlisten = win.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        loadSettings();
        loadData();
      } else if (!pinned && !miniMode) {
        win.hide();
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [pinned, miniMode]);

  const loadData = async () => {
    try {
      const [s, p, b, c, comp] = await Promise.all([
        getUsageSummary('today'),
        getUsageByProvider('today'),
        getBudgetConfig(),
        getCcSwitchStatus(),
        getPeriodComparison('today'),
      ]);
      setSummary(s);
      setProviders(p.slice(0, 5));
      setBudget(b);
      setCcStatus(c);
      setComparison(comp);
    } catch (err) {
      console.error('Failed to load tray data:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSync = async () => {
    if (syncing) return;
    setSyncing(true);
    try {
      await syncCcSwitch();
      await loadData();
    } catch (err) {
      console.error('Sync failed:', err);
    } finally {
      setSyncing(false);
    }
  };

  const navigateTo = async (page: 'dashboard' | 'settings') => {
    try {
      await emit('navigate', page);
      const allWindows = await getAllWindows();
      const mainWindow = allWindows.find((w) => w.label === 'main');
      if (mainWindow) {
        await mainWindow.show();
        await mainWindow.setFocus();
      }
      const trayWindow = getCurrentWindow();
      await trayWindow.hide();
    } catch (err) {
      console.error(`Failed to navigate to ${page}:`, err);
    }
  };

  const toggleMini = () => setMiniMode((prev) => !prev);
  const togglePin = () => {
    if (miniMode) return; // can't unpin in mini mode
    setPinned((prev) => !prev);
  };

  const startDrag = async (e: React.MouseEvent) => {
    e.preventDefault();
    try {
      await getCurrentWindow().startDragging();
    } catch (err) {
      console.error('Drag failed:', err);
    }
  };

  // Mini mode: drag immediately, detect double-click by mouseDown timing
  const handleMiniMouseDown = async () => {
    const now = Date.now();
    if (now - lastMouseDownRef.current < 300) {
      // Double-click detected — exit mini mode
      lastMouseDownRef.current = 0;
      setMiniMode(false);
      return;
    }
    lastMouseDownRef.current = now;
    try {
      await getCurrentWindow().startDragging();
    } catch (err) {
      console.error('Drag failed:', err);
    }
  };

  const budgetProgress = budget?.daily_limit_usd
    ? Math.min(100, (summary?.total_cost_usd ?? 0) / budget.daily_limit_usd * 100)
    : null;

  // Determine budget alert level based on user-configured threshold
  const budgetLevel = (() => {
    if (budgetProgress === null || !budget) return 'normal' as const;
    const threshold = budget.alert_threshold_pct;
    if (budgetProgress >= threshold) return 'danger' as const;
    if (budgetProgress >= threshold * 0.75) return 'warning' as const;
    return 'normal' as const;
  })();

  const budgetColor = {
    danger: 'text-red-500',
    warning: 'text-orange-500',
    normal: 'text-muted-foreground',
  }[budgetLevel];

  const budgetBarColor = {
    danger: 'bg-red-500',
    warning: 'bg-orange-500',
    normal: 'bg-primary',
  }[budgetLevel];

  if (loading) {
    return (
      <div className="flex h-screen items-center justify-center bg-background">
        <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
      </div>
    );
  }

  // Mini mode view: entire area draggable, double-click to exit
  if (miniMode) {
    return (
      <div
        className="h-screen w-screen bg-background flex flex-col select-none overflow-hidden cursor-move"
        onMouseDown={handleMiniMouseDown}
      >
        <div className="flex-1 flex flex-col justify-center px-3 py-2">
          <div className="flex items-baseline justify-between mb-1.5">
            <span className="text-[10px] text-muted-foreground">{t('tray.todayCost')}</span>
            <span className={cn('text-lg font-bold leading-none', budgetLevel !== 'normal' && budgetColor)}>
              {formatUsd(summary?.total_cost_usd ?? 0)}
            </span>
          </div>
          <div className="grid grid-cols-2 gap-x-3 gap-y-1 text-[10px]">
            <div className="flex justify-between text-muted-foreground">
              <span>{t('attribution.input')}</span>
              <span className="font-medium text-foreground">{formatTokens(summary?.input_tokens ?? 0)}</span>
            </div>
            <div className="flex justify-between text-muted-foreground">
              <span>{t('attribution.output')}</span>
              <span className="font-medium text-foreground">{formatTokens(summary?.output_tokens ?? 0)}</span>
            </div>
            <div className="flex justify-between text-muted-foreground">
              <span>{t('attribution.cacheRead')}</span>
              <span className="font-medium text-foreground">{formatTokens(summary?.cache_tokens ?? 0)}</span>
            </div>
            <div className="flex justify-between text-muted-foreground">
              <span>{t('tray.totalTokens')}</span>
              <span className="font-medium text-foreground">{formatTokens(summary?.total_tokens ?? 0)}</span>
            </div>
          </div>
          {budgetProgress !== null && (
            <div className="mt-1.5">
              <div className="flex items-center justify-between text-[10px] mb-0.5">
                <span className="text-muted-foreground">{t('tray.dailyBudget')}</span>
                <span className={cn(budgetLevel !== 'normal' && budgetColor, budgetLevel !== 'normal' && 'font-medium')}>
                  {formatPercent(budgetProgress)}
                </span>
              </div>
              <div className="h-1 rounded-full bg-muted w-full">
                <div
                  className={cn('h-full rounded-full transition-all', budgetBarColor)}
                  style={{ width: `${budgetProgress}%` }}
                />
              </div>
            </div>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="h-screen w-screen bg-background flex flex-col overflow-hidden">
      {/* Header — draggable middle area for window dragging */}
      <div className="flex items-center border-b px-3 py-2">
        <div className="flex items-center gap-2 shrink-0">
          <h1 className="text-sm font-bold">TokenOwl</h1>
          {ccStatus?.proxy_running ? (
            <Wifi className="h-3 w-3 text-green-500" />
          ) : (
            <WifiOff className="h-3 w-3 text-muted-foreground" />
          )}
        </div>
        {/* Drag region */}
        <div
          className="flex-1 h-6 mx-1 cursor-move"
          onMouseDown={startDrag}
        />
        <div className="flex items-center gap-1 shrink-0">
          <button
            onClick={handleSync}
            disabled={syncing}
            className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-50"
            title={t('tray.syncData')}
          >
            <RefreshCw className={cn('h-3 w-3', syncing && 'animate-spin')} />
          </button>
          <button
            onClick={togglePin}
            className={cn('rounded p-1 hover:bg-muted', pinned ? 'text-primary' : 'text-muted-foreground hover:text-foreground')}
            title={pinned ? t('tray.unpin') : t('tray.pin')}
          >
            {pinned ? <Pin className="h-3 w-3" /> : <PinOff className="h-3 w-3" />}
          </button>
          <button
            onClick={toggleMini}
            className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
            title={t('tray.miniMode')}
          >
            <Minimize2 className="h-3 w-3" />
          </button>
          <button
            onClick={() => navigateTo('dashboard')}
            className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
            title={t('tray.openDashboard')}
          >
            <ExternalLink className="h-3 w-3" />
          </button>
        </div>
      </div>

      {/* Today's Summary */}
      <div className="border-b px-3 py-3">
        <div className="text-xs text-muted-foreground mb-1">{t('tray.todayCost')}</div>
        <div className="flex items-center gap-2">
          <div className={cn('text-2xl font-bold', budgetLevel !== 'normal' && budgetColor)}>
            {formatUsd(summary?.total_cost_usd ?? 0)}
          </div>
          {comparison?.cost_change_pct != null && (
            <span className={cn(
              'flex items-center gap-0.5 text-xs font-medium',
              comparison.cost_change_pct > 0
                ? 'text-red-500'
                : comparison.cost_change_pct < 0
                  ? 'text-green-500'
                  : 'text-muted-foreground'
            )}>
              {comparison.cost_change_pct > 0 ? (
                <TrendingUp className="h-3 w-3" />
              ) : comparison.cost_change_pct < 0 ? (
                <TrendingDown className="h-3 w-3" />
              ) : (
                <Minus className="h-3 w-3" />
              )}
              {comparison.cost_change_pct > 0 ? '+' : ''}{comparison.cost_change_pct.toFixed(1)}%
            </span>
          )}
        </div>
        <div className="flex gap-4 mt-2 text-xs text-muted-foreground">
          <span className="flex items-center gap-1">
            <Zap className="h-3 w-3" />
            {formatTokens(summary?.total_tokens ?? 0)}
          </span>
          <span>{t('tray.requests', { count: summary?.request_count ?? 0 })}</span>
        </div>
      </div>

      {/* Budget Progress */}
      {budgetProgress !== null && (
        <div className="border-b px-3 py-2">
          <div className="flex items-center justify-between text-xs mb-1">
            <span className="text-muted-foreground">{t('tray.dailyBudget')}</span>
            <span className={cn(budgetLevel !== 'normal' && budgetColor, budgetLevel !== 'normal' && 'font-medium')}>
              {formatPercent(budgetProgress)}
            </span>
          </div>
          <div className="h-2 rounded-full bg-muted">
            <div
              className={cn('h-full rounded-full transition-all', budgetBarColor)}
              style={{ width: `${budgetProgress}%` }}
            />
          </div>
          <div className={cn('text-xs mt-1', budgetLevel !== 'normal' ? budgetColor : 'text-muted-foreground')}>
            {formatUsd(summary?.total_cost_usd ?? 0)} / {budget?.daily_limit_usd != null ? formatUsd(budget.daily_limit_usd) : '—'}
          </div>
        </div>
      )}

      {/* Provider Breakdown */}
      <div className="flex-1 overflow-y-auto px-3 py-2">
        <div className="text-xs text-muted-foreground mb-2">{t('tray.providerBreakdown')}</div>
        {providers.length === 0 ? (
          <div className="text-center text-xs text-muted-foreground py-4">
            {t('tray.noData')}
          </div>
        ) : (
          <div className="space-y-2">
            {providers.map((provider) => {
              const failureRate = provider.failure_rate * 100;
              const HealthIcon = failureRate === 0
                ? Check
                : failureRate <= 5
                  ? AlertTriangle
                  : X;
              const healthColor = failureRate === 0
                ? 'text-green-500'
                : failureRate <= 5
                  ? 'text-orange-500'
                  : 'text-red-500';

              return (
                <div key={provider.provider_name} className="text-xs">
                  <div className="flex items-center justify-between mb-0.5">
                    <span className="truncate font-medium flex items-center gap-1" title={provider.provider_name}>
                      <HealthIcon className={cn('h-3 w-3', healthColor)} />
                      {provider.provider_name}
                    </span>
                    <span className="font-medium ml-2">
                      {formatUsd(provider.cost_usd)}
                    </span>
                  </div>
                  <div className="h-1 rounded-full bg-muted">
                    <div
                      className="h-full rounded-full bg-primary"
                      style={{ width: `${provider.percentage}%` }}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="border-t px-3 py-2 flex items-center justify-between">
        <span className="text-xs text-muted-foreground">
          {t('tray.totalRecords', { count: ccStatus?.total_records ?? 0 })}
        </span>
        <div className="flex items-center gap-2">
          <button
            onClick={() => navigateTo('settings')}
            className="text-xs text-primary hover:underline flex items-center gap-1"
          >
            <Settings className="h-3 w-3" />
            {t('tray.settings')}
          </button>
          <button
            onClick={quitApp}
            className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-red-500"
            title={t('tray.quit')}
          >
            <LogOut className="h-3 w-3" />
          </button>
        </div>
      </div>
    </div>
  );
}
