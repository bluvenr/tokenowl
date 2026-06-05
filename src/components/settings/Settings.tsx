import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCw } from "lucide-react";
import {
  getSettings,
  updateSettings,
  getAllPrices,
  updateCustomPrice,
  resetCustomPrice,
  deleteCustomPrice,
  getBudgetConfig,
  updateBudgetConfig,
  syncCcSwitch,
  getCcSwitchStatus,
  getDbStats,
  type AppSettings,
  type ModelPricing,
  type BudgetConfig,
  type CcSwitchStatus,
  type DbStats,
} from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { BudgetAlertBanner } from "@/components/budget/BudgetAlert";
import { showToast } from "@/components/budget/BudgetAlert";
import { GeneralTab } from "./GeneralTab";
import { PricingTab } from "./PricingTab";
import { BudgetTab } from "./BudgetTab";
import { PrivacyTab } from "./PrivacyTab";
import { AboutTab } from "./AboutTab";

type Tab = "general" | "pricing" | "budget" | "privacy" | "about";

export function Settings({ onSettingsSaved, initialTab, pricingPrefillSignal = 0 }: {
  onSettingsSaved?: () => void;
  initialTab?: Tab;
  pricingPrefillSignal?: number;
}) {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<Tab>("general");
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [prices, setPrices] = useState<ModelPricing[]>([]);
  const [budget, setBudget] = useState<BudgetConfig | null>(null);
  const [syncing, setSyncing] = useState(false);
  const [ccSwitchStatus, setCcSwitchStatus] = useState<CcSwitchStatus | null>(null);
  const [dbStats, setDbStats] = useState<DbStats | null>(null);

  useEffect(() => {
    loadAll();
  }, []);

  useEffect(() => {
    if (initialTab) {
      setActiveTab(initialTab);
    }
  }, [initialTab]);

  async function loadAll() {
    try {
      const [s, prc, bdg, sts, stats] = await Promise.all([
        getSettings(),
        getAllPrices(),
        getBudgetConfig(),
        getCcSwitchStatus(),
        getDbStats(),
      ]);
      setSettings(s);
      setPrices(prc);
      setBudget(bdg);
      setCcSwitchStatus(sts);
      setDbStats(stats);
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    }
  }

  async function handleSync() {
    setSyncing(true);
    try {
      const result = await syncCcSwitch();
      showToast(t("settings.sync_data"), `+${result.newRecords} ${t("settings.records_found")}`, "success");
      await loadAll();
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    }
    setSyncing(false);
  }

  return (
    <div className="flex-1 flex flex-col min-h-0">
      {!settings && (
        <div className="text-center py-16 text-muted-foreground">
          <div className="animate-pulse text-sm">{t("common.loading")}</div>
        </div>
      )}

      {settings && (
        <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as Tab)} className="flex-1 flex flex-col min-h-0">
          <div className="shrink-0 sticky top-0 bg-background/95 backdrop-blur-sm z-10 border-b border-border/60">
            <div className="max-w-3xl mx-auto w-full flex items-center justify-between px-6 pt-5 pb-4">
              <div>
                <h1 className="text-2xl font-bold tracking-tight">{t("settings.title")}</h1>
                <p className="text-sm text-muted-foreground mt-0.5">{t("settings.subtitle")}</p>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={handleSync}
                disabled={syncing}
                className="gap-2"
              >
                <RefreshCw className={`w-3.5 h-3.5 ${syncing ? "animate-spin" : ""}`} />
                {syncing ? t("settings.syncing") : t("settings.sync_data")}
              </Button>
            </div>

            <div className="max-w-3xl mx-auto w-full px-6 pb-3">
              <TabsList className="w-full h-10 bg-muted/50">
                <TabsTrigger value="general" className="text-xs">{t("settings.general")}</TabsTrigger>
                <TabsTrigger value="pricing" className="text-xs">{t("settings.pricing")}</TabsTrigger>
                <TabsTrigger value="budget" className="text-xs">{t("settings.budget")}</TabsTrigger>
                <TabsTrigger value="privacy" className="text-xs">{t("settings.privacy")}</TabsTrigger>
                <TabsTrigger value="about" className="text-xs">{t("about.title")}</TabsTrigger>
              </TabsList>
            </div>
          </div>

          <div className="flex-1 overflow-y-auto pb-10">
            <div className="max-w-3xl mx-auto px-6 mt-4">
              <BudgetAlertBanner />
            </div>

            <TabsContent value="general" className="max-w-3xl mx-auto px-6 mt-4">
              <GeneralTab
                settings={settings}
                dbStats={dbStats}
                ccSwitchStatus={ccSwitchStatus}
                onSave={async (s) => {
                  try {
                    await updateSettings(s);
                    setSettings(s);
                    onSettingsSaved?.();
                  } catch (e) {
                    showToast(t("common.error"), String(e), "error");
                  }
                }}
              />
            </TabsContent>

            <TabsContent value="pricing" className="max-w-3xl mx-auto px-6 mt-4">
              <PricingTab
                prices={prices}
                pricingPrefillSignal={pricingPrefillSignal}
                onUpdate={async (price) => {
                  try {
                    await updateCustomPrice(price);
                    showToast(t("settings.save"), price.displayName, "success");
                    await loadAll();
                  } catch (e) {
                    showToast(t("common.error"), String(e), "error");
                    throw e;
                  }
                }}
                onReset={async (modelId) => {
                  try {
                    await resetCustomPrice(modelId);
                    showToast(t("settings.reset_default"), t("settings.saved"), "success");
                    await loadAll();
                  } catch (e) {
                    showToast(t("common.error"), String(e), "error");
                  }
                }}
                onDelete={async (modelId) => {
                  try {
                    await deleteCustomPrice(modelId);
                    showToast(t("settings.delete_model"), modelId, "info");
                    await loadAll();
                  } catch (e) {
                    showToast(t("common.error"), String(e), "error");
                  }
                }}
              />
            </TabsContent>

            <TabsContent value="budget" className="max-w-3xl mx-auto px-6 mt-4">
              {budget && (
                <BudgetTab
                  budget={budget}
                  onSave={async (b) => {
                    try {
                      await updateBudgetConfig(b);
                      setBudget(b);
                      onSettingsSaved?.();
                    } catch (e) {
                      showToast(t("common.error"), String(e), "error");
                    }
                  }}
                />
              )}
            </TabsContent>

            <TabsContent value="privacy" className="max-w-3xl mx-auto px-6 mt-4">
              <PrivacyTab dbStats={dbStats} />
            </TabsContent>

            <TabsContent value="about" className="max-w-3xl mx-auto px-6 mt-4">
              <AboutTab />
            </TabsContent>
          </div>
        </Tabs>
      )}
    </div>
  );
}
