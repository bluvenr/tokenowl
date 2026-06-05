import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { formatDateTime, formatUsd, formatLatency, formatTokens } from '@/lib/format';
import { CheckCircle, XCircle, Clock } from 'lucide-react';

export function RecentSessions() {
  const { t } = useTranslation();
  const sessions = useDashboardStore((s) => s.sessions);

  if (sessions.length === 0) {
    return (
      <div className="rounded-lg border bg-card p-4 shadow-sm">
        <h3 className="font-semibold mb-4">{t('dashboard.recentRequests')}</h3>
        <div className="flex h-32 items-center justify-center text-muted-foreground">
          {t('dashboard.noData')}
        </div>
      </div>
    );
  }

  return (
    <div className="rounded-lg border bg-card p-4 shadow-sm">
      <h3 className="font-semibold mb-4">{t('dashboard.recentRequests')}</h3>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b">
              <th className="text-left py-2 px-2 font-medium text-muted-foreground whitespace-nowrap">
                {t('table.status')}
              </th>
              <th className="text-left py-2 px-2 font-medium text-muted-foreground whitespace-nowrap">
                {t('table.model')}
              </th>
              <th className="text-left py-2 px-2 font-medium text-muted-foreground whitespace-nowrap">
                {t('table.provider')}
              </th>
              <th className="text-right py-2 px-2 font-medium text-muted-foreground whitespace-nowrap">
                {t('table.input')}
              </th>
              <th className="text-right py-2 px-2 font-medium text-muted-foreground whitespace-nowrap">
                {t('table.output')}
              </th>
              <th className="text-right py-2 px-2 font-medium text-muted-foreground whitespace-nowrap">
                {t('table.total')}
              </th>
              <th className="text-right py-2 px-2 font-medium text-muted-foreground whitespace-nowrap">
                {t('table.cost')}
              </th>
              <th className="text-right py-2 px-2 font-medium text-muted-foreground whitespace-nowrap">
                {t('table.latency')}
              </th>
              <th className="text-right py-2 px-2 font-medium text-muted-foreground whitespace-nowrap">
                {t('table.time')}
              </th>
            </tr>
          </thead>
          <tbody>
            {sessions.map((session) => (
              <tr key={session.id} className="border-b last:border-0 hover:bg-muted/50">
                <td className="py-2 px-2">
                  {session.status_code && session.status_code < 400 ? (
                    <CheckCircle className="h-4 w-4 text-green-500" />
                  ) : session.status_code ? (
                    <XCircle className="h-4 w-4 text-red-500" />
                  ) : (
                    <Clock className="h-4 w-4 text-muted-foreground" />
                  )}
                </td>
                <td className="py-2 px-2 font-mono text-xs max-w-[180px] truncate" title={session.model}>
                  {session.model}
                </td>
                <td className="py-2 px-2 text-muted-foreground max-w-[120px] truncate" title={session.provider_name || ''}>
                  {session.provider_name || '-'}
                </td>
                <td className="py-2 px-2 text-right font-mono text-xs">
                  {formatTokens(session.input_tokens)}
                </td>
                <td className="py-2 px-2 text-right font-mono text-xs">
                  {formatTokens(session.output_tokens)}
                </td>
                <td className="py-2 px-2 text-right font-mono text-xs font-medium">
                  {formatTokens(session.total_tokens)}
                </td>
                <td className="py-2 px-2 text-right whitespace-nowrap">
                  {session.cost_usd != null
                    ? formatUsd(session.cost_usd)
                    : '-'}
                </td>
                <td className="py-2 px-2 text-right text-muted-foreground whitespace-nowrap">
                  {session.response_time_ms != null
                    ? formatLatency(session.response_time_ms)
                    : '-'}
                </td>
                <td className="py-2 px-2 text-right text-muted-foreground text-xs whitespace-nowrap">
                  {formatDateTime(session.timestamp)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
