import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Card, CardContent } from "@/components/ui/card";

interface Announcement {
  id: string;
  title: string;
  message: string;
  link?: string;
  dismissible: boolean;
}

export function AnnouncementBanner() {
  const { t } = useTranslation();
  const [announcement, setAnnouncement] = useState<Announcement | null>(null);
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    // Listen for announcement events from backend
    const unlisten = listen<Announcement>("tokenowl:announcement", (event) => {
      const ann = event.payload;
      // Check if previously dismissed
      const dismissedIds = getDismissedIds();
      if (!dismissedIds.includes(ann.id)) {
        setAnnouncement(ann);
        setDismissed(false);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  function handleDismiss() {
    if (announcement) {
      saveDismissedId(announcement.id);
    }
    setDismissed(true);
  }

  async function handleLink() {
    if (announcement?.link) {
      await openUrl(announcement.link);
    }
  }

  if (!announcement || dismissed) return null;

  return (
    <Card className="border-blue-200 bg-blue-50 dark:border-blue-800 dark:bg-blue-950/50">
      <CardContent className="p-3 flex items-start gap-3">
        <div className="text-blue-500 text-lg mt-0.5">📢</div>
        <div className="flex-1 min-w-0">
          <div className="text-sm font-medium">{announcement.title}</div>
          <div className="text-xs text-muted-foreground mt-0.5">{announcement.message}</div>
          {announcement.link && (
            <button
              onClick={handleLink}
              className="text-xs text-blue-600 hover:text-blue-700 underline mt-1"
            >
              {t("common.details")}
            </button>
          )}
        </div>
        {announcement.dismissible && (
          <button
            aria-label={t("announcement.dismiss")}
            onClick={handleDismiss}
            className="text-muted-foreground hover:text-foreground text-sm px-1 shrink-0"
          >
            ×
          </button>
        )}
      </CardContent>
    </Card>
  );
}

// Dismissed ID persistence using localStorage
const DISMISS_KEY = "tokenowl:dismissed-announcements";

function getDismissedIds(): string[] {
  try {
    const raw = localStorage.getItem(DISMISS_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveDismissedId(id: string) {
  try {
    const ids = getDismissedIds();
    if (!ids.includes(id)) {
      ids.push(id);
      localStorage.setItem(DISMISS_KEY, JSON.stringify(ids));
    }
  } catch {
    // localStorage unavailable, silently ignore
  }
}
