import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { formatUsd, formatPercent } from '@/lib/format';
import { CHART_COLORS } from '@/lib/constants';
import {
  PieChart,
  Pie,
  Cell,
  ResponsiveContainer,
  Tooltip,
  Legend,
} from 'recharts';

export function ModelBreakdown() {
  const { t } = useTranslation();
  const modelUsage = useDashboardStore((s) => s.modelUsage);

  const data = modelUsage.slice(0, 8).map((m) => ({
    name: m.model,
    value: m.cost_usd,
    percentage: m.percentage,
  }));

  return (
    <div className="rounded-lg border bg-card p-4 shadow-sm">
      <h3 className="font-semibold mb-4">{t('dashboard.modelBreakdown')}</h3>
      <div className="h-64">
        {data.length === 0 ? (
          <div className="flex h-full items-center justify-center text-muted-foreground">
            {t('dashboard.noData')}
          </div>
        ) : (
          <ResponsiveContainer width="100%" height="100%">
            <PieChart>
              <Pie
                data={data}
                cx="50%"
                cy="50%"
                innerRadius={60}
                outerRadius={80}
                paddingAngle={2}
                dataKey="value"
              >
                {data.map((_, index) => (
                  <Cell
                    key={`cell-${index}`}
                    fill={CHART_COLORS[index % CHART_COLORS.length]}
                  />
                ))}
              </Pie>
              <Tooltip
                content={({ payload }) => {
                  if (!payload || !payload.length) return null;
                  const data = payload[0].payload;
                  return (
                    <div className="rounded border bg-background p-2 shadow-md text-sm">
                      <div className="font-medium">{data.name}</div>
                      <div className="text-muted-foreground">
                        {formatUsd(data.value)} ({formatPercent(data.percentage)})
                      </div>
                    </div>
                  );
                }}
              />
              <Legend
                formatter={(value) => (
                  <span className="text-xs text-foreground">{value}</span>
                )}
              />
            </PieChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );
}
