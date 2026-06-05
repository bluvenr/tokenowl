import { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from 'react-i18next';
import { Dashboard } from '@/pages/Dashboard';
import { Settings } from '@/pages/Settings';
import { TrayPopup } from '@/pages/TrayPopup';
import { useAppStore } from '@/stores/app';
import { showTrayPopup } from '@/lib/tauri';
import { cn } from '@/lib/utils';
import { PanelBottom } from 'lucide-react';

function App() {
  const { t } = useTranslation();
  const [windowLabel, setWindowLabel] = useState<string>('');
  const currentPage = useAppStore((s) => s.currentPage);
  const setCurrentPage = useAppStore((s) => s.setCurrentPage);
  const theme = useAppStore((s) => s.theme);
  const setUpdateAvailable = useAppStore((s) => s.setUpdateAvailable);

  useEffect(() => {
    const win = getCurrentWindow();
    setWindowLabel(win.label);
  }, []);

  // Apply theme class to html element
  useEffect(() => {
    document.documentElement.classList.toggle('dark', theme === 'dark');
  }, [theme]);

  // Listen for navigation events from tray popup or menu
  useEffect(() => {
    if (windowLabel === 'tray') return;
    const unlisten = listen<string>('navigate', (event) => {
      if (event.payload === 'dashboard' || event.payload === 'settings') {
        setCurrentPage(event.payload);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [windowLabel, setCurrentPage]);

  // Listen for auto-update events from Rust backend
  useEffect(() => {
    if (windowLabel === 'tray') return;
    const unlisten = listen<{
      current: string;
      latest: string;
      release_url: string | null;
      changelog: string | null;
    }>('update-available', (event) => {
      setUpdateAvailable({
        latest: event.payload.latest,
        releaseUrl: event.payload.release_url,
        changelog: event.payload.changelog,
      });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [windowLabel, setUpdateAvailable]);

  if (windowLabel === 'tray') {
    return <TrayPopup />;
  }

  return (
    <div className="flex h-screen w-screen bg-background">
      {/* Sidebar */}
      <aside className="w-48 border-r bg-muted/30 p-4 flex flex-col">
        <div className="mb-6">
          <h1 className="text-lg font-bold">TokenOwl</h1>
          <p className="text-xs text-muted-foreground">{t('app.tagline')}</p>
        </div>
        <nav className="space-y-1 flex-1">
          <button
            onClick={() => setCurrentPage('dashboard')}
            className={cn(
              'w-full text-left px-3 py-2 rounded-md text-sm font-medium transition-colors',
              currentPage === 'dashboard'
                ? 'bg-primary/10 text-primary'
                : 'hover:bg-muted'
            )}
          >
            {t('nav.dashboard')}
          </button>
          <button
            onClick={() => setCurrentPage('settings')}
            className={cn(
              'w-full text-left px-3 py-2 rounded-md text-sm font-medium transition-colors',
              currentPage === 'settings'
                ? 'bg-primary/10 text-primary'
                : 'hover:bg-muted'
            )}
          >
            {t('nav.settings')}
          </button>
        </nav>
        {/* Tray popup button at bottom */}
        <div className="border-t pt-2 mt-2">
          <button
            onClick={showTrayPopup}
            className="w-full flex items-center gap-2 px-3 py-2 rounded-md text-sm text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
            title={t('tray.showPopup')}
          >
            <PanelBottom className="h-4 w-4" />
            <span>{t('tray.showPopup')}</span>
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-auto">
        {currentPage === 'dashboard' && <Dashboard />}
        {currentPage === 'settings' && <Settings />}
      </main>
    </div>
  );
}

export default App;
