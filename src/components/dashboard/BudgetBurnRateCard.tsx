import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { formatUsd } from '@/lib/format';
import { Flame, Clock, AlertTriangle } from 'lucide-react';

function BurnRateItem({ label, spend, limit, daysRemaining }: { label: string; spend: number | null; limit: number | null; daysRemaining: number | null }) {
  const { t } = useTranslation();

  if (limit === null) return null;

  const days = daysRemaining ?? 0;
  const isExhausted = days === 0;
  const isLow = days > 0 && days <= 3;

  return (
    <div className="flex items-center justify-between py-2 border-b last:border-b-0">
      <div className="flex-1">
        <div className="text-sm font-medium">{label}</div>
        <div className="text-xs text-muted-foreground">
          {formatUsd(spend ?? 0)} / {formatUsd(limit)}
        </div>
      </div>
      <div className="flex items-center gap-2">
        {isExhausted ? (
          <>
            <AlertTriangle className="h-4 w-4 text-red-500" />
            <span className="text-sm font-semibold text-red-500">{t('burnRate.exhausted')}</span>
          </>
        ) : isLow ? (
          <>
            <AlertTriangle className="h-4 w-4 text-orange-500" />
            <span className="text-sm font-semibold text-orange-500">
              {days.toFixed(1)} {t('burnRate.days')}
            </span>
          </>
        ) : (
          <>
            <Clock className="h-4 w-4 text-green-500" />
            <span className="text-sm font-semibold">
              {days.toFixed(1)} {t('burnRate.days')}
            </span>
          </>
        )}
      </div>
    </div>
  );
}

export function BudgetBurnRateCard() {
  const { t } = useTranslation();
  const burnRate = useDashboardStore((s) => s.budgetBurnRate);

  if (!burnRate) return null;

  // Only show if at least one budget is set
  const hasAnyBudget = burnRate.daily_limit !== null || burnRate.weekly_limit !== null || burnRate.monthly_limit !== null;
  if (!hasAnyBudget) return null;

  return (
    <div className="rounded-lg border bg-card p-4 shadow-sm">
      <div className="flex items-center gap-2 mb-4">
        <Flame className="h-5 w-5 text-orange-500" />
        <h3 className="font-semibold">{t('burnRate.title')}</h3>
        <span className="ml-auto text-xs text-muted-foreground">
          {t('burnRate.dailyRate')}: {formatUsd(burnRate.daily_rate)}
        </span>
      </div>

      <div className="space-y-0">
        <BurnRateItem
          label={t('burnRate.daily')}
          spend={burnRate.daily_spend}
          limit={burnRate.daily_limit}
          daysRemaining={burnRate.daily_days_remaining}
        />
        <BurnRateItem
          label={t('burnRate.weekly')}
          spend={burnRate.weekly_spend}
          limit={burnRate.weekly_limit}
          daysRemaining={burnRate.weekly_days_remaining}
        />
        <BurnRateItem
          label={t('burnRate.monthly')}
          spend={burnRate.monthly_spend}
          limit={burnRate.monthly_limit}
          daysRemaining={burnRate.monthly_days_remaining}
        />
      </div>
    </div>
  );
}
