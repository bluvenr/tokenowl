import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { formatUsd } from '@/lib/format';
import { AlertTriangle, TrendingUp, Info } from 'lucide-react';

export function CostAnomalyCard() {
  const { t } = useTranslation();
  const report = useDashboardStore((s) => s.costAnomalyReport);

  if (!report) return null;

  const anomalyCount = report.anomaly_days.length;

  return (
    <div className="rounded-lg border bg-card p-4 shadow-sm">
      <div className="flex items-center gap-2 mb-4">
        <AlertTriangle className="h-5 w-5 text-orange-500" />
        <h3 className="font-semibold">{t('anomaly.title')}</h3>
        <span className="ml-auto rounded-full bg-orange-100 px-2 py-0.5 text-xs font-medium text-orange-700 dark:bg-orange-900/30 dark:text-orange-400">
          {anomalyCount} {t('anomaly.days')}
        </span>
      </div>

      {anomalyCount === 0 ? (
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Info className="h-4 w-4" />
          <span>{t('anomaly.noAnomalies')}</span>
        </div>
      ) : (
        <div className="space-y-3">
          {/* Summary stats */}
          <div className="grid grid-cols-3 gap-4 text-center">
            <div>
              <div className="text-xs text-muted-foreground">{t('anomaly.totalDays')}</div>
              <div className="font-semibold">{report.total_days}</div>
            </div>
            <div>
              <div className="text-xs text-muted-foreground">{t('anomaly.avgCost')}</div>
              <div className="font-semibold">{formatUsd(report.avg_daily_cost)}</div>
            </div>
            <div>
              <div className="text-xs text-muted-foreground">{t('anomaly.threshold')}</div>
              <div className="font-semibold">{report.threshold}σ</div>
            </div>
          </div>

          {/* Anomaly list */}
          <div className="space-y-2 max-h-48 overflow-y-auto">
            {report.anomaly_days.map((day) => (
              <div
                key={day.date}
                className="flex items-center gap-3 rounded-lg bg-muted/50 p-2 text-sm"
              >
                <TrendingUp className="h-4 w-4 shrink-0 text-red-500" />
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{day.date}</span>
                    <span className="text-red-500 font-semibold">
                      {formatUsd(day.cost_usd)}
                    </span>
                  </div>
                  <div className="flex items-center gap-2 text-xs text-muted-foreground">
                    <span>{day.deviation.toFixed(1)}σ</span>
                    {day.top_provider && (
                      <>
                        <span>·</span>
                        <span>{day.top_provider}</span>
                      </>
                    )}
                    {day.top_model && (
                      <>
                        <span>·</span>
                        <span className="truncate">{day.top_model}</span>
                      </>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
