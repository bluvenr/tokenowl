import { useTranslation } from "react-i18next";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import {
  exportUsageCsv,
  exportUsageJson,
  type DbStats,
} from "@/lib/tauri";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { showToast } from "@/components/budget/BudgetAlert";

export function PrivacyTab({ dbStats }: { dbStats: DbStats | null }) {
  const { t } = useTranslation();

  async function handleExportCsv() {
    try {
      const csv = await exportUsageCsv("all");
      // Try native file save dialog first (Tauri dialog plugin)
      const filePath = await save({
        title: t("common.export_csv"),
        defaultPath: "tokenowl-export.csv",
        filters: [{ name: "CSV Files", extensions: ["csv"] }],
      });
      if (filePath) {
        await writeFile(filePath, new TextEncoder().encode(csv));
        showToast(t("common.export"), t("common.export_csv"), "success");
      } else {
        // Fallback to browser download if user cancels or dialog not available
        downloadFileBrowser(csv, "tokenowl-export.csv", "text/csv");
        showToast(t("common.export"), t("common.export_csv"), "success");
      }
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    }
  }

  async function handleExportJson() {
    try {
      const json = await exportUsageJson("all");
      // Try native file save dialog first (Tauri dialog plugin)
      const filePath = await save({
        title: t("common.export_json"),
        defaultPath: "tokenowl-export.json",
        filters: [{ name: "JSON Files", extensions: ["json"] }],
      });
      if (filePath) {
        await writeFile(filePath, new TextEncoder().encode(json));
        showToast(t("common.export"), t("common.export_json"), "success");
      } else {
        // Fallback to browser download if user cancels or dialog not available
        downloadFileBrowser(json, "tokenowl-export.json", "application/json");
        showToast(t("common.export"), t("common.export_json"), "success");
      }
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    }
  }

  function downloadFileBrowser(content: string, filename: string, type: string) {
    const blob = new Blob([content], { type });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  return (
    <Card className="border-border/60">
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
