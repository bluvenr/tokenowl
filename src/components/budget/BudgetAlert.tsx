import { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import {
  checkBudgetAlert,
  getBudgetConfig,
  sendNotification,
  type BudgetAlert as BudgetAlertType,
} from "@/lib/tauri";
import { Card, CardContent } from "@/components/ui/card";

// Module-level shared dismiss state using a Set for proper listener management
let budgetAlertDismissed = false;
const dismissListeners = new Set<() => void>();

/** Reset dismiss state (useful for testing or re-triggering alerts) */
export function resetBudgetAlertDismissed() {
  budgetAlertDismissed = false;
}

export function BudgetAlertBanner() {
  const { t } = useTranslation();
  const [alert, setAlert] = useState<BudgetAlertType | null>(null);
  const [dismissed, setDismissed] = useState(budgetAlertDismissed);

  // Sync with module-level dismiss state across instances
  useEffect(() => {
    const handler = () => setDismissed(true);
    dismissListeners.add(handler);
    return () => {
      dismissListeners.delete(handler);
    };
  }, []);

  useEffect(() => {
    checkBudgetAlert()
      .then(async (result) => {
        if (result?.triggered) {
          setAlert(result);

          // Send system notification if enabled in budget config
          try {
            const config = await getBudgetConfig();
            if (config.alertSystemNotify) {
              await sendNotification(
                t("budget.alert_title"),
                result.message
              );
            }
          } catch (_) {
            // silently ignore notification errors
          }
        }
      })
      .catch(() => {});
  }, []);

  if (!alert || dismissed) return null;

  return (
    <Card className="border-destructive/50 bg-destructive/5">
      <CardContent className="p-3 flex items-center gap-3">
        <div className="text-destructive text-lg">⚠</div>
        <div className="flex-1">
          <div className="text-sm font-medium">{t("budget.alert_title")}</div>
          <div className="text-xs text-muted-foreground">{alert.message}</div>
        </div>
        <button
          aria-label={t("announcement.dismiss")}
          onClick={() => {
            budgetAlertDismissed = true;
            setDismissed(true);
            dismissListeners.forEach((fn) => fn());
          }}
          className="text-muted-foreground hover:text-foreground text-sm px-2"
        >
          ×
        </button>
      </CardContent>
    </Card>
  );
}

// ─── Toast Notification Component ──────────────────────────────────────

interface Toast {
  id: number;
  title: string;
  message: string;
  type: "info" | "warning" | "success" | "error";
}

let toastId = 0;
const listeners = new Set<(toast: Toast) => void>();

export function showToast(title: string, message: string, type: Toast["type"] = "info") {
  const toast: Toast = { id: ++toastId, title, message, type };
  listeners.forEach((fn) => fn(toast));
}

export function ToastContainer() {
  const { t } = useTranslation();
  const [toasts, setToasts] = useState<Toast[]>([]);
  const timersRef = useRef<Map<number, ReturnType<typeof setTimeout>>>(new Map());

  const addToast = useCallback((toast: Toast) => {
    setToasts((prev) => [...prev, toast]);
    // Auto-dismiss after 5 seconds, tracking the timer for cleanup
    const timerId = setTimeout(() => {
      timersRef.current.delete(toast.id);
      setToasts((prev) => prev.filter((t) => t.id !== toast.id));
    }, 5000);
    timersRef.current.set(toast.id, timerId);
  }, []);

  // Clean up all pending timers on unmount
  useEffect(() => {
    listeners.add(addToast);
    return () => {
      listeners.delete(addToast);
      timersRef.current.forEach((timer) => clearTimeout(timer));
      timersRef.current.clear();
    };
  }, [addToast]);

  if (toasts.length === 0) return null;

  const typeStyles: Record<string, string> = {
    info: "border-blue-200 bg-blue-50 dark:border-blue-800 dark:bg-blue-950",
    warning: "border-amber-200 bg-amber-50 dark:border-amber-800 dark:bg-amber-950",
    success: "border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950",
    error: "border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-950",
  };

  return (
    <div className="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 max-w-sm">
      {toasts.map((toast) => (
        <Card
          key={toast.id}
          className={`border shadow-lg animate-in slide-in-from-bottom-2 ${typeStyles[toast.type] || typeStyles.info}`}
        >
          <CardContent className="p-3">
            <div className="flex items-start gap-2">
              <div className="flex-1">
                <div className="text-sm font-medium">{toast.title}</div>
                <div className="text-xs text-muted-foreground mt-0.5">{toast.message}</div>
              </div>
              <button
                aria-label={t("announcement.dismiss")}
                onClick={() => {
                  // Clear the auto-dismiss timer when manually dismissed
                  const timer = timersRef.current.get(toast.id);
                  if (timer) {
                    clearTimeout(timer);
                    timersRef.current.delete(toast.id);
                  }
                  setToasts((prev) => prev.filter((t) => t.id !== toast.id));
                }}
                className="text-muted-foreground hover:text-foreground text-xs px-1"
              >
                ×
              </button>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
