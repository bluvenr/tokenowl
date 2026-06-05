import { useEffect, useState, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useAppStore } from '@/stores/app';

export function AnalyticsTab() {
  const { t } = useTranslation();
  const settings = useAppStore((s) => s.settings);
  const saveSettings = useAppStore((s) => s.saveSettings);
  const [localSettings, setLocalSettings] = useState(settings);
  const [saved, setSaved] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    setLocalSettings(settings);
  }, [settings]);

  useEffect(() => {
    if (localSettings === settings) return;
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(async () => {
      try {
        await saveSettings(localSettings);
        setSaved(true);
        setTimeout(() => setSaved(false), 1500);
      } catch (err) {
        console.error('Failed to auto-save settings:', err);
      }
    }, 400);
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [localSettings, settings, saveSettings]);

  return (
    <div className="max-w-2xl space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold mb-1">{t('settings.analytics.title')}</h2>
          <p className="text-sm text-muted-foreground">
            {t('settings.analytics.description')}
          </p>
        </div>
        {saved && (
          <span className="text-sm text-green-600 transition-opacity">{t('settings.saved')}</span>
        )}
      </div>

      {/* Anomaly Threshold */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('settings.analytics.anomaly')}</h3>
        <label className="block text-sm mb-2">
          <span className="text-muted-foreground">{t('settings.analytics.anomalyThreshold')}</span>
          <div className="flex items-center gap-3 mt-1">
            <input
              type="range"
              min="1"
              max="5"
              step="0.1"
              value={localSettings.anomaly_threshold}
              onChange={(e) =>
                setLocalSettings({
                  ...localSettings,
                  anomaly_threshold: parseFloat(e.target.value),
                })
              }
              className="flex-1"
            />
            <span className="w-12 text-center font-mono">
              {localSettings.anomaly_threshold.toFixed(1)}
            </span>
          </div>
        </label>
        <p className="text-xs text-muted-foreground">
          {t('settings.analytics.anomalyDesc', { value: localSettings.anomaly_threshold.toFixed(1) })}
        </p>
      </div>

      {/* Forecast Method */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('settings.analytics.forecast')}</h3>
        <div className="space-y-2">
          {[
            { value: 'linear', label: t('settings.analytics.forecastLinear'), desc: t('settings.analytics.forecastLinearDesc') },
            { value: 'moving_avg', label: t('settings.analytics.forecastMoving'), desc: t('settings.analytics.forecastMovingDesc') },
            { value: 'exponential', label: t('settings.analytics.forecastExponential'), desc: t('settings.analytics.forecastExponentialDesc') },
          ].map((option) => (
            <label
              key={option.value}
              className="flex items-start gap-3 p-3 rounded-lg border hover:bg-muted/50 cursor-pointer"
            >
              <input
                type="radio"
                name="forecast_method"
                value={option.value}
                checked={localSettings.forecast_method === option.value}
                onChange={(e) =>
                  setLocalSettings({
                    ...localSettings,
                    forecast_method: e.target.value,
                  })
                }
                className="mt-0.5"
              />
              <div>
                <div className="font-medium text-sm">{option.label}</div>
                <div className="text-xs text-muted-foreground">{option.desc}</div>
              </div>
            </label>
          ))}
        </div>
      </div>

      {/* Data Retention */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('settings.analytics.retention')}</h3>
        <label className="block text-sm">
          <span className="text-muted-foreground">{t('settings.analytics.retentionDays')}</span>
          <select
            value={localSettings.data_retention_days}
            onChange={(e) =>
              setLocalSettings({
                ...localSettings,
                data_retention_days: parseInt(e.target.value),
              })
            }
            className="mt-1 block w-full rounded-md border bg-background px-3 py-2 text-sm"
          >
            <option value={30}>{t('settings.analytics.retentionDaysLabel', { days: 30 })}</option>
            <option value={60}>{t('settings.analytics.retentionDaysLabel', { days: 60 })}</option>
            <option value={90}>{t('settings.analytics.retentionDaysDefault', { days: 90 })}</option>
            <option value={180}>{t('settings.analytics.retentionDaysLabel', { days: 180 })}</option>
            <option value={365}>{t('settings.analytics.retentionYear')}</option>
            <option value={0}>{t('settings.analytics.retentionNever')}</option>
          </select>
        </label>
        <p className="text-xs text-muted-foreground mt-2">
          {t('settings.analytics.retentionDesc')}
        </p>
      </div>
    </div>
  );
}
