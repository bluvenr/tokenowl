import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

/**
 * Hook to manage remote services (price sync, announcements).
 * Call this once in App.tsx on mount.
 * Update checking is handled independently by the UpdateDialog component.
 */
export function useRemoteServices() {
  useEffect(() => {
    // Listen for price sync events
    const unlistenPrices = listen<number>("tokenowl:prices-synced", () => {
      // Price sync completed — dashboard will refresh via data-changed event
    });

    return () => {
      unlistenPrices.then((fn) => fn());
    };
  }, []);
}
