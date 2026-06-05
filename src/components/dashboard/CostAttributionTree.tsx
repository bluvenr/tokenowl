import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDashboardStore } from '@/stores/dashboard';
import { formatUsd, formatTokens } from '@/lib/format';
import { TreePine, ChevronRight, ChevronDown, Layers } from 'lucide-react';

const TOKEN_TYPE_I18N_KEYS: Record<string, string> = {
  input: 'attribution.input',
  output: 'attribution.output',
  cache_write: 'attribution.cacheWrite',
  cache_read: 'attribution.cacheRead',
  reasoning: 'attribution.reasoning',
};

function TokenBar({ percentage }: { percentage: number }) {
  return (
    <div className="h-1.5 w-full rounded-full bg-muted overflow-hidden">
      <div
        className="h-full rounded-full bg-primary transition-all duration-300"
        style={{ width: `${Math.min(100, Math.max(1, percentage))}%` }}
      />
    </div>
  );
}

export function CostAttributionTree() {
  const { t } = useTranslation();
  const providers = useDashboardStore((s) => s.costAttribution);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  if (!providers || providers.length === 0) return null;

  const toggle = (key: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      return next;
    });
  };

  return (
    <div className="rounded-lg border bg-card p-4 shadow-sm">
      <div className="flex items-center gap-2 mb-4">
        <TreePine className="h-5 w-5 text-emerald-500" />
        <h3 className="font-semibold">{t('attribution.title')}</h3>
      </div>

      <div className="space-y-2">
        {providers.map((provider) => {
          const providerKey = provider.provider_name;
          const isProviderOpen = expanded.has(providerKey);

          return (
            <div key={providerKey} className="rounded-lg border bg-muted/30">
              {/* Provider row */}
              <button
                type="button"
                className="flex w-full items-center gap-2 p-3 text-sm text-left hover:bg-muted/50 transition-colors"
                onClick={() => toggle(providerKey)}
              >
                {isProviderOpen ? (
                  <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
                ) : (
                  <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
                )}
                <span className="font-medium flex-1 truncate" title={provider.provider_name}>{provider.provider_name}</span>
                <span className="text-muted-foreground">{provider.models.length} {t('attribution.models')}</span>
                <span className="font-semibold w-20 text-right">{formatUsd(provider.cost_usd)}</span>
                <span className="text-xs text-muted-foreground w-12 text-right">
                  {provider.percentage.toFixed(1)}%
                </span>
              </button>

              {/* Cost bar */}
              <div className="px-3 pb-1">
                <TokenBar percentage={provider.percentage} />
              </div>

              {/* Models */}
              {isProviderOpen && (
                <div className="border-t">
                  {provider.models.map((model) => {
                    const modelKey = `${providerKey}/${model.model}`;
                    const isModelOpen = expanded.has(modelKey);

                    return (
                      <div key={modelKey}>
                        {/* Model row */}
                        <button
                          type="button"
                          className="flex w-full items-center gap-2 p-2.5 pl-8 text-sm text-left hover:bg-muted/50 transition-colors"
                          onClick={() => toggle(modelKey)}
                        >
                          {isModelOpen ? (
                            <ChevronDown className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                          ) : (
                            <ChevronRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                          )}
                          <span className="flex-1 truncate" title={model.model}>{model.model}</span>
                          <span className="text-muted-foreground">{formatTokens(model.total_tokens)}</span>
                          <span className="font-medium w-20 text-right">{formatUsd(model.cost_usd)}</span>
                          <span className="text-xs text-muted-foreground w-12 text-right">
                            {model.percentage.toFixed(1)}%
                          </span>
                        </button>

                        {/* Token breakdown */}
                        {isModelOpen && model.token_breakdown.length > 0 && (
                          <div className="px-12 pb-3 space-y-1.5">
                            <div className="flex items-center gap-1.5 text-xs text-muted-foreground mb-1">
                              <Layers className="h-3 w-3" />
                              <span>{t('attribution.tokenTypes')}</span>
                            </div>
                            {model.token_breakdown.map((token) => (
                              <div key={token.token_type} className="flex items-center gap-2 text-xs">
                                <span className="w-24 text-muted-foreground">
                                  {TOKEN_TYPE_I18N_KEYS[token.token_type] ? t(TOKEN_TYPE_I18N_KEYS[token.token_type]) : token.token_type}
                                </span>
                                <div className="flex-1">
                                  <TokenBar percentage={token.percentage} />
                                </div>
                                <span className="w-16 text-right">{formatTokens(token.tokens)}</span>
                                <span className="w-16 text-right font-medium">{formatUsd(token.cost_usd)}</span>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
