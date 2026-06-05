import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { useAppStore } from '@/stores/app';
import { PeriodSelector } from '@/components/dashboard/PeriodSelector';
import { SummaryCards } from '@/components/dashboard/SummaryCards';
import { CostTrendChart } from '@/components/dashboard/CostTrendChart';
import { ModelBreakdown } from '@/components/dashboard/ModelBreakdown';
import { ProviderBreakdown } from '@/components/dashboard/ProviderBreakdown';
import { RecentSessions } from '@/components/dashboard/RecentSessions';
import { CcSwitchStatus } from '@/components/dashboard/CcSwitchStatus';
import { SavingsInsights } from '@/components/dashboard/SavingsInsights';
import { CostAnomalyCard } from '@/components/dashboard/CostAnomalyCard';
import { CostAttributionTree } from '@/components/dashboard/CostAttributionTree';
import { BudgetBurnRateCard } from '@/components/dashboard/BudgetBurnRateCard';
import { CacheTrendChart } from '@/components/dashboard/CacheTrendChart';
import { SyncButton } from '@/components/dashboard/SyncButton';
import { BudgetAlertBanner } from '@/components/dashboard/BudgetAlertBanner';
import { UpdateBanner } from '@/components/dashboard/UpdateBanner';

export function Dashboard() {
  const { t } = useTranslation();
  const fetchDashboardData = useDashboardStore((s) => s.fetchDashboardData);
  const initializePeriod = useDashboardStore((s) => s.initializePeriod);
  const settings = useAppStore((s) => s.settings);
  const settingsLoaded = useAppStore((s) => s.settingsLoaded);
  const loadSettings = useAppStore((s) => s.loadSettings);
  const loadVersion = useAppStore((s) => s.loadVersion);
  const loading = useDashboardStore((s) => s.loading);
  const error = useDashboardStore((s) => s.error);

  useEffect(() => {
    loadSettings();
    loadVersion();
  }, []);

  useEffect(() => {
    if (settingsLoaded) {
      initializePeriod(settings.default_period || 'week');
      fetchDashboardData();

      const interval = setInterval(() => {
        fetchDashboardData();
      }, 5 * 60 * 1000);

      return () => clearInterval(interval);
    }
  }, [settingsLoaded]);

  return (
    <div className="min-h-screen bg-background">
      <header className="sticky top-0 z-10 border-b bg-background/95 backdrop-blur">
        <div className="flex h-14 items-center justify-between px-6">
          <h1 className="text-lg font-semibold">{t('dashboard.title')}</h1>
          <div className="flex items-center gap-3">
            <PeriodSelector />
            <SyncButton />
          </div>
        </div>
      </header>

      <UpdateBanner />
      <BudgetAlertBanner />

      <main className="p-6 space-y-6">
        {error && (
          <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-sm text-destructive">
            {error}
          </div>
        )}

        {loading && !useDashboardStore.getState().summary ? (
          <div className="flex h-64 items-center justify-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
          </div>
        ) : (
          <>
            <SummaryCards />
            <CcSwitchStatus />
            <div className="grid gap-6 md:grid-cols-2">
              <CostTrendChart />
              <ModelBreakdown />
            </div>
            <div className="grid gap-6 md:grid-cols-2">
              <CacheTrendChart />
              <CostAnomalyCard />
            </div>
            <ProviderBreakdown />
            <SavingsInsights />
            <CostAttributionTree />
            <BudgetBurnRateCard />
            <RecentSessions />
          </>
        )}
      </main>
    </div>
  );
}
