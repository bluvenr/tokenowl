import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { formatUsd } from '@/lib/format';
import { CHART_COLORS, GRANULARITIES } from '@/lib/constants';
import { cn } from '@/lib/utils';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';

export function CostTrendChart() {
  const { t } = useTranslation();
  const trend = useDashboardStore((s) => s.trend);
  const granularity = useDashboardStore((s) => s.granularity);
  const setGranularity = useDashboardStore((s) => s.setGranularity);

  const data = trend.map((point) => ({
    time: point.timestamp,
    cost: point.cost_usd,
    tokens: point.total_tokens,
  }));

  return (
    <div className="rounded-lg border bg-card p-4 shadow-sm">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold">{t('dashboard.costTrend')}</h3>
        <div className="inline-flex rounded border bg-muted/50 p-0.5">
          {Object.entries(GRANULARITIES).map(([, value]) => (
            <button
              key={value}
              onClick={() => setGranularity(value)}
              className={cn(
                'px-2 py-1 text-xs font-medium rounded transition-colors',
                granularity === value
                  ? 'bg-accent text-accent-foreground shadow-sm'
                  : 'text-muted-foreground hover:text-foreground'
              )}
            >
              {t(`granularity.${value}`)}
            </button>
          ))}
        </div>
      </div>
      <div className="h-64">
        {data.length === 0 ? (
          <div className="flex h-full items-center justify-center text-muted-foreground">
            {t('dashboard.noData')}
          </div>
        ) : (
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={data}>
              <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
              <XAxis
                dataKey="time"
                tick={{ fontSize: 12 }}
                tickFormatter={(v) => v.slice(-10)}
              />
              <YAxis
                tick={{ fontSize: 12 }}
                tickFormatter={(v) => `$${v.toFixed(2)}`}
              />
              <Tooltip
                formatter={(value) => [formatUsd(Number(value)), t('dashboard.cost')]}
                labelFormatter={(label) => `${t('table.time')}: ${label}`}
              />
              <Line
                type="monotone"
                dataKey="cost"
                stroke={CHART_COLORS[0]}
                strokeWidth={2}
                dot={false}
              />
            </LineChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );
}
