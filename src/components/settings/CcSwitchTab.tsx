import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  getCcSwitchStatus,
  syncCcSwitch,
  ccswitchUpdateSyncConfig,
  ccswitchSetDbPath,
  type CcSwitchStatus as CcSwitchStatusType,
  type SyncResult,
} from '@/lib/tauri';
import { formatDateTime } from '@/lib/format';
import {
  CheckCircle,
  XCircle,
  Wifi,
  WifiOff,
  RefreshCw,
  FolderOpen,
  Clock,
  MapPin,
  Save,
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { open } from '@tauri-apps/plugin-dialog';

const SYNC_INTERVALS = [
  { value: 0, label: 'ccswitch.syncOff' },
  { value: 60, label: 'ccswitch.syncInterval1m' },
  { value: 300, label: 'ccswitch.syncInterval5m' },
  { value: 900, label: 'ccswitch.syncInterval15m' },
  { value: 1800, label: 'ccswitch.syncInterval30m' },
  { value: 3600, label: 'ccswitch.syncInterval1h' },
];

export function CcSwitchTab() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<CcSwitchStatusType | null>(null);
  const [syncing, setSyncing] = useState(false);
  const [lastSync, setLastSync] = useState<SyncResult | null>(null);
  const [loading, setLoading] = useState(true);

  // Sync config state
  const [syncInterval, setSyncInterval] = useState(300);
  const [customPath, setCustomPath] = useState('');
  const [pathSaving, setPathSaving] = useState(false);
  const [pathMsg, setPathMsg] = useState<{ type: 'ok' | 'err'; text: string } | null>(null);
  const [intervalSaving, setIntervalSaving] = useState(false);

  useEffect(() => {
    loadStatus();
  }, []);

  const loadStatus = async () => {
    try {
      const s = await getCcSwitchStatus();
      setStatus(s);
      setSyncInterval(s.sync_interval_secs);
      setCustomPath(s.custom_db_path ?? '');
    } catch (err) {
      console.error('Failed to load CC Switch status:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSync = async () => {
    setSyncing(true);
    try {
      const result = await syncCcSwitch();
      setLastSync(result);
      await loadStatus();
    } catch (err) {
      console.error('Sync failed:', err);
    } finally {
      setSyncing(false);
    }
  };

  const handleIntervalChange = async (value: number) => {
    setSyncInterval(value);
    setIntervalSaving(true);
    try {
      await ccswitchUpdateSyncConfig(value);
    } catch (err) {
      console.error('Failed to update sync interval:', err);
    } finally {
      setIntervalSaving(false);
    }
  };

  const handleSavePath = async () => {
    setPathSaving(true);
    setPathMsg(null);
    try {
      await ccswitchSetDbPath(customPath || null);
      setPathMsg({ type: 'ok', text: t('ccswitch.pathSaved') });
      await loadStatus();
    } catch (err) {
      setPathMsg({ type: 'err', text: String(err) });
    } finally {
      setPathSaving(false);
    }
  };

  const handleBrowsePath = async () => {
    try {
      const selected = await open({
        title: t('ccswitch.selectDb'),
        filters: [{ name: 'SQLite Database', extensions: ['db', 'sqlite', 'sqlite3'] }],
        multiple: false,
      });
      if (selected) {
        setCustomPath(selected);
      }
    } catch (err) {
      console.error('File dialog error:', err);
      setPathMsg({ type: 'err', text: t('ccswitch.dialogError') });
    }
  };

  if (loading) {
    return (
      <div className="flex h-32 items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="max-w-2xl space-y-6">
      <div>
        <h2 className="text-xl font-semibold mb-1">{t('ccswitch.connection')}</h2>
        <p className="text-sm text-muted-foreground">
          {t('ccswitch.connectionDesc')}
        </p>
      </div>

      {/* Connection Status */}
      <div className="rounded-lg border p-4">
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-medium">{t('ccswitch.connectionStatus')}</h3>
          {status?.detected ? (
            <span className="inline-flex items-center gap-1 rounded-full bg-green-100 dark:bg-green-900/30 px-3 py-1 text-sm font-medium text-green-700 dark:text-green-400">
              <CheckCircle className="h-4 w-4" />
              {t('ccswitch.connected')}
            </span>
          ) : (
            <span className="inline-flex items-center gap-1 rounded-full bg-red-100 dark:bg-red-900/30 px-3 py-1 text-sm font-medium text-red-700 dark:text-red-400">
              <XCircle className="h-4 w-4" />
              {t('ccswitch.notDetected')}
            </span>
          )}
        </div>

        {status && (
          <dl className="space-y-3 text-sm">
            <div className="flex justify-between">
              <dt className="text-muted-foreground">{t('ccswitch.dbPath')}</dt>
              <dd className="font-mono text-xs flex items-center gap-1">
                {status.db_path || status.custom_db_path || '-'}
                {(status.db_path || status.custom_db_path) && (
                  <FolderOpen className="h-3 w-3 cursor-pointer hover:text-primary" />
                )}
              </dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-muted-foreground">{t('ccswitch.proxyStatus')}</dt>
              <dd className="flex items-center gap-1">
                {status.proxy_running ? (
                  <>
                    <Wifi className="h-4 w-4 text-green-500" />
                    {t('ccswitch.running')}
                  </>
                ) : (
                  <>
                    <WifiOff className="h-4 w-4 text-muted-foreground" />
                    {t('ccswitch.notRunning')}
                  </>
                )}
              </dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-muted-foreground">{t('ccswitch.recordCount')}</dt>
              <dd>{status.total_records.toLocaleString()}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-muted-foreground">{t('ccswitch.lastSync')}</dt>
              <dd>
                {status.last_sync_time
                  ? formatDateTime(status.last_sync_time)
                  : t('ccswitch.never')}
              </dd>
            </div>
          </dl>
        )}
      </div>

      {/* Custom Database Path */}
      <div className="rounded-lg border p-4">
        <div className="flex items-center gap-2 mb-3">
          <MapPin className="h-4 w-4 text-muted-foreground" />
          <h3 className="font-medium">{t('ccswitch.customPath')}</h3>
        </div>
        <p className="text-sm text-muted-foreground mb-3">
          {t('ccswitch.customPathDesc')}
        </p>
        <div className="flex gap-2">
          <input
            type="text"
            value={customPath}
            onChange={(e) => setCustomPath(e.target.value)}
            placeholder={t('ccswitch.customPathPlaceholder')}
            className="flex-1 rounded-md border bg-background px-3 py-2 text-sm font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
          />
          <button
            onClick={handleBrowsePath}
            className="rounded-md border px-3 py-2 text-sm hover:bg-accent transition-colors"
          >
            <FolderOpen className="h-4 w-4" />
          </button>
          <button
            onClick={handleSavePath}
            disabled={pathSaving}
            className="inline-flex items-center gap-1 rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50 transition-colors"
          >
            <Save className="h-4 w-4" />
            {t('ccswitch.savePath')}
          </button>
        </div>
        {pathMsg && (
          <p className={cn('mt-2 text-xs', pathMsg.type === 'ok' ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400')}>
            {pathMsg.text}
          </p>
        )}
      </div>

      {/* Auto-Sync Config */}
      <div className="rounded-lg border p-4">
        <div className="flex items-center gap-2 mb-3">
          <Clock className="h-4 w-4 text-muted-foreground" />
          <h3 className="font-medium">{t('ccswitch.autoSync')}</h3>
        </div>
        <p className="text-sm text-muted-foreground mb-3">
          {t('ccswitch.autoSyncDesc')}
        </p>
        <div className="flex items-center gap-3">
          <label className="text-sm text-muted-foreground">{t('ccswitch.syncInterval')}</label>
          <select
            value={syncInterval}
            onChange={(e) => handleIntervalChange(Number(e.target.value))}
            disabled={intervalSaving}
            className="rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
          >
            {SYNC_INTERVALS.map(({ value, label }) => (
              <option key={value} value={value}>
                {t(label)}
              </option>
            ))}
          </select>
          {intervalSaving && (
            <span className="text-xs text-muted-foreground">{t('settings.saving')}</span>
          )}
        </div>
      </div>

      {/* Manual Sync Section */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('ccswitch.sync')}</h3>
        <p className="text-sm text-muted-foreground mb-4">
          {t('ccswitch.syncDesc')}
        </p>
        <button
          onClick={handleSync}
          disabled={syncing || !status?.detected}
          className={cn(
            'inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors',
            'hover:bg-primary/90',
            (syncing || !status?.detected) && 'opacity-50 cursor-not-allowed'
          )}
        >
          <RefreshCw className={cn('h-4 w-4', syncing && 'animate-spin')} />
          {syncing ? t('dashboard.syncing') : t('ccswitch.syncNow')}
        </button>

        {lastSync && (
          <div className="mt-4 rounded-lg bg-muted p-3 text-sm">
            <div className="font-medium mb-1">{t('ccswitch.syncDone')}</div>
            <ul className="space-y-1 text-muted-foreground">
              <li>{t('ccswitch.newRecordsCount', { count: lastSync.new_records })}</li>
              <li>{t('ccswitch.skippedCount', { count: lastSync.skipped_duplicates })}</li>
              <li>{t('ccswitch.errorCount', { count: lastSync.errors })}</li>
              <li>{t('ccswitch.durationMs', { ms: lastSync.sync_duration_ms })}</li>
            </ul>
          </div>
        )}
      </div>

      {/* Troubleshooting */}
      {!status?.detected && (
        <div className="rounded-lg border border-orange-200 dark:border-orange-800 bg-orange-50 dark:bg-orange-950/30 p-4">
          <h3 className="font-medium text-orange-800 dark:text-orange-300 mb-2">
            {t('ccswitch.troubleshoot.title')}
          </h3>
          <ul className="text-sm text-orange-700 dark:text-orange-400 space-y-1 list-disc list-inside">
            <li>{t('ccswitch.troubleshoot.step1')}</li>
            <li>{t('ccswitch.troubleshoot.step2')}</li>
            <li>{t('ccswitch.troubleshoot.step3')}</li>
          </ul>
        </div>
      )}
    </div>
  );
}
