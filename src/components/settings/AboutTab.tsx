import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { ExternalLink, RefreshCw } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  getAppVersion,
  checkForUpdate,
  type UpdateInfo,
} from "@/lib/tauri";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { CrashLogViewer } from "@/components/crash/CrashLogViewer";

export function AboutTab() {
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
      <Card className="border-border/60">
        <CardContent className="pt-6">
          <div className="flex flex-col items-center text-center py-4">
            <img src="/logo.png" alt="TokenOwl" className="w-16 h-16 mb-4" />
            <h2 className="text-xl font-bold tracking-tight">TokenOwl</h2>
            <p className="text-sm text-muted-foreground mt-1">
              {t("about.description")}
            </p>
            <div className="mt-3 inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-muted text-xs font-medium tabular-nums">
              {t("about.version")} {version}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Check for Updates */}
      <Card className="border-border/60">
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
      <Card className="border-border/60">
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
