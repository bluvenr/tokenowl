import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { RefreshCw } from 'lucide-react';
import { cn } from '@/lib/utils';

export function SyncButton() {
  const { t } = useTranslation();
  const syncing = useDashboardStore((s) => s.syncing);
  const syncData = useDashboardStore((s) => s.syncData);

  const handleSync = async () => {
    if (syncing) return;
    const result = await syncData();
    if (result && result.new_records > 0) {
      console.log(`Synced ${result.new_records} new records`);
    }
  };

  return (
    <button
      onClick={handleSync}
      disabled={syncing}
      className={cn(
        'inline-flex items-center gap-2 rounded-lg border bg-background px-3 py-1.5 text-sm font-medium transition-colors',
        'hover:bg-muted',
        syncing && 'opacity-50 cursor-not-allowed'
      )}
    >
      <RefreshCw className={cn('h-4 w-4', syncing && 'animate-spin')} />
      {syncing ? t('dashboard.syncing') : t('dashboard.syncData')}
    </button>
  );
}
