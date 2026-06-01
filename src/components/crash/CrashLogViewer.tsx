import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  AlertTriangle,
  Trash2,
  ExternalLink,
  ChevronDown,
  ChevronRight,
  Bug,
} from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  getCrashLogs,
  clearCrashLogs,
  getCrashIssueUrl,
  type CrashEntry,
} from "@/lib/tauri";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { showToast } from "@/components/budget/BudgetAlert";

export function CrashLogViewer() {
  const { t } = useTranslation();
  const [logs, setLogs] = useState<CrashEntry[]>([]);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadLogs();
  }, []);

  async function loadLogs() {
    try {
      const entries = await getCrashLogs();
      const sorted = [...entries].sort(
        (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime(),
      );
      setLogs(sorted);
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    } finally {
      setLoading(false);
    }
  }

  async function handleClearAll() {
    if (!window.confirm(t("crash.clear_confirm"))) return;
    try {
      await clearCrashLogs();
      setLogs([]);
      setExpandedId(null);
      showToast(t("crash.title"), t("crash.no_logs"), "success");
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    }
  }

  async function handleReport(id: string, e: React.MouseEvent) {
    e.stopPropagation();
    try {
      const url = await getCrashIssueUrl(id);
      await openUrl(url);
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    }
  }

  function formatTimestamp(ts: string): string {
    try {
      return new Date(ts).toLocaleString();
    } catch {
      return ts;
    }
  }

  if (loading) {
    return (
      <Card>
        <CardContent className="py-6 text-center text-sm text-muted-foreground">
          {t("common.loading")}
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-3">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Bug className="w-4 h-4 text-muted-foreground" />
          <h3 className="text-sm font-medium">{t("crash.title")}</h3>
          {logs.length > 0 && (
            <span className="text-xs text-muted-foreground tabular-nums">
              ({logs.length})
            </span>
          )}
        </div>
        {logs.length > 0 && (
          <Button
            variant="ghost"
            size="sm"
            className="h-7 text-xs text-red-500 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-950/30"
            onClick={handleClearAll}
          >
            <Trash2 className="w-3 h-3 mr-1" />
            {t("crash.clear_logs")}
          </Button>
        )}
      </div>

      {/* Empty state */}
      {logs.length === 0 && (
        <Card>
          <CardContent className="py-8 text-center">
            <AlertTriangle className="w-8 h-8 text-muted-foreground/50 mx-auto mb-2" />
            <p className="text-sm text-muted-foreground">{t("crash.no_logs")}</p>
          </CardContent>
        </Card>
      )}

      {/* Log entries */}
      <div className="space-y-2">
        {logs.map((log) => {
          const isExpanded = expandedId === log.id;
          return (
            <Card
              key={log.id}
              className={`cursor-pointer transition-colors ${
                isExpanded
                  ? "border-red-200 dark:border-red-900/50 bg-red-50/30 dark:bg-red-950/10"
                  : "hover:bg-muted/30"
              }`}
              onClick={() => setExpandedId(isExpanded ? null : log.id)}
            >
              <CardContent className="py-3">
                {/* Summary row */}
                <div className="flex items-start gap-2">
                  <div className="mt-0.5 shrink-0">
                    {isExpanded ? (
                      <ChevronDown className="w-3.5 h-3.5 text-muted-foreground" />
                    ) : (
                      <ChevronRight className="w-3.5 h-3.5 text-muted-foreground" />
                    )}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-0.5">
                      <span className="text-[10px] font-medium px-1.5 py-0.5 rounded bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300 truncate">
                        {log.errorType}
                      </span>
                      <span className="text-[10px] text-muted-foreground tabular-nums">
                        {formatTimestamp(log.timestamp)}
                      </span>
                    </div>
                    <p className="text-xs text-foreground/90 line-clamp-2">
                      {log.message}
                    </p>
                    <div className="flex items-center gap-2 mt-1 text-[10px] text-muted-foreground">
                      <span>v{log.appVersion}</span>
                      <span>&middot;</span>
                      <span className="truncate">{log.osInfo}</span>
                    </div>
                  </div>
                </div>

                {/* Expanded details */}
                {isExpanded && (
                  <div className="mt-3 pl-5 space-y-3">
                    {/* Report button */}
                    <Button
                      variant="outline"
                      size="sm"
                      className="h-7 text-xs"
                      onClick={(e) => handleReport(log.id, e)}
                    >
                      <ExternalLink className="w-3 h-3 mr-1" />
                      {t("crash.report_issue")}
                    </Button>

                    {/* Stack trace */}
                    {log.stackTrace && (
                      <div>
                        <div className="text-[10px] font-medium text-muted-foreground mb-1">
                          Stack Trace
                        </div>
                        <pre className="text-[11px] leading-relaxed whitespace-pre-wrap break-all rounded-md bg-muted p-2.5 max-h-48 overflow-y-auto font-mono text-foreground/80">
                          {log.stackTrace}
                        </pre>
                      </div>
                    )}

                    {/* Context */}
                    {log.context && Object.keys(log.context).length > 0 && (
                      <div>
                        <div className="text-[10px] font-medium text-muted-foreground mb-1">
                          Context
                        </div>
                        <pre className="text-[11px] leading-relaxed whitespace-pre-wrap break-all rounded-md bg-muted p-2.5 max-h-32 overflow-y-auto font-mono text-foreground/80">
                          {JSON.stringify(log.context, null, 2)}
                        </pre>
                      </div>
                    )}
                  </div>
                )}
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
