import { create } from 'zustand';
import {
  type AppSettings,
  type AppVersion,
  getSettings,
  updateSettings,
  getAppVersion,
  checkForUpdate,
  updateTrayMenu,
} from '@/lib/tauri';
import { DEFAULT_SETTINGS } from '@/lib/constants';
import i18n from '@/i18n';

/** Apply language setting to i18next, returns the resolved language */
function applyLanguage(language: string): string {
  const lng = language === 'auto'
    ? (navigator.language.startsWith('zh') ? 'zh-CN' : 'en')
    : language;
  i18n.changeLanguage(lng);
  return lng;
}

interface AppState {
  // Settings
  settings: AppSettings;
  settingsLoaded: boolean;

  // Version info
  version: AppVersion | null;
  updateAvailable: {
    latest: string;
    releaseUrl: string | null;
    changelog: string | null;
  } | null;

  // UI state
  theme: 'light' | 'dark';
  sidebarOpen: boolean;
  currentPage: string;

  // Actions
  loadSettings: () => Promise<void>;
  saveSettings: (settings: Partial<AppSettings>) => Promise<void>;
  loadVersion: () => Promise<void>;
  checkUpdate: () => Promise<void>;
  setUpdateAvailable: (info: { latest: string; releaseUrl: string | null; changelog: string | null } | null) => void;
  setTheme: (theme: 'light' | 'dark') => void;
  setSidebarOpen: (open: boolean) => void;
  setCurrentPage: (page: string) => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  // Initial state
  settings: DEFAULT_SETTINGS as AppSettings,
  settingsLoaded: false,
  version: null,
  updateAvailable: null,
  theme: 'light',
  sidebarOpen: true,
  currentPage: 'dashboard',

  loadSettings: async () => {
    try {
      const settings = await getSettings();
      set({ settings, settingsLoaded: true });

      // Apply language
      const resolvedLang = applyLanguage(settings.language);
      updateTrayMenu(resolvedLang).catch(console.error);

      // Apply theme
      const { theme } = settings;
      if (theme === 'system') {
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        set({ theme: prefersDark ? 'dark' : 'light' });
        document.documentElement.classList.toggle('dark', prefersDark);
      } else {
        set({ theme: theme as 'light' | 'dark' });
        document.documentElement.classList.toggle('dark', theme === 'dark');
      }
    } catch (err) {
      console.error('Failed to load settings:', err);
    }
  },

  saveSettings: async (partialSettings) => {
    const { settings } = get();
    const newSettings = { ...settings, ...partialSettings };
    try {
      await updateSettings(newSettings);
      set({ settings: newSettings });

      // Apply theme change if needed
      if ('theme' in partialSettings) {
        const theme = partialSettings.theme;
        if (theme === 'system') {
          const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
          set({ theme: prefersDark ? 'dark' : 'light' });
          document.documentElement.classList.toggle('dark', prefersDark);
        } else {
          set({ theme: theme as 'light' | 'dark' });
          document.documentElement.classList.toggle('dark', theme === 'dark');
        }
      }

      // Apply language change if needed
      if ('language' in partialSettings && partialSettings.language) {
        const resolved = applyLanguage(partialSettings.language);
        updateTrayMenu(resolved).catch(console.error);
      }
    } catch (err) {
      console.error('Failed to save settings:', err);
      throw err;
    }
  },

  loadVersion: async () => {
    try {
      const version = await getAppVersion();
      set({ version });
    } catch (err) {
      console.error('Failed to load version:', err);
    }
  },

  checkUpdate: async () => {
    try {
      const version = await checkForUpdate();
      set({ version });
      // Sync auto-detected banner with manual check result
      if (version.update_available && version.latest) {
        set({
          updateAvailable: {
            latest: version.latest,
            releaseUrl: version.release_url ?? null,
            changelog: version.changelog ?? null,
          },
        });
      }
    } catch (err) {
      console.error('Failed to check update:', err);
    }
  },

  setUpdateAvailable: (info) => set({ updateAvailable: info }),

  setTheme: (theme) => {
    set({ theme });
    document.documentElement.classList.toggle('dark', theme === 'dark');
  },

  setSidebarOpen: (open) => set({ sidebarOpen: open }),

  setCurrentPage: (page) => set({ currentPage: page }),
}));
