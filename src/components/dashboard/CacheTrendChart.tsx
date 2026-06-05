import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { formatPercent } from '@/lib/format';
import { CHART_COLORS } from '@/lib/constants';
import { Database } from 'lucide-react';
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';

export function CacheTrendChart() {
  const { t } = useTranslation();
  const cacheTrend = useDashboardStore((s) => s.cacheTrend);

  const data = cacheTrend.map((point) => ({
    time: point.timestamp,
    hitRate: point.cache_hit_rate,
    cacheTokens: point.cache_tokens,
    totalTokens: point.total_tokens,
  }));

  const avgRate =
    data.length > 0
      ? data.reduce((sum, d) => sum + d.hitRate, 0) / data.length
      : 0;

  return (
    <div className="rounded-lg border bg-card p-4 shadow-sm">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <Database className="h-4 w-4 text-muted-foreground" />
          <h3 className="font-semibold">{t('cacheTrend.title')}</h3>
        </div>
        {data.length > 0 && (
          <span className="text-sm text-muted-foreground">
            {t('cacheTrend.avg')}: {formatPercent(avgRate)}
          </span>
        )}
      </div>
      <div className="h-40">
        {data.length === 0 ? (
          <div className="flex h-full items-center justify-center text-muted-foreground text-sm">
            {t('dashboard.noData')}
          </div>
        ) : (
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={data}>
              <defs>
                <linearGradient id="cacheHitGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor={CHART_COLORS[5]} stopOpacity={0.3} />
                  <stop offset="95%" stopColor={CHART_COLORS[5]} stopOpacity={0} />
                </linearGradient>
              </defs>
              <XAxis
                dataKey="time"
                tick={{ fontSize: 10 }}
                tickFormatter={(v) => v.slice(-10)}
                interval="preserveStartEnd"
              />
              <YAxis
                tick={{ fontSize: 10 }}
                tickFormatter={(v) => `${v.toFixed(0)}%`}
                domain={[0, 100]}
                width={40}
              />
              <Tooltip
                formatter={(value) => [formatPercent(Number(value)), t('cacheTrend.hitRate')]}
                labelFormatter={(label) => `${t('table.time')}: ${label}`}
              />
              <Area
                type="monotone"
                dataKey="hitRate"
                stroke={CHART_COLORS[5]}
                strokeWidth={2}
                fill="url(#cacheHitGrad)"
                dot={false}
              />
            </AreaChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );
}
