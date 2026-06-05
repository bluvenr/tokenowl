import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { formatUsd, formatTokens } from '@/lib/format';
import { cn } from '@/lib/utils';
import { TrendingUp, TrendingDown, DollarSign, Zap, MessageSquare, Minus } from 'lucide-react';

function ChangeBadge({ value }: { value: number | null }) {
  const { t } = useTranslation();

  if (value === null) {
    return (
      <span className="inline-flex items-center gap-0.5 text-xs text-muted-foreground">
        <Minus className="h-3 w-3" />
        {t('dashboard.na')}
      </span>
    );
  }

  const isUp = value > 0;
  const isDown = value < 0;
  const absValue = Math.abs(value);

  if (!isUp && !isDown) {
    return (
      <span className="inline-flex items-center gap-0.5 text-xs text-muted-foreground">
        <Minus className="h-3 w-3" />
        0%
      </span>
    );
  }

  return (
    <span
      className={cn(
        'inline-flex items-center gap-0.5 text-xs font-medium',
        isUp ? 'text-red-500' : 'text-green-500',
      )}
    >
      {isUp ? <TrendingUp className="h-3 w-3" /> : <TrendingDown className="h-3 w-3" />}
      {isUp ? '+' : ''}{absValue.toFixed(1)}%
    </span>
  );
}

export function SummaryCards() {
  const { t } = useTranslation();
  const summary = useDashboardStore((s) => s.summary);
  const comparison = useDashboardStore((s) => s.periodComparison);

  if (!summary) return null;

  const cards = [
    {
      label: t('dashboard.totalCost'),
      value: formatUsd(summary.total_cost_usd),
      icon: DollarSign,
      color: 'text-green-500',
      change: comparison?.cost_change_pct ?? null,
    },
    {
      label: t('dashboard.totalTokens'),
      value: formatTokens(summary.total_tokens),
      icon: Zap,
      color: 'text-blue-500',
      change: comparison?.tokens_change_pct ?? null,
    },
    {
      label: t('dashboard.requests'),
      value: summary.request_count.toLocaleString(),
      icon: MessageSquare,
      color: 'text-purple-500',
      change: comparison?.requests_change_pct ?? null,
    },
    {
      label: t('dashboard.sessions'),
      value: summary.session_count.toLocaleString(),
      icon: TrendingUp,
      color: 'text-orange-500',
      change: comparison?.sessions_change_pct ?? null,
    },
  ];

  return (
    <div className="grid gap-4 md:grid-cols-4">
      {cards.map((card) => (
        <div
          key={card.label}
          className="rounded-lg border bg-card p-4 shadow-sm"
        >
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">{card.label}</span>
            <card.icon className={cn('h-4 w-4', card.color)} />
          </div>
          <div className="mt-2 text-2xl font-bold">{card.value}</div>
          <div className="mt-1">
            <ChangeBadge value={card.change} />
          </div>
        </div>
      ))}
    </div>
  );
}
