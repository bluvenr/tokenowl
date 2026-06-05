import { useEffect, useState, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import {
  getBudgetConfig,
  updateBudgetConfig,
  type BudgetConfig as BudgetConfigType,
} from '@/lib/tauri';

export function BudgetAlertTab() {
  const { t } = useTranslation();
  const [config, setConfig] = useState<BudgetConfigType | null>(null);
  const [initialConfig, setInitialConfig] = useState<BudgetConfigType | null>(null);
  const [loading, setLoading] = useState(true);
  const [saved, setSaved] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const c = await getBudgetConfig();
      setConfig(c);
      setInitialConfig(c);
    } catch (err) {
      console.error('Failed to load budget config:', err);
    } finally {
      setLoading(false);
    }
  };

  // Auto-save with debounce
  useEffect(() => {
    if (!config || !initialConfig || JSON.stringify(config) === JSON.stringify(initialConfig)) return;
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(async () => {
      try {
        await updateBudgetConfig(config);
        setInitialConfig(config);
        setSaved(true);
        setTimeout(() => setSaved(false), 1500);
      } catch (err) {
        console.error('Failed to auto-save budget config:', err);
      }
    }, 500);
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [config, initialConfig]);

  if (loading || !config) {
    return (
      <div className="flex h-32 items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="max-w-2xl space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold mb-1">{t('budget.title')}</h2>
          <p className="text-sm text-muted-foreground">
            {t('budget.description')}
          </p>
        </div>
        {saved && (
          <span className="text-sm text-green-600 transition-opacity">{t('settings.saved')}</span>
        )}
      </div>

      {/* Budget Limits */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('budget.limits')}</h3>
        <div className="space-y-4">
          <label className="block">
            <span className="text-sm text-muted-foreground">{t('budget.daily')}</span>
            <input
              type="number"
              min="0"
              step="0.1"
              value={config.daily_limit_usd ?? ''}
              onChange={(e) =>
                setConfig({
                  ...config,
                  daily_limit_usd: e.target.value ? parseFloat(e.target.value) : null,
                })
              }
              placeholder={t('budget.noLimit')}
              className="mt-1 block w-full rounded-md border bg-background px-3 py-2 text-sm"
            />
          </label>
          <label className="block">
            <span className="text-sm text-muted-foreground">{t('budget.weekly')}</span>
            <input
              type="number"
              min="0"
              step="0.1"
              value={config.weekly_limit_usd ?? ''}
              onChange={(e) =>
                setConfig({
                  ...config,
                  weekly_limit_usd: e.target.value ? parseFloat(e.target.value) : null,
                })
              }
              placeholder={t('budget.noLimit')}
              className="mt-1 block w-full rounded-md border bg-background px-3 py-2 text-sm"
            />
          </label>
          <label className="block">
            <span className="text-sm text-muted-foreground">{t('budget.monthly')}</span>
            <input
              type="number"
              min="0"
              step="0.1"
              value={config.monthly_limit_usd ?? ''}
              onChange={(e) =>
                setConfig({
                  ...config,
                  monthly_limit_usd: e.target.value ? parseFloat(e.target.value) : null,
                })
              }
              placeholder={t('budget.noLimit')}
              className="mt-1 block w-full rounded-md border bg-background px-3 py-2 text-sm"
            />
          </label>
        </div>
      </div>

      {/* Alert Settings */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('budget.alertSettings')}</h3>
        <label className="block mb-4">
          <span className="text-sm text-muted-foreground">{t('budget.threshold')}</span>
          <div className="flex items-center gap-3 mt-1">
            <input
              type="range"
              min="50"
              max="100"
              step="5"
              value={config.alert_threshold_pct}
              onChange={(e) =>
                setConfig({
                  ...config,
                  alert_threshold_pct: parseInt(e.target.value),
                })
              }
              className="flex-1"
            />
            <span className="w-12 text-center font-mono">
              {config.alert_threshold_pct}%
            </span>
          </div>
        </label>
        <p className="text-xs text-muted-foreground mb-4">
          {t('budget.thresholdDesc', { value: config.alert_threshold_pct })}
        </p>

        <div className="space-y-3">
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={config.alert_icon_color}
              onChange={(e) =>
                setConfig({ ...config, alert_icon_color: e.target.checked })
              }
              className="rounded"
            />
            <span className="text-sm">{t('budget.iconColor')}</span>
          </label>
          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={config.alert_system_notify}
              onChange={(e) =>
                setConfig({ ...config, alert_system_notify: e.target.checked })
              }
              className="rounded"
            />
            <span className="text-sm">{t('budget.systemNotify')}</span>
          </label>
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={config.alert_dashboard_banner}
              onChange={(e) =>
                setConfig({ ...config, alert_dashboard_banner: e.target.checked })
              }
              className="rounded"
            />
            <span className="text-sm">{t('budget.dashboardBanner')}</span>
          </label>
        </div>
      </div>
    </div>
  );
}
