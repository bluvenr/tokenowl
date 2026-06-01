import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { ExternalLink, RefreshCw } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  getSettings,
  updateSettings,
  getSourceConfigs,
  updateSourceConfig,
  getAllPrices,
  updateCustomPrice,
  resetCustomPrice,
  deleteCustomPrice,
  getBudgetConfig,
  updateBudgetConfig,
  exportUsageCsv,
  exportUsageJson,
  rescan,
  getSourceStatus,
  getDbStats,
  getAppVersion,
  checkForUpdate,
  rebuildTrayMenu,
  type AppSettings,
  type SourceConfig,
  type ModelPricing,
  type BudgetConfig,
  type SourceStatus,
  type DbStats,
  type UpdateInfo,
} from "@/lib/tauri";
import { LANGUAGES, DATA_SOURCES } from "@/lib/constants";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { BudgetAlertBanner } from "@/components/budget/BudgetAlert";
import { showToast } from "@/components/budget/BudgetAlert";
import { CrashLogViewer } from "@/components/crash/CrashLogViewer";

type Tab = "general" | "data_source" | "pricing" | "budget" | "privacy" | "about";

export function Settings({ onSettingsSaved }: { onSettingsSaved?: () => void }) {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<Tab>("general");
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [sources, setSources] = useState<SourceConfig[]>([]);
  const [sourceStatuses, setSourceStatuses] = useState<SourceStatus[]>([]);
  const [prices, setPrices] = useState<ModelPricing[]>([]);
  const [budget, setBudget] = useState<BudgetConfig | null>(null);
  const [rescanning, setRescanning] = useState(false);
  const [dbStats, setDbStats] = useState<DbStats | null>(null);

  useEffect(() => {
    loadAll();
  }, []);

  async function loadAll() {
    try {
      const [s, src, prc, bdg, sts, stats] = await Promise.all([
        getSettings(),
        getSourceConfigs(),
        getAllPrices(),
        getBudgetConfig(),
        getSourceStatus(),
        getDbStats(),
      ]);
      setSettings(s);
      setSources(src);
      setPrices(prc);
      setBudget(bdg);
      setSourceStatuses(sts);
      setDbStats(stats);
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    }
  }

  async function handleRescan() {
    setRescanning(true);
    try {
      const count = await rescan();
      showToast(t("settings.rescan"), `${count} ${t("settings.records_found")}`, "success");
      await loadAll();
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    }
    setRescanning(false);
  }

  return (
    <div className="flex-1 flex flex-col min-h-0">
      {!settings && (
        <div className="text-center py-12 text-muted-foreground">
          {t("common.loading")}
        </div>
      )}

      {settings && (
        <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as Tab)} className="flex-1 flex flex-col min-h-0">
          {/* Sticky header + tab bar */}
          <div className="shrink-0 sticky top-0 bg-background z-10 border-b">
            <div className="max-w-3xl mx-auto w-full flex items-center justify-between px-6 pt-4 pb-3">
              <h1 className="text-2xl font-bold">{t("settings.title")}</h1>
              <Button
                variant="outline"
                size="sm"
                onClick={handleRescan}
                disabled={rescanning}
              >
                {rescanning ? t("settings.rescanning") : t("settings.rescan")}
              </Button>
            </div>

            <div className="max-w-3xl mx-auto w-full px-6 pb-3">
              <TabsList className="w-full h-10">
                <TabsTrigger value="general">{t("settings.general")}</TabsTrigger>
                <TabsTrigger value="data_source">{t("settings.data_source")}</TabsTrigger>
                <TabsTrigger value="pricing">{t("settings.pricing")}</TabsTrigger>
                <TabsTrigger value="budget">{t("settings.budget")}</TabsTrigger>
                <TabsTrigger value="privacy">{t("settings.privacy")}</TabsTrigger>
                <TabsTrigger value="about">{t("about.title")}</TabsTrigger>
              </TabsList>
            </div>
          </div>

          {/* Scrollable content area - full width so scrollbar sits at window edge */}
          <div className="flex-1 overflow-y-auto pb-10">
            <div className="max-w-3xl mx-auto px-6 mt-4">
              <BudgetAlertBanner />
            </div>

            <TabsContent value="general" className="max-w-3xl mx-auto px-6">
              <GeneralTab
                settings={settings}
                dbStats={dbStats}
                onSave={async (s) => {
                  await updateSettings(s);
                  setSettings(s);
                  onSettingsSaved?.();
                }}
              />
            </TabsContent>

            <TabsContent value="data_source" className="mt-4 max-w-3xl mx-auto px-6">
              <DataSourceTab
                sources={sources}
                statuses={sourceStatuses}
                onToggle={async (source, enabled) => {
                  try {
                    const src = sources.find((s) => s.source === source);
                    await updateSourceConfig(source, enabled, src?.customPath ?? null);
                    setSources((prev) =>
                      prev.map((s) => (s.source === source ? { ...s, enabled } : s))
                    );
                  } catch (e) {
                    showToast(t("common.error"), String(e), "error");
                  }
                }}
                onCustomPath={async (source, customPath) => {
                  try {
                    const src = sources.find((s) => s.source === source);
                    await updateSourceConfig(source, src?.enabled ?? true, customPath || null);
                    setSources((prev) =>
                      prev.map((s) => (s.source === source ? { ...s, customPath: customPath || null } : s))
                    );
                  } catch (e) {
                    showToast(t("common.error"), String(e), "error");
                  }
                }}
              />
            </TabsContent>

            <TabsContent value="pricing" className="mt-4 max-w-3xl mx-auto px-6">
              <PricingTab
                prices={prices}
                onUpdate={async (price) => {
                  await updateCustomPrice(price);
                  setPrices((prev) =>
                    prev.map((p) => (p.modelId === price.modelId ? price : p))
                  );
                  showToast(t("settings.save"), price.displayName, "success");
                }}
                onReset={async (modelId) => {
                  await resetCustomPrice(modelId);
                  setPrices((prev) =>
                    prev.map((p) =>
                      p.modelId === modelId ? { ...p, priceSource: "builtin" } : p
                    )
                  );
                }}
                onDelete={async (modelId) => {
                  await deleteCustomPrice(modelId);
                  setPrices((prev) => prev.filter((p) => p.modelId !== modelId));
                  showToast(t("settings.delete_model"), modelId, "info");
                }}
                onRefresh={loadAll}
              />
            </TabsContent>

            <TabsContent value="budget" className="mt-4 max-w-3xl mx-auto px-6">
              {budget && (
                <BudgetTab
                  budget={budget}
                  onSave={async (b) => {
                    await updateBudgetConfig(b);
                    setBudget(b);
                    onSettingsSaved?.();
                  }}
                />
              )}
            </TabsContent>

            <TabsContent value="privacy" className="mt-4 max-w-3xl mx-auto px-6">
              <PrivacyTab dbStats={dbStats} />
            </TabsContent>

            <TabsContent value="about" className="mt-4 max-w-3xl mx-auto px-6">
              <AboutTab />
            </TabsContent>
          </div>
        </Tabs>
      )}
    </div>
  );
}

