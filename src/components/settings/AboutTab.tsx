import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useAppStore } from '@/stores/app';
import {
  getCrashLogs,
  clearCrashLogs,
  getCrashIssueUrl,
  type CrashLogEntry,
} from '@/lib/tauri';
import { formatDateTime } from '@/lib/format';
import { GITHUB_REPO } from '@/lib/constants';
import {
  ExternalLink,
  Trash2,
  AlertTriangle,
  RefreshCw,
} from 'lucide-react';

export function AboutTab() {
  const { t } = useTranslation();
  const version = useAppStore((s) => s.version);
  const checkUpdate = useAppStore((s) => s.checkUpdate);
  const [crashLogs, setCrashLogs] = useState<CrashLogEntry[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadCrashLogs();
  }, []);

  const loadCrashLogs = async () => {
    setLoading(true);
    try {
      const logs = await getCrashLogs();
      setCrashLogs(logs);
    } catch (err) {
      console.error('Failed to load crash logs:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleClearLogs = async () => {
    if (!confirm(t('about.clearConfirm'))) return;
    try {
      await clearCrashLogs();
      setCrashLogs([]);
    } catch (err) {
      console.error('Failed to clear crash logs:', err);
    }
  };

  const handleReportIssue = async () => {
    try {
      const url = await getCrashIssueUrl();
      window.open(url, '_blank');
    } catch (err) {
      window.open(`${GITHUB_REPO}/issues/new`, '_blank');
    }
  };

  return (
    <div className="max-w-2xl space-y-6">
      <div>
        <h2 className="text-xl font-semibold mb-1">{t('about.title')}</h2>
        <p className="text-sm text-muted-foreground">
          {t('about.description')}
        </p>
      </div>

      {/* App Info */}
      <div className="rounded-lg border p-4">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="font-medium">TokenOwl</h3>
            <p className="text-sm text-muted-foreground">
              {t('app.tagline')}
            </p>
          </div>
          <div className="text-right">
            <div className="font-mono text-sm">
              v{version?.current || '1.0.0'}
            </div>
            {version?.update_available ? (
              <div className="text-sm text-green-600">
                {t('about.updateAvailable', { version: version.latest })}
              </div>
            ) : (
              <div className="text-xs text-muted-foreground">{t('about.latest')}</div>
            )}
          </div>
        </div>
        <button
          onClick={() => checkUpdate()}
          className="mt-3 inline-flex items-center gap-2 rounded-md border px-3 py-1.5 text-sm hover:bg-muted"
        >
          <RefreshCw className="h-4 w-4" />
          {t('about.checkUpdate')}
        </button>
      </div>

      {/* Links */}
      <div className="rounded-lg border p-4">
        <h3 className="font-medium mb-3">{t('about.links')}</h3>
        <div className="space-y-2">
          <a
            href={GITHUB_REPO}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 text-sm text-primary hover:underline"
          >
            <ExternalLink className="h-4 w-4" />
            {t('about.github')}
          </a>
          <a
            href={`${GITHUB_REPO}/issues`}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 text-sm text-primary hover:underline"
          >
            <ExternalLink className="h-4 w-4" />
            {t('about.reportIssue')}
          </a>
        </div>
      </div>

      {/* Crash Logs */}
      <div className="rounded-lg border p-4">
        <div className="flex items-center justify-between mb-3">
          <h3 className="font-medium">{t('about.crashLogs')}</h3>
          <div className="flex gap-2">
            <button
              onClick={handleReportIssue}
              className="inline-flex items-center gap-1 rounded-md border px-2 py-1 text-xs hover:bg-muted"
            >
              <AlertTriangle className="h-3 w-3" />
              {t('about.reportIssue')}
            </button>
            <button
              onClick={handleClearLogs}
              disabled={crashLogs.length === 0}
              className="inline-flex items-center gap-1 rounded-md border px-2 py-1 text-xs hover:bg-muted disabled:opacity-50"
            >
              <Trash2 className="h-3 w-3" />
              {t('about.clearLogs')}
            </button>
          </div>
        </div>

        {loading ? (
          <div className="flex h-20 items-center justify-center">
            <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-primary"></div>
          </div>
        ) : crashLogs.length === 0 ? (
          <div className="py-8 text-center text-sm text-muted-foreground">
            {t('about.noCrashLogs')}
          </div>
        ) : (
          <div className="max-h-64 overflow-y-auto space-y-2">
            {crashLogs.map((log) => (
              <div
                key={log.id}
                className="rounded border bg-muted/50 p-2 text-xs"
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="font-medium text-red-600">
                    {log.error_type}
                  </span>
                  <span className="text-muted-foreground">
                    {formatDateTime(log.timestamp)}
                  </span>
                </div>
                <div className="text-foreground font-mono truncate">
                  {log.message}
                </div>
                {log.backtrace && (
                  <details className="mt-1">
                    <summary className="cursor-pointer text-muted-foreground">
                      {t('about.viewStack')}
                    </summary>
                    <pre className="mt-1 overflow-x-auto text-muted-foreground whitespace-pre-wrap">
                      {log.backtrace}
                    </pre>
                  </details>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* License */}
      <div className="text-center text-xs text-muted-foreground">
        <p>{t('about.copyright')}</p>
        <p>{t('about.license')}</p>
      </div>
    </div>
  );
}
