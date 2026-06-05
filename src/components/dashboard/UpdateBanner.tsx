import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useAppStore } from '@/stores/app';
import { ArrowUpCircle, ExternalLink, X } from 'lucide-react';

export function UpdateBanner() {
  const { t } = useTranslation();
  const updateAvailable = useAppStore((s) => s.updateAvailable);
  const [dismissed, setDismissed] = useState(false);

  if (!updateAvailable || dismissed) {
    return null;
  }

  return (
    <div className="border-b bg-blue-50 dark:bg-blue-950/30 px-6 py-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm">
          <ArrowUpCircle className="h-4 w-4 text-blue-500" />
          <span className="font-medium text-blue-800 dark:text-blue-300">
            {t('about.updateAvailable', { version: updateAvailable.latest })}
          </span>
          {updateAvailable.releaseUrl && (
            <a
              href={updateAvailable.releaseUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1 text-blue-600 dark:text-blue-400 hover:underline"
            >
              {t('about.viewRelease')}
              <ExternalLink className="h-3 w-3" />
            </a>
          )}
        </div>
        <button
          onClick={() => setDismissed(true)}
          className="text-blue-500 hover:text-blue-700"
        >
          <X className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}
