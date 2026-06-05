import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { BudgetConfig } from "@/lib/tauri";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { showToast } from "@/components/budget/BudgetAlert";
import { SettingRow } from "./shared";

export function BudgetTab({
  budget,
  onSave,
}: {
  budget: BudgetConfig;
  onSave: (b: BudgetConfig) => void;
}) {
  const { t } = useTranslation();
  const [local, setLocal] = useState(budget);

  function applyChange(update: Partial<BudgetConfig>) {
    const updated = { ...local, ...update };
    setLocal(updated);
    Promise.resolve(onSave(updated)).catch((e) => {
      showToast(t("common.error"), String(e), "error");
    });
  }

  async function handleSave() {
    try {
      await onSave(local);
      showToast(t("settings.save"), t("settings.saved"), "success");
    } catch (e) {
      showToast(t("common.error"), String(e), "error");
    }
  }

  return (
    <Card className="border-border/60">
      <CardContent className="space-y-4 pt-6">
        <BudgetInput
          label={t("settings.daily_budget")}
          value={local.dailyLimitUsd}
          onChange={(v) => setLocal({ ...local, dailyLimitUsd: v })}
        />
        <BudgetInput
          label={t("settings.weekly_budget")}
          value={local.weeklyLimitUsd}
          onChange={(v) => setLocal({ ...local, weeklyLimitUsd: v })}
        />
        <BudgetInput
          label={t("settings.monthly_budget")}
          value={local.monthlyLimitUsd}
          onChange={(v) => setLocal({ ...local, monthlyLimitUsd: v })}
        />

        <SettingRow label={t("settings.alert_threshold")}>
          <div className="flex items-center gap-2">
            <Input
              type="number"
              className="w-20 h-8 text-sm"
              value={local.alertThresholdPct}
              onChange={(e) =>
                setLocal({ ...local, alertThresholdPct: parseInt(e.target.value) || 80 })
              }
              min={10}
              max={100}
            />
            <span className="text-sm text-muted-foreground">%</span>
          </div>
        </SettingRow>

        <div className="flex items-center gap-3 pt-2">
          <Button onClick={handleSave}>{t("settings.save")}</Button>
        </div>

        <div className="border-t pt-4 mt-2 space-y-3">
          <SettingRow label={t("settings.alert_icon_color")}>
            <Switch
              checked={local.alertIconColor}
              onCheckedChange={(v) => applyChange({ alertIconColor: v })}
            />
          </SettingRow>

          <SettingRow label={t("settings.alert_system_notify")}>
            <Switch
              checked={local.alertSystemNotify}
              onCheckedChange={(v) => applyChange({ alertSystemNotify: v })}
            />
          </SettingRow>
        </div>
      </CardContent>
    </Card>
  );
}

function BudgetInput({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number | null;
  onChange: (v: number | null) => void;
}) {
  const { t } = useTranslation();
  const hasValue = value !== null && value > 0;

  return (
    <SettingRow label={label}>
      <div className="flex items-center gap-2">
        {hasValue && <span className="text-sm text-muted-foreground">$</span>}
        <Input
          type="number"
          className="w-24 h-8 text-sm"
          value={hasValue ? value : ""}
          onChange={(e) => {
            const v = e.target.value;
            onChange(v === "" ? null : parseFloat(v));
          }}
          placeholder={t("settings.no_limit")}
          min={0}
          step={1}
        />
      </div>
    </SettingRow>
  );
}
