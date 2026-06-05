import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { cn } from '@/lib/utils';
import { CcSwitchTab } from '@/components/settings/CcSwitchTab';
import { AnalyticsTab } from '@/components/settings/AnalyticsTab';
import { BudgetAlertTab } from '@/components/settings/BudgetAlertTab';
import { GeneralTab } from '@/components/settings/GeneralTab';
import { AboutTab } from '@/components/settings/AboutTab';
import {
  Link,
  BarChart3,
  DollarSign,
  Settings as SettingsIcon,
  Info,
} from 'lucide-react';

type TabId = 'ccswitch' | 'analytics' | 'budget' | 'general' | 'about';

interface Tab {
  id: TabId;
  labelKey: string;
  icon: React.ComponentType<{ className?: string }>;
}

const TABS: Tab[] = [
  { id: 'ccswitch', labelKey: 'CC Switch', icon: Link },
  { id: 'analytics', labelKey: 'nav.analytics', icon: BarChart3 },
  { id: 'budget', labelKey: 'budget.title', icon: DollarSign },
  { id: 'general', labelKey: 'settings.general.title', icon: SettingsIcon },
  { id: 'about', labelKey: 'about.title', icon: Info },
];

export function Settings() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<TabId>('ccswitch');

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-10 border-b bg-background/95 backdrop-blur">
        <div className="flex h-14 items-center px-6">
          <h1 className="text-lg font-semibold">{t('settings.title')}</h1>
        </div>
      </header>

      <div className="flex h-[calc(100vh-3.5rem)]">
        {/* Tab Navigation */}
        <nav className="w-48 border-r p-4">
          <div className="space-y-1">
            {TABS.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={cn(
                  'w-full flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium transition-colors',
                  activeTab === tab.id
                    ? 'bg-primary/10 text-primary'
                    : 'hover:bg-muted text-muted-foreground hover:text-foreground'
                )}
              >
                <tab.icon className="h-4 w-4" />
                {tab.id === 'ccswitch' ? tab.labelKey : t(tab.labelKey)}
              </button>
            ))}
          </div>
        </nav>

        {/* Tab Content */}
        <div className="flex-1 overflow-auto p-6">
          {activeTab === 'ccswitch' && <CcSwitchTab />}
          {activeTab === 'analytics' && <AnalyticsTab />}
          {activeTab === 'budget' && <BudgetAlertTab />}
          {activeTab === 'general' && <GeneralTab />}
          {activeTab === 'about' && <AboutTab />}
        </div>
      </div>
    </div>
  );
}
