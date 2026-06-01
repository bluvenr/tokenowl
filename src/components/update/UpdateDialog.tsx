import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

interface UpdateInfo {
  currentVersion: string;
  newVersion: string;
  notes: string;
  downloadUrl: string;
}

export function UpdateDialog() {
  const { t } = useTranslation();
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    // Listen for update-available events from backend
    const unlisten = listen<UpdateInfo>("tokenowl:update-available", (event) => {
      setUpdate(event.payload);
      setOpen(true);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function handleUpdate() {
    if (update?.downloadUrl) {
      await openUrl(update.downloadUrl);
      setOpen(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("update.title")}</DialogTitle>
          <DialogDescription>
            {t("update.current_version")}: v{update?.currentVersion}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">{t("update.new_version")}:</span>
            <span className="text-sm font-semibold text-green-600">v{update?.newVersion}</span>
          </div>

          {update?.notes && (
            <div className="rounded-md bg-muted p-3 max-h-40 overflow-y-auto">
              <div className="text-xs font-medium mb-1">{t("update.changelog")}</div>
              <pre className="text-xs whitespace-pre-wrap font-sans">{update.notes}</pre>
            </div>
          )}

          <div className="flex gap-2 pt-2">
            <Button onClick={handleUpdate} className="flex-1">
              {t("update.update_now")}
            </Button>
            <Button variant="outline" onClick={() => setOpen(false)}>
              {t("update.later")}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
