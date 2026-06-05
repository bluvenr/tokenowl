import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { PERIODS } from '@/lib/constants';
import { cn } from '@/lib/utils';

export function PeriodSelector() {
  const { t } = useTranslation();
  const period = useDashboardStore((s) => s.period);
  const setPeriod = useDashboardStore((s) => s.setPeriod);

  return (
    <div className="inline-flex rounded-lg border bg-muted/50 p-1">
      {Object.entries(PERIODS).map(([, value]) => (
        <button
          key={value}
          onClick={() => setPeriod(value)}
          className={cn(
            'px-3 py-1.5 text-sm font-medium rounded-md transition-colors',
            period === value
              ? 'bg-accent text-accent-foreground shadow-sm'
              : 'text-muted-foreground hover:text-foreground'
          )}
        >
          {t(`period.${value}`)}
        </button>
      ))}
    </div>
  );
}
