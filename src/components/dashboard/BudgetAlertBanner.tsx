import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { AlertTriangle, X } from 'lucide-react';
import { useState } from 'react';

export function BudgetAlertBanner() {
  const { t } = useTranslation();
  const budgetAlert = useDashboardStore((s) => s.budgetAlert);
  const [dismissed, setDismissed] = useState(false);

  if (!budgetAlert || !budgetAlert.triggered || dismissed) {
    return null;
  }

  return (
    <div className="border-b bg-orange-50 dark:bg-orange-950/30 px-6 py-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm">
          <AlertTriangle className="h-4 w-4 text-orange-500" />
          <span className="font-medium text-orange-800 dark:text-orange-300">
            {t('budget.warning')}
          </span>
          <span className="text-orange-700 dark:text-orange-400">
            {budgetAlert.message}
          </span>
        </div>
        <button
          onClick={() => setDismissed(true)}
          className="text-orange-500 hover:text-orange-700"
        >
          <X className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}
