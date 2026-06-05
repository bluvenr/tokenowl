import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CheckCircle2, AlertCircle, Database, HardDrive, Layers } from "lucide-react";
import {
  rebuildTrayMenu,
  type AppSettings,
  type DbStats,
  type CcSwitchStatus,
} from "@/lib/tauri";
import { LANGUAGES } from "@/lib/constants";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { showToast } from "@/components/budget/BudgetAlert";
import { SettingRow } from "./shared";

export function GeneralTab({
  settings,
  dbStats,
  ccSwitchStatus,
  onSave,
}: {
  settings: AppSettings;
  dbStats: DbStats | null;
  ccSwitchStatus: CcSwitchStatus | null;
  onSave: (s: AppSettings) => void;
}) {
  const { t } = useTranslation();
  const [local, setLocal] = useState(settings);

  function applyChange(update: Partial<AppSettings>) {
    const updated = { ...local, ...update };
    setLocal(updated);
    Promise.resolve(onSave(updated)).then(() => {
      if (update.language !== undefined) {
        rebuildTrayMenu(
          t("tray.open_dashboard"),
          t("tray.sync_data"),
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
    <div className="space-y-6">
      {/* App Preferences */}
      <Card className="border-border/60">
        <CardHeader className="pb-4">
          <CardTitle className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
            {t("settings.preferences")}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-5">
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

      {/* CC Switch Connection */}
      {ccSwitchStatus && (
        <Card className="border-border/60">
          <CardHeader className="pb-4">
            <CardTitle className="text-xs font-medium text-muted-foreground uppercase tracking-wide flex items-center gap-2">
              {ccSwitchStatus.detected ? (
                <CheckCircle2 className="w-3.5 h-3.5 text-green-500" />
              ) : (
                <AlertCircle className="w-3.5 h-3.5 text-amber-500" />
              )}
              {t("settings.ccswitch_title")}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-3">
              <InfoItem 
                label={t("settings.ccswitch_status")} 
                value={ccSwitchStatus.detected ? t("settings.ccswitch_detected") : t("settings.ccswitch_not_found")}
                highlight={ccSwitchStatus.detected}
              />
              <InfoItem 
                label={t("settings.ccswitch_records")} 
                value={String(ccSwitchStatus.recordCount)}
              />
              <InfoItem 
                label={t("settings.ccswitch_db_path")} 
                value={ccSwitchStatus.dbPath}
                truncate
              />
              <InfoItem 
                label={t("settings.ccswitch_last_sync")} 
                value={ccSwitchStatus.lastSyncAt ?? t("settings.ccswitch_never")}
              />
            </div>
          </CardContent>
        </Card>
      )}

      {/* Database Stats */}
      {dbStats && (
        <Card className="border-border/60">
          <CardHeader className="pb-4">
            <CardTitle className="text-xs font-medium text-muted-foreground uppercase tracking-wide flex items-center gap-2">
              <Database className="w-3.5 h-3.5" />
              {t("settings.db_stats")}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-3 gap-3">
              <StatCard 
                icon={<Layers className="w-4 h-4" />}
                label={t("settings.db_records")} 
                value={dbStats.recordCount.toLocaleString()}
              />
              <StatCard 
                icon={<HardDrive className="w-4 h-4" />}
                label={t("settings.db_size")} 
                value={formatBytes(dbStats.dbSizeBytes)}
              />
              <StatCard 
                icon={<Database className="w-4 h-4" />}
                label={t("dashboard.sessions")} 
                value={dbStats.sessionCount.toLocaleString()}
              />
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function InfoItem({ label, value, highlight, truncate }: { 
  label: string; 
  value: string; 
  highlight?: boolean;
  truncate?: boolean;
}) {
  return (
    <div className="flex items-center justify-between py-2.5 px-3.5 rounded-lg bg-muted/40">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className={`text-xs font-medium tabular-nums ${highlight ? "text-green-600 dark:text-green-400" : ""} ${truncate ? "truncate max-w-[140px]" : ""}`}>
        {value}
      </span>
    </div>
  );
}

function StatCard({ icon, label, value }: { 
  icon: React.ReactNode;
  label: string; 
  value: string;
}) {
  return (
    <div className="flex flex-col items-center justify-center py-5 px-3 rounded-lg bg-muted/40 text-center">
      <div className="text-muted-foreground mb-2">{icon}</div>
      <div className="text-lg font-bold tabular-nums">{value}</div>
      <div className="text-[10px] text-muted-foreground mt-1.5 uppercase tracking-wide">{label}</div>
    </div>
  );
}