// ─── General Tab ────────────────────────────────────────────────────

function GeneralTab({
  settings,
  dbStats,
  onSave,
}: {
  settings: AppSettings;
  dbStats: DbStats | null;
  onSave: (s: AppSettings) => void;
}) {
  const { t } = useTranslation();
  const [local, setLocal] = useState(settings);

  function applyChange(update: Partial<AppSettings>) {
    const updated = { ...local, ...update };
    setLocal(updated);
    Promise.resolve(onSave(updated)).then(() => {
      // Sync tray menu text when language changes
      if (update.language !== undefined) {
        rebuildTrayMenu(
          t("tray.open_dashboard"),
          t("tray.rescan"),
          t("tray.quit"),
        ).catch((e) => console.warn("Tray menu rebuild failed:", e));
      }
    }).catch((e) => {
      showToast(t("common.error"), String(e), "error");
    });
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardContent className="space-y-4 pt-6">
          <SettingRow label={t("settings.language")}>
            <Select
              value={local.language}
              onValueChange={(v) => applyChange({ language: v })}
            >
              <SelectTrigger className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {LANGUAGES.map((l) => (
                  <SelectItem key={l.key} value={l.key}>
                    {l.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </SettingRow>

          <SettingRow label={t("settings.theme")}>
            <Select
              value={local.theme}
              onValueChange={(v) => applyChange({ theme: v })}
            >
              <SelectTrigger className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="system">{t("settings.theme_system")}</SelectItem>
                <SelectItem value="light">{t("settings.theme_light")}</SelectItem>
                <SelectItem value="dark">{t("settings.theme_dark")}</SelectItem>
              </SelectContent>
            </Select>
          </SettingRow>

          <SettingRow label={t("settings.download_source")}>
            <Select
              value={local.downloadSource}
              onValueChange={(v) => applyChange({ downloadSource: v })}
            >
              <SelectTrigger className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="auto">{t("settings.download_auto")}</SelectItem>
                <SelectItem value="github">GitHub</SelectItem>
                <SelectItem value="gitee">Gitee</SelectItem>
              </SelectContent>
            </Select>
          </SettingRow>

          <SettingRow label={t("settings.auto_start")}>
            <Switch
              checked={local.autoStart}
              onCheckedChange={(v) => applyChange({ autoStart: v })}
            />
          </SettingRow>
        </CardContent>
      </Card>

      {/* Database Stats */}
      {dbStats && (
        <Card>
          <CardContent className="pt-6">
            <div className="text-sm font-medium mb-3">{t("settings.db_stats")}</div>
            <div className="grid grid-cols-2 gap-3">
              <StatItem label={t("settings.db_records")} value={dbStats.recordCount.toLocaleString()} />
              <StatItem label={t("settings.db_size")} value={formatBytes(dbStats.dbSizeBytes)} />
              <StatItem label={t("settings.data_source")} value={String(dbStats.sourceCount)} />
              <StatItem label={t("dashboard.sessions")} value={dbStats.sessionCount.toLocaleString()} />
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function StatItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between py-1.5 px-3 rounded-md bg-muted/50">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-sm font-medium tabular-nums">{value}</span>
    </div>
  );
}

// ─── Data Source Tab ────────────────────────────────────────────────

function DataSourceTab({
  sources,
  statuses,
  onToggle,
  onCustomPath,
}: {
  sources: SourceConfig[];
  statuses: SourceStatus[];
  onToggle: (source: string, enabled: boolean) => void;
  onCustomPath: (source: string, path: string) => void;
}) {
  const { t } = useTranslation();
  const [editingPath, setEditingPath] = useState<string | null>(null);
  const [pathValue, setPathValue] = useState("");

  function getStatus(source: string) {
    return statuses.find((s) => s.source === source);
  }

  function startEditPath(source: string, currentPath: string | null) {
    setEditingPath(source);
    setPathValue(currentPath || "");
  }

  function savePath(source: string) {
    onCustomPath(source, pathValue.trim());
    setEditingPath(null);
  }

  return (
    <div className="space-y-2">
      {sources.map((s) => {
        const status = getStatus(s.source);
        const isAvailable = status?.available ?? s.available;
        const recordCount = status?.recordCount ?? 0;

        return (
          <Card key={s.source}>
            <CardContent className="py-3 space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div
                    className={`w-2.5 h-2.5 rounded-full ${
                      isAvailable ? "bg-green-500" : "bg-gray-300"
                    }`}
                  />
                  <div>
                    <div className="text-sm font-medium">
                      {status?.displayName || s.source}
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {isAvailable ? t("settings.available") : t("settings.unavailable")}
                      {recordCount > 0 && ` · ${recordCount} ${t("settings.record_count")}`}
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <span className="text-xs text-muted-foreground">
                    {s.enabled ? t("settings.enabled") : t("settings.disabled")}
                  </span>
                  <Switch
                    checked={s.enabled}
                    onCheckedChange={(v) => onToggle(s.source, v)}
                  />
                </div>
              </div>

              {/* Custom Path Section */}
              {editingPath === s.source ? (
                <div className="flex items-center gap-2 pl-5">
                  <Input
                    className="flex-1 h-7 text-xs"
                    value={pathValue}
                    onChange={(e) => setPathValue(e.target.value)}
                    placeholder={t("settings.custom_path_placeholder")}
                  />
                  <Button size="sm" variant="default" className="h-7 text-xs" onClick={() => savePath(s.source)}>
                    {t("settings.save")}
                  </Button>
                  <Button size="sm" variant="ghost" className="h-7 text-xs" onClick={() => setEditingPath(null)}>
                    {t("common.cancel")}
                  </Button>
                </div>
              ) : (
                <div className="flex items-center gap-2 pl-5">
                  <span className="text-[10px] text-muted-foreground flex-1 truncate">
                    {s.customPath || t("settings.custom_path")}
                  </span>
                  <button
                    className="text-[10px] text-blue-500 hover:text-blue-600"
                    onClick={() => startEditPath(s.source, s.customPath)}
                  >
                    {s.customPath ? t("common.edit") : "+"}
                  </button>
                  {s.customPath && (
                    <button
                      className="text-[10px] text-red-400 hover:text-red-500"
                      onClick={() => {
                        onCustomPath(s.source, "");
                      }}
                    >
                      ×
                    </button>
                  )}
                </div>
              )}
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}

// ─── Pricing Tab ────────────────────────────────────────────────────

function PricingTab({
  prices,
  onUpdate,
  onReset,
  onDelete,
  onRefresh,
}: {
  prices: ModelPricing[];
  onUpdate: (price: ModelPricing) => void;
  onReset: (modelId: string) => void;
  onDelete: (modelId: string) => void;
  onRefresh: () => void;
}) {
  const { t } = useTranslation();
  const [editing, setEditing] = useState<string | null>(null);
  const [editValues, setEditValues] = useState({ input: "", output: "" });
  const [showAddForm, setShowAddForm] = useState(false);

  function startEdit(p: ModelPricing) {
    setEditing(p.modelId);
    setEditValues({
      input: String(p.inputPerMillion),
      output: String(p.outputPerMillion),
    });
  }

  async function saveEdit(p: ModelPricing) {
    const inputVal = parseFloat(editValues.input);
    const outputVal = parseFloat(editValues.output);
    const updated: ModelPricing = {
      ...p,
      inputPerMillion: isNaN(inputVal) ? p.inputPerMillion : inputVal,
      outputPerMillion: isNaN(outputVal) ? p.outputPerMillion : outputVal,
      priceSource: "custom",
    };
    await onUpdate(updated);
    setEditing(null);
  }

  const sourceBadge = (source: string) => {
    const colors: Record<string, string> = {
      custom: "bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300",
      remote: "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300",
      builtin: "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400",
    };
    return (
      <span className={`text-[10px] px-1.5 py-0.5 rounded font-medium ${colors[source] || colors.builtin}`}>
        {t(`settings.source_${source}`)}
      </span>
    );
  };

  return (
    <div className="space-y-2">
      {/* Add Custom Model Button */}
      {!showAddForm && (
        <Button
          variant="outline"
          size="sm"
          className="w-full text-xs"
          onClick={() => setShowAddForm(true)}
        >
          + {t("settings.add_custom_model")}
        </Button>
      )}

      {/* Add Custom Model Form */}
      {showAddForm && (
        <AddCustomModelForm
          onSave={async (price) => {
            await onUpdate(price);
            setShowAddForm(false);
            onRefresh();
          }}
          onCancel={() => setShowAddForm(false)}
        />
      )}

      {/* Existing Price List */}
      {prices.map((p) => (
        <Card key={p.modelId}>
          <CardContent className="flex items-center gap-3 py-2.5">
            <span className="flex-1 text-sm font-medium truncate">{p.displayName}</span>
            {editing === p.modelId ? (
              <div className="flex items-center gap-2">
                <span className="text-[10px] text-muted-foreground">In:</span>
                <Input
                  className="w-20 h-7 text-xs"
                  value={editValues.input}
                  onChange={(e) => setEditValues({ ...editValues, input: e.target.value })}
                />
                <span className="text-[10px] text-muted-foreground">Out:</span>
                <Input
                  className="w-20 h-7 text-xs"
                  value={editValues.output}
                  onChange={(e) => setEditValues({ ...editValues, output: e.target.value })}
                />
                <Button size="sm" variant="default" className="h-7 text-xs" onClick={() => saveEdit(p)}>
                  {t("settings.save")}
                </Button>
                <Button size="sm" variant="ghost" className="h-7 text-xs" onClick={() => setEditing(null)}>
                  {t("common.cancel")}
                </Button>
              </div>
            ) : (
              <>
                <span className="text-xs text-muted-foreground tabular-nums">
                  ${p.inputPerMillion}/M
                </span>
                <span className="text-xs text-muted-foreground tabular-nums">
                  ${p.outputPerMillion}/M
                </span>
                {sourceBadge(p.priceSource)}
                <Button
                  size="sm"
                  variant="ghost"
                  className="h-7 text-xs px-2"
                  onClick={() => startEdit(p)}
                >
                  {t("common.edit")}
                </Button>
                {p.priceSource === "custom" && (
                  <>
                    <Button
                      size="sm"
                      variant="ghost"
                      className="h-7 text-xs px-2"
                      onClick={() => onReset(p.modelId)}
                    >
                      {t("settings.reset_default")}
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      className="h-7 text-xs px-2 text-red-500 hover:text-red-600"
                      onClick={() => onDelete(p.modelId)}
                    >
                      {t("settings.delete_model")}
                    </Button>
                  </>
                )}
              </>
            )}
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

// ─── Add Custom Model Form ──────────────────────────────────────────

function AddCustomModelForm({
  onSave,
  onCancel,
}: {
  onSave: (price: ModelPricing) => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  const [form, setForm] = useState({
    modelId: "",
    displayName: "",
    source: "claude_code",
    inputPerMillion: "",
    outputPerMillion: "",
    cacheWritePerMillion: "",
    cacheReadPerMillion: "",
  });

  function handleSave() {
    if (!form.modelId.trim() || !form.displayName.trim()) return;
    onSave({
      modelId: form.modelId.trim(),
      displayName: form.displayName.trim(),
      source: form.source,
      inputPerMillion: parseFloat(form.inputPerMillion) || 0,
      outputPerMillion: parseFloat(form.outputPerMillion) || 0,
      cacheWritePerMillion: form.cacheWritePerMillion ? parseFloat(form.cacheWritePerMillion) : null,
      cacheReadPerMillion: form.cacheReadPerMillion ? parseFloat(form.cacheReadPerMillion) : null,
      priceSource: "custom",
    });
  }

  return (
    <Card className="border-dashed border-2">
      <CardContent className="space-y-3 pt-4">
        <div className="text-sm font-medium">{t("settings.add_custom_model")}</div>

        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="text-[10px] text-muted-foreground">{t("settings.model_id")}</label>
            <Input
              className="h-7 text-xs"
              value={form.modelId}
              onChange={(e) => setForm({ ...form, modelId: e.target.value })}
              placeholder={t("settings.model_id_placeholder")}
            />
          </div>
          <div>
            <label className="text-[10px] text-muted-foreground">{t("settings.display_name")}</label>
            <Input
              className="h-7 text-xs"
              value={form.displayName}
              onChange={(e) => setForm({ ...form, displayName: e.target.value })}
              placeholder={t("settings.display_name_placeholder")}
            />
          </div>
        </div>

        <div>
          <label className="text-[10px] text-muted-foreground">{t("settings.data_source")}</label>
          <Select
            value={form.source}
            onValueChange={(v) => setForm({ ...form, source: v })}
          >
            <SelectTrigger className="h-7 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {DATA_SOURCES.map((ds) => (
                <SelectItem key={ds.key} value={ds.key}>
                  {ds.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="text-[10px] text-muted-foreground">{t("settings.input_price")}</label>
            <Input
              className="h-7 text-xs"
              type="number"
              step="0.01"
              value={form.inputPerMillion}
              onChange={(e) => setForm({ ...form, inputPerMillion: e.target.value })}
              placeholder={t("settings.price_placeholder")}
            />
          </div>
          <div>
            <label className="text-[10px] text-muted-foreground">{t("settings.output_price")}</label>
            <Input
              className="h-7 text-xs"
              type="number"
              step="0.01"
              value={form.outputPerMillion}
              onChange={(e) => setForm({ ...form, outputPerMillion: e.target.value })}
              placeholder={t("settings.price_placeholder")}
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="text-[10px] text-muted-foreground">{t("settings.cache_write_price")}</label>
            <Input
              className="h-7 text-xs"
              type="number"
              step="0.01"
              value={form.cacheWritePerMillion}
              onChange={(e) => setForm({ ...form, cacheWritePerMillion: e.target.value })}
              placeholder={t("common.optional")}
            />
          </div>
          <div>
            <label className="text-[10px] text-muted-foreground">{t("settings.cache_read_price")}</label>
            <Input
              className="h-7 text-xs"
              type="number"
              step="0.01"
              value={form.cacheReadPerMillion}
              onChange={(e) => setForm({ ...form, cacheReadPerMillion: e.target.value })}
              placeholder={t("common.optional")}
            />
          </div>
        </div>

        <div className="flex gap-2 pt-1">
          <Button size="sm" className="h-7 text-xs" onClick={handleSave}>
            {t("settings.save")}
          </Button>
          <Button size="sm" variant="ghost" className="h-7 text-xs" onClick={onCancel}>
            {t("common.cancel")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

// ─── Budget Tab ─────────────────────────────────────────────────────

function BudgetTab({
  budget,
  onSave,
}: {
  budget: BudgetConfig;
  onSave: (b: BudgetConfig) => void;
}) {
  const { t } = useTranslation();
  const [local, setLocal] = useState(budget);

  function applyChange(update: Partial<BudgetConfig>) {
    const updated = { ...local, ...update };
    setLocal(updated);
    Promise.resolve(onSave(updated)).catch((e) => {
      showToast(t("common.error"), String(e), "error");
    });
  }

  async function handleSave() {
    try {
      await onSave(local);
      showToast(t("settings.save"), t("settings.saved"), "success");
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    }
  }

  return (
    <Card>
      <CardContent className="space-y-4 pt-6">
        <BudgetInput
          label={t("settings.daily_budget")}
          value={local.dailyLimitUsd}
          onChange={(v) => setLocal({ ...local, dailyLimitUsd: v })}
        />
        <BudgetInput
          label={t("settings.weekly_budget")}
          value={local.weeklyLimitUsd}
          onChange={(v) => setLocal({ ...local, weeklyLimitUsd: v })}
        />
        <BudgetInput
          label={t("settings.monthly_budget")}
          value={local.monthlyLimitUsd}
          onChange={(v) => setLocal({ ...local, monthlyLimitUsd: v })}
        />

        <SettingRow label={t("settings.alert_threshold")}>
          <div className="flex items-center gap-2">
            <Input
              type="number"
              className="w-20 h-8 text-sm"
              value={local.alertThresholdPct}
              onChange={(e) =>
                setLocal({ ...local, alertThresholdPct: parseInt(e.target.value) || 80 })
              }
              min={10}
              max={100}
            />
            <span className="text-sm text-muted-foreground">%</span>
          </div>
        </SettingRow>

        <div className="flex items-center gap-3 pt-2">
          <Button onClick={handleSave}>{t("settings.save")}</Button>
        </div>

        <div className="border-t pt-4 mt-2 space-y-3">
          <SettingRow label={t("settings.alert_icon_color")}>
            <Switch
              checked={local.alertIconColor}
              onCheckedChange={(v) => applyChange({ alertIconColor: v })}
            />
          </SettingRow>

          <SettingRow label={t("settings.alert_system_notify")}>
            <Switch
              checked={local.alertSystemNotify}
              onCheckedChange={(v) => applyChange({ alertSystemNotify: v })}
            />
          </SettingRow>
        </div>
      </CardContent>
    </Card>
  );
}

function BudgetInput({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number | null;
  onChange: (v: number | null) => void;
}) {
  const { t } = useTranslation();
  const hasValue = value !== null && value > 0;

  return (
    <SettingRow label={label}>
      <div className="flex items-center gap-2">
        {hasValue && <span className="text-sm text-muted-foreground">$</span>}
        <Input
          type="number"
          className="w-24 h-8 text-sm"
          value={hasValue ? value : ""}
          onChange={(e) => {
            const v = e.target.value;
            onChange(v === "" ? null : parseFloat(v));
          }}
          placeholder={t("settings.no_limit")}
          min={0}
          step={1}
        />
      </div>
    </SettingRow>
  );
}

// ─── Privacy Tab ────────────────────────────────────────────────────

function PrivacyTab({ dbStats }: { dbStats: DbStats | null }) {
  const { t } = useTranslation();

  async function handleExportCsv() {
    try {
      const csv = await exportUsageCsv("all");
      downloadFile(csv, "tokenowl-export.csv", "text/csv");
      showToast(t("common.export"), t("common.export_csv"), "success");
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    }
  }

  async function handleExportJson() {
    try {
      const json = await exportUsageJson("all");
      downloadFile(json, "tokenowl-export.json", "application/json");
      showToast(t("common.export"), t("common.export_json"), "success");
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    }
  }

  function downloadFile(content: string, filename: string, type: string) {
    const blob = new Blob([content], { type });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  return (
    <Card>
      <CardContent className="space-y-4 pt-6">
        <div>
          <div className="text-sm font-medium mb-2">{t("common.export")}</div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={handleExportCsv}>
              {t("common.export_csv")}
            </Button>
            <Button variant="outline" size="sm" onClick={handleExportJson}>
              {t("common.export_json")}
            </Button>
          </div>
          {dbStats && (
            <div className="text-xs text-muted-foreground mt-2">
              {t("settings.db_records")}: {dbStats.recordCount.toLocaleString()} · {t("settings.db_size")}: {dbStats.dbSizeBytes < 1024 ? `${dbStats.dbSizeBytes} B` : `${(dbStats.dbSizeBytes / 1024).toFixed(1)} KB`}
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

// ─── About Tab ───────────────────────────────────────────────────────

function AboutTab() {
  const { t } = useTranslation();
  const [version, setVersion] = useState<string>("");
  const [checking, setChecking] = useState(false);
  const [updateResult, setUpdateResult] = useState<"idle" | "up_to_date" | "available" | "error">("idle");
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);

  useEffect(() => {
    getAppVersion().then(setVersion).catch(() => setVersion("unknown"));
  }, []);

  async function handleCheckUpdate() {
    setChecking(true);
    setUpdateResult("idle");
    setUpdateInfo(null);
    try {
      const result = await checkForUpdate();
      if (result) {
        setUpdateInfo(result);
        setUpdateResult("available");
      } else {
        setUpdateResult("up_to_date");
      }
    } catch {
      setUpdateResult("error");
    }
    setChecking(false);
  }

  async function handleDownload() {
    if (updateInfo?.downloadUrl) {
      await openUrl(updateInfo.downloadUrl);
    }
  }

  return (
    <div className="space-y-4">
      {/* Product Info */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex flex-col items-center text-center py-4">
            {/* App Icon */}
            <img src="/logo.png" alt="TokenOwl" className="w-16 h-16 mb-4" />

            {/* App Name */}
            <h2 className="text-xl font-bold tracking-tight">TokenOwl</h2>

            {/* Tagline */}
            <p className="text-sm text-muted-foreground mt-1">
              {t("about.description")}
            </p>

            {/* Version */}
            <div className="mt-3 inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-muted text-xs font-medium tabular-nums">
              {t("about.version")} {version}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Check for Updates */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-sm font-medium">{t("about.check_update")}</div>
              {updateResult === "up_to_date" && (
                <div className="text-xs text-green-600 dark:text-green-400 mt-1">
                  {t("about.up_to_date")}
                </div>
              )}
              {updateResult === "available" && updateInfo && (
                <div className="text-xs text-blue-600 dark:text-blue-400 mt-1">
                  {t("about.update_available", { version: updateInfo.newVersion })}
                </div>
              )}
              {updateResult === "error" && (
                <div className="text-xs text-red-500 mt-1">
                  {t("about.check_failed")}
                </div>
              )}
            </div>
            <div className="flex items-center gap-2">
              {updateResult === "available" && updateInfo ? (
                <Button size="sm" onClick={handleDownload}>
                  {t("update.update_now")}
                </Button>
              ) : (
                <Button
                  size="sm"
                  variant="outline"
                  onClick={handleCheckUpdate}
                  disabled={checking}
                >
                  <RefreshCw className={`w-3.5 h-3.5 mr-1.5 ${checking ? "animate-spin" : ""}`} />
                  {checking ? t("about.checking_update") : t("about.check_update")}
                </Button>
              )}
            </div>
          </div>

          {/* Changelog (when update available) */}
          {updateResult === "available" && updateInfo?.notes && (
            <div className="mt-3 rounded-md bg-muted p-3 max-h-32 overflow-y-auto">
              <div className="text-xs font-medium mb-1">{t("update.changelog")}</div>
              <pre className="text-xs whitespace-pre-wrap font-sans text-muted-foreground">
                {updateInfo.notes}
              </pre>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Links */}
      <Card>
        <CardContent className="pt-6 space-y-1">
          <button
            className="flex items-center justify-between w-full py-2 px-1 rounded-md hover:bg-muted/50 transition-colors text-sm"
            onClick={() => openUrl("https://github.com/bluvenr/tokenowl")}
          >
            <span>{t("about.github")}</span>
            <ExternalLink className="w-3.5 h-3.5 text-muted-foreground" />
          </button>
          <div className="flex items-center justify-between py-2 px-1 text-sm">
            <span>{t("about.license")}</span>
            <span className="text-xs text-muted-foreground">{t("about.license_type")}</span>
          </div>
        </CardContent>
      </Card>

      {/* Copyright */}
      <div className="text-center text-xs text-muted-foreground pt-2">
        {t("about.copyright")}
      </div>

      {/* Crash Logs */}
      <CrashLogViewer />
    </div>
  );
}

// ─── Helper ─────────────────────────────────────────────────────────

function SettingRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between py-1">
      <span className="text-sm">{label}</span>
      {children}
    </div>
  );
}
