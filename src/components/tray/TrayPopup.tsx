import { useEffect, useRef, useState } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useAppStore } from "@/stores/appStore";
import { formatCost, formatTokens, getModelColor } from "@/lib/format";
import { getSettings } from "@/lib/tauri";
import { useTranslation } from "react-i18next";
import { X, Pin, Minimize2, LayoutDashboard } from "lucide-react";

export function TrayPopup() {
  const { refresh, summary, byModel, loading, error } = useAppStore();
  const { t, i18n } = useTranslation();

  useEffect(() => {
    refresh();
  }, [refresh]);

  // Apply theme to tray window (with system theme listener)
  useEffect(() => {
    function applyTheme(theme: string) {
      const root = document.documentElement;
      root.classList.remove("light", "dark");
      if (theme === "dark") {
        root.classList.add("dark");
      } else if (theme === "light") {
        root.classList.add("light");
      } else {
        const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
        root.classList.add(isDark ? "dark" : "light");
      }
    }

    let active = true;
    getSettings().then((s) => {
      if (!active) return;
      applyTheme(s.theme);
    }).catch(() => {});

    // Listen for OS theme changes in "system" mode
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      getSettings().then((s) => {
        if (active) applyTheme(s.theme);
      }).catch(() => {});
    };
    mq.addEventListener("change", handler);
    return () => { active = false; mq.removeEventListener("change", handler); };
  }, []);

  // Apply language to tray window
  useEffect(() => {
    getSettings().then((s) => {
      if (s.language && s.language !== "auto") {
        i18n.changeLanguage(s.language);
      } else {
        i18n.changeLanguage(navigator.language.startsWith("zh") ? "zh-CN" : "en");
      }
    }).catch(() => {});
  }, [i18n]);

  // Track drag state to prevent onFocusChanged from hiding window during drag
  const draggingRef = useRef(false);
  // Pin state: when pinned, window stays visible and on top even when unfocused
  const [pinned, setPinned] = useState(false);
  const pinnedRef = useRef(false);
  // Mini mode state: compact view showing only core data
  const [isMini, setIsMini] = useState(false);
  const isMiniRef = useRef(false);
  // Double-click detection for mini mode (startDragging captures mouse, so dblclick won't fire)
  const lastClickTimeRef = useRef(0);
  // Remember pin state before entering mini mode, restore on exit
  const prevPinnedRef = useRef(false);

  // Hide tray popup when it loses focus (but not during drag or when pinned)
  // Re-apply theme + language when it gains focus (sync with settings changes)
  useEffect(() => {
    const appWindow = getCurrentWindow();
    const unlisten = appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        // Re-read theme + language when window becomes visible
        getSettings().then((s) => {
          // Theme
          const root = document.documentElement;
          root.classList.remove("light", "dark");
          if (s.theme === "dark") {
            root.classList.add("dark");
          } else if (s.theme === "light") {
            root.classList.add("light");
          } else {
            const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
            root.classList.add(isDark ? "dark" : "light");
          }
          // Language
          if (s.language && s.language !== "auto") {
            i18n.changeLanguage(s.language);
          } else {
            i18n.changeLanguage(navigator.language.startsWith("zh") ? "zh-CN" : "en");
          }
        }).catch(() => {});
      } else if (!draggingRef.current && !pinnedRef.current) {
        appWindow.hide();
      }
    });
    return () => { unlisten.then(fn => fn()); };
  }, [i18n]);

  async function handleDrag(e: React.MouseEvent) {
    e.preventDefault();
    // In mini mode, detect double-click to expand (startDragging captures mouse so dblclick won't fire)
    if (isMiniRef.current) {
      const now = Date.now();
      if (now - lastClickTimeRef.current < 400) {
        lastClickTimeRef.current = 0;
        await toggleMini();
        return;
      }
      lastClickTimeRef.current = now;
    }
    draggingRef.current = true;
    try {
      await getCurrentWindow().startDragging();
    } catch {}
    setTimeout(() => { draggingRef.current = false; }, 300);
  }

  async function handleClose() {
    // Unpin when closing so next open starts unpinned
    pinnedRef.current = false;
    setPinned(false);
    try { await getCurrentWindow().setAlwaysOnTop(false); } catch {}
    // Reset mini mode so next open starts in full view
    isMiniRef.current = false;
    setIsMini(false);
    try { await getCurrentWindow().setSize(new LogicalSize(320, 400)); } catch {}
    await getCurrentWindow().hide();
  }

  async function togglePin() {
    const next = !pinned;
    setPinned(next);
    pinnedRef.current = next;
    try {
      await getCurrentWindow().setAlwaysOnTop(next);
    } catch {}
  }

  async function toggleMini() {
    const next = !isMini;
    try {
      await getCurrentWindow().setSize(
        next ? new LogicalSize(180, 80) : new LogicalSize(320, 400)
      );
    } catch {}
    // Entering mini: save current pin state, then auto-pin
    // Exiting mini: restore the pin state from before entering
    if (next) {
      prevPinnedRef.current = pinnedRef.current;
    }
    const shouldPin = next ? true : prevPinnedRef.current;
    try {
      await getCurrentWindow().setAlwaysOnTop(shouldPin);
    } catch {}
    pinnedRef.current = shouldPin;
    setPinned(shouldPin);
    setIsMini(next);
    isMiniRef.current = next;
  }

  async function handleOpenDashboard() {
    const mainWin = await WebviewWindow.getByLabel("main");
    if (mainWin) {
      await mainWin.show();
      await mainWin.setFocus();
    }
  }

  return (
    <div
      className={`w-full min-h-screen bg-background flex flex-col ${isMini ? "cursor-grab active:cursor-grabbing" : ""}`}
      onMouseDown={isMini ? handleDrag : undefined}
    >
      {/* Header: hidden in mini mode */}
      {!isMini && (
        <div
          className="flex items-center justify-between px-3 py-2.5 border-b border-border/60 cursor-grab active:cursor-grabbing select-none shrink-0"
          onMouseDown={handleDrag}
        >
          <div className="flex items-center gap-2">
            <img src="/favicon.png" alt="" className="w-4 h-4" />
            <span className="text-xs font-semibold tracking-tight">{t("app.name")}</span>
          </div>
          <div className="flex items-center gap-0.5">
            <button
              aria-label={t("tray.open_dashboard")}
              title={t("tray.open_dashboard")}
              className="p-1 rounded-md hover:bg-muted/60 text-muted-foreground hover:text-foreground transition-colors"
              onClick={handleOpenDashboard}
              onMouseDown={(e) => e.stopPropagation()}
            >
              <LayoutDashboard className="w-3.5 h-3.5" />
            </button>
            <button
              aria-label={t("tray.mini_mode")}
              title={t("tray.mini_mode")}
              className="p-1 rounded-md hover:bg-muted/60 text-muted-foreground hover:text-foreground transition-colors"
              onClick={toggleMini}
              onMouseDown={(e) => e.stopPropagation()}
            >
              <Minimize2 className="w-3.5 h-3.5" />
            </button>
            <button
              aria-label={pinned ? t("tray.unpin") : t("tray.pin")}
              title={pinned ? t("tray.unpin") : t("tray.pin")}
              className={`p-1 rounded-md transition-colors ${
                pinned
                  ? "bg-primary/10 text-primary hover:bg-primary/15"
                  : "text-muted-foreground hover:bg-muted/60 hover:text-foreground"
              }`}
              onClick={togglePin}
              onMouseDown={(e) => e.stopPropagation()}
            >
              <Pin className="w-3.5 h-3.5" fill={pinned ? "currentColor" : "none"} />
            </button>
            <button
              aria-label={t("common.close")}
              className="p-1 rounded-md hover:bg-muted/60 text-muted-foreground hover:text-foreground transition-colors"
              onClick={handleClose}
              onMouseDown={(e) => e.stopPropagation()}
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      )}

      {/* Content */}
      <div
        className={`flex-1 p-3 overflow-y-auto ${isMini ? "overflow-hidden" : "space-y-2.5"}`}
      >
        {isMini ? (
          /* Mini mode: ultra-compact, draggable, double-click to expand */
          <div className="flex flex-col items-center justify-center h-full select-none">
            <div className="text-[10px] text-muted-foreground uppercase tracking-wide leading-tight">{t("tray.today_cost")}</div>
            <div className="text-lg font-bold tracking-tight leading-snug tabular-nums">
              {summary ? formatCost(summary.totalCostUsd) : "$0.00"}
            </div>
            <div className="text-[9px] text-muted-foreground tabular-nums leading-tight">
              {summary ? formatTokens(summary.totalTokens) : "0"} {t("dashboard.tokens_label")}
              <span className="mx-0.5">·</span>
              {summary?.sessionCount ?? 0} {t("dashboard.sessions")}
            </div>
          </div>
        ) : (
          /* Full mode: show all details */
          <>
            {error && (
              <div className="text-center text-xs text-red-500 py-4">
                {error}
              </div>
            )}
            {/* Today's cost */}
            <div className="rounded-lg border border-border/60 bg-card p-3">
              <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-1">{t("tray.today_cost")}</div>
              <div className="text-2xl font-bold tracking-tight tabular-nums">
                {summary ? formatCost(summary.totalCostUsd) : "$0.00"}
              </div>
              <div className="mt-1.5 flex gap-3 text-[10px] text-muted-foreground tabular-nums">
                <span>{summary ? formatTokens(summary.totalTokens) : "0"} {t("dashboard.tokens_label")}</span>
                <span>{summary?.sessionCount ?? 0} {t("dashboard.sessions")}</span>
              </div>
            </div>

            {/* Per-model breakdown */}
            {byModel.length > 0 && (
              <div className="rounded-lg border border-border/60 bg-card p-3">
                <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-2">{t("dashboard.tool_breakdown")}</div>
                <div className="space-y-1.5">
                  {byModel.slice(0, 5).map((m, i) => (
                    <div key={m.model} className="flex items-center gap-2">
                      <div
                        className="w-2 h-2 rounded-full shrink-0"
                        style={{ backgroundColor: getModelColor(m.model, i) }}
                      />
                      <span className="text-[11px] flex-1 truncate">{m.model.length > 20 ? m.model.slice(0, 18) + "..." : m.model}</span>
                      <span className="text-[11px] font-medium tabular-nums">{formatCost(m.costUsd)}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {byModel.length === 0 && !loading && (
              <div className="text-center text-xs text-muted-foreground py-6">
                {t("dashboard.no_data")}
              </div>
            )}

            {loading && (
              <div className="text-center text-[10px] text-muted-foreground uppercase tracking-wide py-4">
                {t("common.loading")}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
