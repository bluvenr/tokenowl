import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { formatUsd, formatPercent, formatLatency } from '@/lib/format';
import { cn } from '@/lib/utils';
import { CHART_COLORS } from '@/lib/constants';
import { AlertTriangle } from 'lucide-react';

function FailureBadge({ rate }: { rate: number }) {
  const pct = rate * 100;
  const isGood = pct < 1;
  const isWarn = pct >= 1 && pct < 5;

  return (
    <span
      className={cn(
        'inline-flex items-center gap-0.5 text-xs font-medium px-1.5 py-0.5 rounded',
        isGood && 'text-green-700 dark:text-green-400 bg-green-100 dark:bg-green-900/30',
        isWarn && 'text-yellow-700 dark:text-yellow-400 bg-yellow-100 dark:bg-yellow-900/30',
        !isGood && !isWarn && 'text-red-700 dark:text-red-400 bg-red-100 dark:bg-red-900/30',
      )}
    >
      {!isGood && <AlertTriangle className="h-3 w-3" />}
      {pct.toFixed(1)}%
    </span>
  );
}

export function ProviderBreakdown() {
  const { t } = useTranslation();
  const providerUsage = useDashboardStore((s) => s.providerUsage);

  if (providerUsage.length === 0) {
    return (
      <div className="rounded-lg border bg-card p-4 shadow-sm">
        <h3 className="font-semibold mb-4">{t('dashboard.providerBreakdown')}</h3>
        <div className="flex h-32 items-center justify-center text-muted-foreground">
          {t('dashboard.noData')}
        </div>
      </div>
    );
  }

  const maxCost = Math.max(...providerUsage.map((p) => p.cost_usd));

  return (
    <div className="rounded-lg border bg-card p-4 shadow-sm">
      <h3 className="font-semibold mb-4">{t('dashboard.providerBreakdown')}</h3>

      {/* Table header */}
      <div className="grid grid-cols-[minmax(0,1fr)_auto_auto_auto_auto] gap-4 text-xs text-muted-foreground font-medium mb-2 px-1">
        <span className="truncate">{t('table.provider')}</span>
        <span className="text-right w-20">{t('table.latency')}</span>
        <span className="text-right w-20">{t('dashboard.requests')}</span>
        <span className="text-right w-20">{t('provider.failureRate')}</span>
        <span className="text-right w-24">{t('dashboard.cost')}</span>
      </div>

      <div className="space-y-3">
        {providerUsage.map((provider, index) => (
          <div key={provider.provider_name} className="space-y-1">
            <div className="grid grid-cols-[minmax(0,1fr)_auto_auto_auto_auto] gap-4 items-center text-sm px-1">
              <div className="flex items-center gap-2 min-w-0 overflow-hidden">
                <div
                  className="h-3 w-3 rounded-full shrink-0"
                  style={{
                    backgroundColor:
                      CHART_COLORS[index % CHART_COLORS.length],
                  }}
                />
                <span className="font-medium truncate max-w-[min(40vw,200px)]" title={provider.provider_name}>{provider.provider_name}</span>
                <span className="text-xs text-muted-foreground shrink-0">
                  {formatPercent(provider.percentage)}
                </span>
              </div>
              <span className="text-right text-muted-foreground w-20">
                {formatLatency(provider.avg_latency_ms)}
              </span>
              <span className="text-right text-muted-foreground w-20">
                {provider.request_count.toLocaleString()}
              </span>
              <div className="text-right w-20">
                <FailureBadge rate={provider.failure_rate} />
              </div>
              <span className="text-right font-medium w-24">
                {formatUsd(provider.cost_usd)}
              </span>
            </div>
            <div className="h-2 rounded-full bg-muted">
              <div
                className="h-full rounded-full transition-all"
                style={{
                  width: `${(provider.cost_usd / maxCost) * 100}%`,
                  backgroundColor:
                    CHART_COLORS[index % CHART_COLORS.length],
                }}
              />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
