import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { CheckCircle, XCircle, Wifi, WifiOff, Server, Activity } from 'lucide-react';
import { cn } from '@/lib/utils';

export function CcSwitchStatus() {
  const { t } = useTranslation();
  const ccSwitchStatus = useDashboardStore((s) => s.ccSwitchStatus);

  if (!ccSwitchStatus) return null;

  const successRate = ccSwitchStatus.success_rate;
  const isSuccessGood = successRate >= 95;
  const isSuccessWarn = successRate >= 80 && successRate < 95;

  return (
    <div className="rounded-lg border bg-card p-4 shadow-sm">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h3 className="font-semibold">{t('ccswitch.title')}</h3>
          {ccSwitchStatus.detected ? (
            <span className="inline-flex items-center gap-1 rounded-full bg-green-100 dark:bg-green-900/30 px-2 py-0.5 text-xs font-medium text-green-700 dark:text-green-400">
              <CheckCircle className="h-3 w-3" />
              {t('ccswitch.detected')}
            </span>
          ) : (
            <span className="inline-flex items-center gap-1 rounded-full bg-red-100 dark:bg-red-900/30 px-2 py-0.5 text-xs font-medium text-red-700 dark:text-red-400">
              <XCircle className="h-3 w-3" />
              {t('ccswitch.notDetected')}
            </span>
          )}
        </div>
        <div className="flex items-center gap-4 text-sm text-muted-foreground">
          <span className="flex items-center gap-1">
            {ccSwitchStatus.proxy_running ? (
              <>
                <Wifi className="h-4 w-4 text-green-500" />
                {t('ccswitch.proxyRunning')}
              </>
            ) : (
              <>
                <WifiOff className="h-4 w-4" />
                {t('ccswitch.proxyNotRunning')}
              </>
            )}
          </span>
          <span className="flex items-center gap-1">
            <Server className="h-4 w-4 text-blue-500" />
            {ccSwitchStatus.provider_count} {t('ccswitch.providers')}
          </span>
          <span
            className={cn(
              'flex items-center gap-1',
              isSuccessGood && 'text-green-600 dark:text-green-400',
              isSuccessWarn && 'text-yellow-600 dark:text-yellow-400',
              !isSuccessGood && !isSuccessWarn && 'text-red-600 dark:text-red-400',
            )}
          >
            <Activity className="h-4 w-4" />
            {successRate.toFixed(1)}% {t('ccswitch.successRate')}
          </span>
          <span>{t('ccswitch.totalRecords', { count: ccSwitchStatus.total_records })}</span>
        </div>
      </div>
      {ccSwitchStatus.db_path && (
        <div className="mt-2 text-xs text-muted-foreground font-mono truncate">
          {ccSwitchStatus.db_path}
        </div>
      )}
    </div>
  );
}
