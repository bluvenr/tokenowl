import { useEffect, useState, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useAppStore } from '@/stores/app';
import { LANGUAGE_LABELS, PERIODS } from '@/lib/constants';

export function GeneralTab() {
  const { t } = useTranslation();
  const settings = useAppStore((s) => s.settings);
  const saveSettings = useAppStore((s) => s.saveSettings);
  const [localSettings, setLocalSettings] = useState(settings);
  const [saved, setSaved] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    setLocalSettings(settings);
  }, [settings]);

  // Auto-save with debounce when localSettings change
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
          <h2 className="text-xl font-semibold mb-1">{t('settings.general.title')}</h2>
          <p className="text-sm text-muted-foreground">
            {t('settings.general.description')}
          </p>
        </div>
        {saved && (
          <span className="text-sm text-green-600 transition-opacity">{t('settings.saved')}</span>
        )}
      </div>

      {/* Language */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('settings.general.language')}</h3>
        <select
          value={localSettings.language}
          onChange={(e) =>
            setLocalSettings({ ...localSettings, language: e.target.value })
          }
          className="block w-full rounded-md border bg-background px-3 py-2 text-sm"
        >
          {Object.entries(LANGUAGE_LABELS).map(([value, label]) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </select>
      </div>

      {/* Theme */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('settings.general.theme')}</h3>
        <div className="flex gap-3">
          {[
            { value: 'light', label: t('settings.general.themeLight') },
            { value: 'dark', label: t('settings.general.themeDark') },
            { value: 'system', label: t('settings.general.themeSystem') },
          ].map((option) => (
            <label
              key={option.value}
              className="flex items-center gap-2 cursor-pointer hover:bg-muted/50 rounded-md px-2 py-1 transition-colors"
            >
              <input
                type="radio"
                name="theme"
                value={option.value}
                checked={localSettings.theme === option.value}
                onChange={(e) =>
                  setLocalSettings({ ...localSettings, theme: e.target.value })
                }
              />
              <span className="text-sm">{option.label}</span>
            </label>
          ))}
        </div>
      </div>

      {/* Startup */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('settings.general.startup')}</h3>
        <label className="flex items-center gap-3 cursor-pointer">
          <input
            type="checkbox"
            checked={localSettings.auto_start}
            onChange={(e) =>
              setLocalSettings({ ...localSettings, auto_start: e.target.checked })
            }
            className="rounded"
          />
          <span className="text-sm">{t('settings.general.autoStart')}</span>
        </label>
      </div>

      {/* Tray Display */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('settings.general.trayDisplay')}</h3>
        <select
          value={localSettings.tray_display}
          onChange={(e) =>
            setLocalSettings({ ...localSettings, tray_display: e.target.value })
          }
          className="block w-full rounded-md border bg-background px-3 py-2 text-sm"
        >
          <option value="cost">{t('settings.general.trayCost')}</option>
          <option value="tokens">{t('settings.general.trayTokens')}</option>
          <option value="requests">{t('settings.general.trayRequests')}</option>
        </select>
      </div>

      {/* Default Period */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('settings.general.defaultPeriod')}</h3>
        <p className="text-xs text-muted-foreground mb-3">
          {t('settings.general.defaultPeriodDesc')}
        </p>
        <select
          value={localSettings.default_period}
          onChange={(e) =>
            setLocalSettings({ ...localSettings, default_period: e.target.value })
          }
          className="block w-full rounded-md border bg-background px-3 py-2 text-sm"
        >
          {Object.entries(PERIODS).map(([, value]) => (
            <option key={value} value={value}>
              {t(`period.${value}`)}
            </option>
          ))}
        </select>
      </div>

      {/* Privacy */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('settings.general.privacy')}</h3>
        <label className="flex items-center gap-3 cursor-pointer">
          <input
            type="checkbox"
            checked={localSettings.crash_log_enabled}
            onChange={(e) =>
              setLocalSettings({
                ...localSettings,
                crash_log_enabled: e.target.checked,
              })
            }
            className="rounded"
          />
          <div>
            <span className="text-sm">{t('settings.general.crashLog')}</span>
            <p className="text-xs text-muted-foreground">
              {t('settings.general.crashLogDesc')}
            </p>
          </div>
        </label>
      </div>
    </div>
  );
}
