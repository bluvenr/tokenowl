import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { formatUsd, formatPercent } from '@/lib/format';
import { Lightbulb, Target, Clock } from 'lucide-react';
import { cn } from '@/lib/utils';

export function SavingsInsights() {
  const { t } = useTranslation();
  const savingsAnalysis = useDashboardStore((s) => s.savingsAnalysis);
  const budgetBurnRate = useDashboardStore((s) => s.budgetBurnRate);

  if (!savingsAnalysis) return null;

  const concentrationColor = {
    high: 'text-red-500',
    moderate: 'text-orange-500',
    diverse: 'text-green-500',
  }[savingsAnalysis.model_concentration] || 'text-muted-foreground';

  const concentrationLabel = {
    high: t('insights.high'),
    moderate: t('insights.moderate'),
    diverse: t('insights.diverse'),
  }[savingsAnalysis.model_concentration] || savingsAnalysis.model_concentration;

  return (
    <div className="rounded-lg border bg-card p-4 shadow-sm">
      <div className="flex items-center gap-2 mb-4">
        <Lightbulb className="h-5 w-5 text-yellow-500" />
        <h3 className="font-semibold">{t('insights.title')}</h3>
      </div>

      <div className="grid gap-4 md:grid-cols-4 mb-4">
        <div className="text-center">
          <div className="text-sm text-muted-foreground">{t('insights.cacheHitRate')}</div>
          <div className="text-xl font-bold">
            {formatPercent(savingsAnalysis.cache_hit_rate)}
          </div>
        </div>
        <div className="text-center">
          <div className="text-sm text-muted-foreground">{t('insights.cacheSavings')}</div>
          <div className="text-xl font-bold text-green-500">
            {formatUsd(savingsAnalysis.cache_savings_usd)}
          </div>
        </div>
        <div className="text-center">
          <div className="text-sm text-muted-foreground">{t('insights.modelConcentration')}</div>
          <div className={cn('text-xl font-bold', concentrationColor)}>
            {concentrationLabel}
          </div>
        </div>
        <div className="text-center">
          <div className="text-sm text-muted-foreground">{t('insights.monthlyForecast')}</div>
          <div className="text-xl font-bold">
            {formatUsd(savingsAnalysis.monthly_forecast_usd)}
          </div>
          <div className="text-xs text-muted-foreground">
            {t('insights.confidence')} {formatPercent(savingsAnalysis.forecast_confidence * 100)}
          </div>
          {budgetBurnRate?.monthly_days_remaining != null && (
            <div className={cn(
              'mt-1 flex items-center justify-center gap-1 text-xs font-medium',
              budgetBurnRate.monthly_days_remaining <= 0
                ? 'text-red-500'
                : budgetBurnRate.monthly_days_remaining <= 7
                  ? 'text-orange-500'
                  : 'text-green-500'
            )}>
              <Clock className="h-3 w-3" />
              {budgetBurnRate.monthly_days_remaining <= 0
                ? t('burnRate.exhausted')
                : `${Math.floor(budgetBurnRate.monthly_days_remaining)} ${t('insights.daysLeft')}`}
            </div>
          )}
        </div>
      </div>

      {savingsAnalysis.recommendations.length > 0 && (
        <div className="space-y-2">
          <div className="text-sm font-medium text-muted-foreground">{t('insights.recommendations')}</div>
          {savingsAnalysis.recommendations.map((rec, index) => (
            <div
              key={index}
              className="flex items-start gap-2 rounded-lg bg-muted/50 p-3 text-sm"
            >
              <Target className="h-4 w-4 mt-0.5 shrink-0 text-primary" />
              <span>{rec}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
