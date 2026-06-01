import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { LayoutDashboard, Settings as SettingsIcon, Monitor } from "lucide-react";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Dashboard } from "@/components/dashboard";
import { Settings } from "@/components/settings/Settings";
import { BudgetAlertBanner, ToastContainer } from "@/components/budget/BudgetAlert";
import { UpdateDialog } from "@/components/update/UpdateDialog";
import { AnnouncementBanner } from "@/components/announcement/AnnouncementBanner";
import { useRemoteServices } from "@/hooks/useUpdater";
import { useAppStore } from "@/stores/appStore";
import { getSettings, rebuildTrayMenu } from "@/lib/tauri";
import { Button } from "@/components/ui/button";

type Page = "dashboard" | "settings";

function App() {
  const [page, setPage] = useState<Page>("dashboard");
  const [theme, setTheme] = useState<string>("system");
  const { t, i18n } = useTranslation();
  const initEventListeners = useAppStore((s) => s.initEventListeners);

  // Initialize remote services (update checker, price sync, announcements)
  useRemoteServices();

  useEffect(() => {
    initEventListeners();
  }, [initEventListeners]);

  // Load theme + language from settings on mount
  useEffect(() => {
    getSettings().then((s) => {
      setTheme(s.theme);
      // M-FE-1: Apply saved language preference at startup
      if (s.language && s.language !== "auto") {
        i18n.changeLanguage(s.language);
      } else {
        // "auto" — detect from OS locale
        i18n.changeLanguage(navigator.language.startsWith("zh") ? "zh-CN" : "en");
      }
      // Sync tray menu text when language is "auto" (backend can't detect OS locale)
      if (s.language === "auto") {
        rebuildTrayMenu(
          t("tray.open_dashboard"),
          t("tray.rescan"),
          t("tray.quit"),
        ).catch((e) => console.warn("Tray menu rebuild failed:", e));
      }
    }).catch(() => {
      // silently ignore settings load errors on mount
    });
  }, [i18n]);

  // Callback for Settings page to notify App of saved changes
  const handleSettingsSaved = useCallback(() => {
    getSettings().then((s) => {
      setTheme(s.theme);
      if (s.language && s.language !== "auto") {
        i18n.changeLanguage(s.language);
      } else {
        // M-FE-6: "auto" — re-detect from OS locale
        i18n.changeLanguage(navigator.language.startsWith("zh") ? "zh-CN" : "en");
      }
    }).catch(() => {
      // silently ignore settings reload errors
    });
  }, [i18n]);

  // Apply theme class to html element whenever theme changes
  useEffect(() => {
    const root = document.documentElement;

    function applyTheme(t: string) {
      root.classList.remove("light", "dark");
      if (t === "dark") {
        root.classList.add("dark");
      } else if (t === "light") {
        root.classList.add("light");
      } else {
        // "system" — follow OS preference
        const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
        root.classList.add(isDark ? "dark" : "light");
      }
    }

    applyTheme(theme);

    // For "system" theme, listen for OS preference changes
    if (theme === "system") {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      const handler = () => applyTheme("system");
      mq.addEventListener("change", handler);
      return () => mq.removeEventListener("change", handler);
    }
  }, [theme]);

  // Re-read theme when switching to settings page (user may have changed it)
  const handlePageChange = useCallback((p: Page) => {
    setPage(p);
    if (p === "dashboard") {
      getSettings().then((s) => setTheme(s.theme)).catch(() => {});
    }
  }, []);

  // Tray popup visibility state
  const [trayVisible, setTrayVisible] = useState(false);

  useEffect(() => {
    let cleanupFn: (() => void) | undefined;
    (async () => {
      const trayWindow = await WebviewWindow.getByLabel("tray");
      if (trayWindow) {
        trayWindow.isVisible().then(setTrayVisible).catch(() => {});
        const unlisten1 = await trayWindow.onFocusChanged(() => {
          trayWindow.isVisible().then(setTrayVisible).catch(() => {});
        });
        cleanupFn = () => unlisten1();
      }
    })();
    return () => { cleanupFn?.(); };
  }, []);

  const handleToggleTray = useCallback(async () => {
    const trayWindow = await WebviewWindow.getByLabel("tray");
    if (!trayWindow) return;
    const visible = await trayWindow.isVisible();
    if (visible) {
      await trayWindow.hide();
    } else {
      await trayWindow.show();
      await trayWindow.setFocus();
    }
    setTrayVisible(!visible);
  }, []);

  return (
    <div className="h-screen flex flex-col bg-background">
      {/* Navigation bar */}
      <nav className="sticky top-0 z-50 border-b bg-background/95 backdrop-blur px-4 py-2 flex items-center gap-2">
        <div className="flex items-center gap-2 mr-auto">
          <img src="/favicon.png" alt="" className="w-5 h-5" />
          <h1 className="text-lg font-bold tracking-tight">{t("app.name")}</h1>
        </div>
        <Button
          variant={page === "dashboard" ? "default" : "ghost"}
          size="sm"
          onClick={() => handlePageChange("dashboard")}
          className="gap-1.5"
        >
          <LayoutDashboard className="w-4 h-4" />
          {t("dashboard.overview")}
        </Button>
        <Button
          variant={page === "settings" ? "default" : "ghost"}
          size="sm"
          onClick={() => handlePageChange("settings")}
          className="gap-1.5"
        >
          <SettingsIcon className="w-4 h-4" />
          {t("settings.title")}
        </Button>
        <div className="w-px h-5 bg-border mx-1" />
        <Button
          variant={trayVisible ? "default" : "ghost"}
          size="sm"
          onClick={handleToggleTray}
          title={trayVisible ? t("tray.hide_widget") : t("tray.show_widget")}
          className="gap-1.5"
        >
          <Monitor className="w-4 h-4" />
        </Button>
      </nav>

      {/* Announcement banner (shown on dashboard) */}
      {page === "dashboard" && (
        <div className="px-6 pt-3">
          <div className="max-w-5xl mx-auto space-y-2">
            <AnnouncementBanner />
            <BudgetAlertBanner />
          </div>
        </div>
      )}

      {/* Page content */}
      {page === "dashboard" && <Dashboard />}
      {page === "settings" && <Settings onSettingsSaved={handleSettingsSaved} />}

      {/* Update dialog (triggered by background update checker) */}
      <UpdateDialog />

      {/* Global toast notifications */}
      <ToastContainer />
    </div>
  );
}

export default App;
