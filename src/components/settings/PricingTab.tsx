import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Search } from "lucide-react";
import {
  type ModelPricing,
} from "@/lib/tauri";
import { DATA_SOURCES } from "@/lib/constants";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

export function PricingTab({
  prices,
  pricingPrefillSignal: _pricingPrefillSignal = 0,
  onUpdate,
  onReset,
  onDelete,
}: {
  prices: ModelPricing[];
  pricingPrefillSignal?: number;
  onUpdate: (price: ModelPricing) => Promise<void>;
  onReset: (modelId: string) => void;
  onDelete: (modelId: string) => void;
}) {
  const { t } = useTranslation();
  const [editing, setEditing] = useState<string | null>(null);
  const [editValues, setEditValues] = useState({ input: "", output: "", cacheWrite: "", cacheRead: "" });
  const [editTouched, setEditTouched] = useState(false);
  const editInputValid = editValues.input.trim() !== "" && !isNaN(parseFloat(editValues.input)) && parseFloat(editValues.input) >= 0;
  const editOutputValid = editValues.output.trim() !== "" && !isNaN(parseFloat(editValues.output)) && parseFloat(editValues.output) >= 0;
  const editCWValid = editValues.cacheWrite.trim() === "" || (!isNaN(parseFloat(editValues.cacheWrite)) && parseFloat(editValues.cacheWrite) >= 0);
  const editCRValid = editValues.cacheRead.trim() === "" || (!isNaN(parseFloat(editValues.cacheRead)) && parseFloat(editValues.cacheRead) >= 0);
  const editFormValid = editInputValid && editOutputValid && editCWValid && editCRValid;
  const [search, setSearch] = useState("");
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const [sourceFilter, setSourceFilter] = useState<"all" | "custom" | "default">("all");
  const [showAddForm, setShowAddForm] = useState(false);

  const filtered = prices.filter((p) => {
    if (sourceFilter === "custom" && p.priceSource !== "custom") return false;
    if (sourceFilter === "default" && p.priceSource === "custom") return false;
    if (search.trim()) {
      const q = search.toLowerCase();
      return (
        p.modelId.toLowerCase().includes(q) ||
        p.displayName.toLowerCase().includes(q) ||
        p.source.toLowerCase().includes(q)
      );
    }
    return true;
  });

  function startEdit(p: ModelPricing) {
    setEditing(p.modelId);
    setEditTouched(false);
    setEditValues({
      input: String(p.inputPerMillion),
      output: String(p.outputPerMillion),
      cacheWrite: p.cacheWritePerMillion != null ? String(p.cacheWritePerMillion) : "",
      cacheRead: p.cacheReadPerMillion != null ? String(p.cacheReadPerMillion) : "",
    });
  }

  async function saveEdit(p: ModelPricing) {
    setEditTouched(true);
    if (!editFormValid) return;
    const inputVal = parseFloat(editValues.input);
    const outputVal = parseFloat(editValues.output);
    const cacheWriteVal = parseFloat(editValues.cacheWrite);
    const cacheReadVal = parseFloat(editValues.cacheRead);
    const updated: ModelPricing = {
      ...p,
      inputPerMillion: isNaN(inputVal) ? p.inputPerMillion : inputVal,
      outputPerMillion: isNaN(outputVal) ? p.outputPerMillion : outputVal,
      cacheWritePerMillion: isNaN(cacheWriteVal) ? (p.cacheWritePerMillion ?? null) : cacheWriteVal,
      cacheReadPerMillion: isNaN(cacheReadVal) ? (p.cacheReadPerMillion ?? null) : cacheReadVal,
      priceSource: "custom",
    };
    await onUpdate(updated);
    setEditing(null);
  }

  const sourceBadge = (source: string) => {
    const colors: Record<string, string> = {
      custom: "bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300",
      remote: "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300",
    };
    const fallback = "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400";
    const display = source === "cached" ? "remote" : source;
    return (
      <span className={`text-[10px] px-1.5 py-0.5 rounded font-medium ${colors[display] || fallback}`}>
        {t(`settings.source_${display}`)}
      </span>
    );
  };

  return (
    <>
    <div className="space-y-2">
      {/* Search bar */}
      <div className="relative">
        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
        <Input
          className="h-8 pl-8 text-xs"
          placeholder={t("settings.search_model")}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      {/* Source filter */}
      <div className="flex gap-0.5 rounded-lg bg-muted/40 p-0.5">
        {([
          ["all", "filter_all"],
          ["custom", "filter_custom"],
          ["default", "filter_default"],
        ] as const).map(([key, label]) => (
          <button
            key={key}
            onClick={() => setSourceFilter(key)}
            className={`px-2.5 py-1 text-[11px] font-medium rounded-md transition-all ${
              sourceFilter === key
                ? "bg-background text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground/80"
            }`}
          >
            {t(`settings.${label}`)}
            {key === "custom" && prices.filter((p) => p.priceSource === "custom").length > 0 && (
              <span className="ml-1 opacity-60">{prices.filter((p) => p.priceSource === "custom").length}</span>
            )}
          </button>
        ))}
      </div>

      {/* Add Custom Model Button */}
      {!showAddForm && (
        <Button
          variant="outline"
          size="sm"
          className="w-full text-xs border-border/60 border-dashed"
          onClick={() => { setShowAddForm(true); }}
        >
          + {t("settings.add_custom_model")}
        </Button>
      )}

      {/* Add Custom Model Form */}
      {showAddForm && (
        <AddCustomModelForm
          existingIds={new Set(prices.map((p) => p.modelId))}
          onSave={async (price) => {
            await onUpdate(price);
            setShowAddForm(false);
          }}
          onCancel={() => { setShowAddForm(false); }}
        />
      )}

      {/* Existing Price List (filtered) */}
      {filtered.map((p) => (
        <Card key={p.modelId} className="border-border/60">
          <CardContent className={editing === p.modelId ? "py-3 space-y-3" : "flex items-center gap-3 py-2.5"}>
            {editing === p.modelId ? (
              <>
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">{p.displayName}</span>
                  {sourceBadge(p.priceSource)}
                </div>
                <div className="grid grid-cols-2 gap-2">
                  <div>
                    <label className="text-[10px] text-muted-foreground">{t("settings.input_price")} <span className="text-red-500">*</span></label>
                    <Input
                      className={`h-7 text-xs ${editTouched && !editInputValid ? "border-red-500 focus-visible:ring-red-500" : ""}`}
                      type="number"
                      step="0.01"
                      min="0"
                      value={editValues.input}
                      onChange={(e) => setEditValues({ ...editValues, input: e.target.value })}
                    />
                    {editTouched && !editInputValid && (
                      <p className="text-[10px] text-red-500 mt-0.5">{t("settings.invalid_price")}</p>
                    )}
                  </div>
                  <div>
                    <label className="text-[10px] text-muted-foreground">{t("settings.output_price")} <span className="text-red-500">*</span></label>
                    <Input
                      className={`h-7 text-xs ${editTouched && !editOutputValid ? "border-red-500 focus-visible:ring-red-500" : ""}`}
                      type="number"
                      step="0.01"
                      min="0"
                      value={editValues.output}
                      onChange={(e) => setEditValues({ ...editValues, output: e.target.value })}
                    />
                    {editTouched && !editOutputValid && (
                      <p className="text-[10px] text-red-500 mt-0.5">{t("settings.invalid_price")}</p>
                    )}
                  </div>
                  <div>
                    <label className="text-[10px] text-muted-foreground">{t("settings.cache_write_price")}</label>
                    <Input
                      className={`h-7 text-xs ${editTouched && !editCWValid ? "border-red-500 focus-visible:ring-red-500" : ""}`}
                      type="number"
                      step="0.01"
                      min="0"
                      value={editValues.cacheWrite}
                      onChange={(e) => setEditValues({ ...editValues, cacheWrite: e.target.value })}
                      placeholder={t("common.optional")}
                    />
                    {editTouched && !editCWValid && (
                      <p className="text-[10px] text-red-500 mt-0.5">{t("settings.invalid_price")}</p>
                    )}
                  </div>
                  <div>
                    <label className="text-[10px] text-muted-foreground">{t("settings.cache_read_price")}</label>
                    <Input
                      className={`h-7 text-xs ${editTouched && !editCRValid ? "border-red-500 focus-visible:ring-red-500" : ""}`}
                      type="number"
                      step="0.01"
                      min="0"
                      value={editValues.cacheRead}
                      onChange={(e) => setEditValues({ ...editValues, cacheRead: e.target.value })}
                      placeholder={t("common.optional")}
                    />
                    {editTouched && !editCRValid && (
                      <p className="text-[10px] text-red-500 mt-0.5">{t("settings.invalid_price")}</p>
                    )}
                  </div>
                </div>
                <div className="flex items-center gap-2 justify-end">
                  <Button size="sm" variant="default" className="h-7 text-xs" onClick={() => saveEdit(p)} disabled={editTouched && !editFormValid}>
                    {t("settings.save")}
                  </Button>
                  <Button size="sm" variant="ghost" className="h-7 text-xs" onClick={() => setEditing(null)}>
                    {t("common.cancel")}
                  </Button>
                </div>
              </>
            ) : (
              <>
                <span className="flex-1 text-sm font-medium truncate">{p.displayName}</span>
                <div className="flex flex-col items-end">
                  <div className="flex items-center gap-2 text-xs text-muted-foreground tabular-nums">
                    <span>{t("settings.lbl_input")}: ${p.inputPerMillion}</span>
                    <span>{t("settings.lbl_output")}: ${p.outputPerMillion}</span>
                  </div>
                  {(p.cacheWritePerMillion != null || p.cacheReadPerMillion != null) && (
                    <div className="flex items-center gap-2 text-[10px] text-muted-foreground/70 tabular-nums">
                      {p.cacheWritePerMillion != null && <span>{t("settings.lbl_cache_write")}: ${p.cacheWritePerMillion}</span>}
                      {p.cacheReadPerMillion != null && <span>{t("settings.lbl_cache_read")}: ${p.cacheReadPerMillion}</span>}
                    </div>
                  )}
                </div>
                {sourceBadge(p.priceSource)}
                <Button
                  size="sm"
                  variant="ghost"
                  className="h-7 text-xs px-2"
                  onClick={() => startEdit(p)}
                >
                  {t("common.edit")}
                </Button>
                {p.priceSource === "custom" && p.hasDefault && (
                  <Button
                    size="sm"
                    variant="ghost"
                    className="h-7 text-xs px-2"
                    onClick={() => onReset(p.modelId)}
                  >
                    {t("settings.reset_default")}
                  </Button>
                )}
                {p.priceSource === "custom" && !p.hasDefault && (
                  <Button
                    size="sm"
                    variant="ghost"
                    className="h-7 text-xs px-2 text-red-500 hover:text-red-600"
                    onClick={() => setDeleteConfirmId(p.modelId)}
                  >
                    {t("settings.delete_model")}
                  </Button>
                )}
              </>
            )}
          </CardContent>
        </Card>
      ))}
      {filtered.length === 0 && search.trim() && (
        <div className="text-center text-xs text-muted-foreground py-6">
          {t("settings.no_match")}
        </div>
      )}
    </div>
    {/* Delete confirmation dialog */}
    <Dialog open={deleteConfirmId !== null} onOpenChange={(open) => { if (!open) setDeleteConfirmId(null); }}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>{t("settings.delete_confirm_title")}</DialogTitle>
          <DialogDescription>
            {t("settings.delete_confirm_msg", { modelId: deleteConfirmId ?? "" })}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className="mt-2">
          <Button variant="outline" size="sm" onClick={() => setDeleteConfirmId(null)}>
            {t("common.cancel")}
          </Button>
          <Button
            variant="destructive"
            size="sm"
            onClick={() => {
              if (deleteConfirmId) {
                onDelete(deleteConfirmId);
              }
              setDeleteConfirmId(null);
            }}
          >
            {t("settings.confirm_delete")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
    </>
  );
}

// ─── Add Custom Model Form ──────────────────────────────────────────

function AddCustomModelForm({
  existingIds = new Set<string>(),
  onSave,
  onCancel,
}: {
  existingIds?: Set<string>;
  onSave: (price: ModelPricing) => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  const [form, setForm] = useState({
    modelId: "",
    displayName: "",
    source: "cc_switch",
    inputPerMillion: "",
    outputPerMillion: "",
    cacheWritePerMillion: "",
    cacheReadPerMillion: "",
  });
  const isDuplicate = existingIds.has(form.modelId.trim());

  const modelIdValid = form.modelId.trim().length > 0;
  const displayNameValid = form.displayName.trim().length > 0;
  const inputValid = form.inputPerMillion.trim() !== "" && !isNaN(parseFloat(form.inputPerMillion)) && parseFloat(form.inputPerMillion) >= 0;
  const outputValid = form.outputPerMillion.trim() !== "" && !isNaN(parseFloat(form.outputPerMillion)) && parseFloat(form.outputPerMillion) >= 0;
  const cacheWriteValid = form.cacheWritePerMillion.trim() === "" || (!isNaN(parseFloat(form.cacheWritePerMillion)) && parseFloat(form.cacheWritePerMillion) >= 0);
  const cacheReadValid = form.cacheReadPerMillion.trim() === "" || (!isNaN(parseFloat(form.cacheReadPerMillion)) && parseFloat(form.cacheReadPerMillion) >= 0);
  const formValid = modelIdValid && displayNameValid && inputValid && outputValid && cacheWriteValid && cacheReadValid;
  const [touched, setTouched] = useState(false);
  const [showDupConfirm, setShowDupConfirm] = useState(false);

  function handleSave() {
    setTouched(true);
    if (!formValid) return;
    if (isDuplicate) {
      setShowDupConfirm(true);
      return;
    }
    doSave();
  }

  function doSave() {
    setShowDupConfirm(false);
    onSave({
      modelId: form.modelId.trim(),
      displayName: form.displayName.trim(),
      source: form.source,
      inputPerMillion: parseFloat(form.inputPerMillion) || 0,
      outputPerMillion: parseFloat(form.outputPerMillion) || 0,
      cacheWritePerMillion: form.cacheWritePerMillion ? parseFloat(form.cacheWritePerMillion) : null,
      cacheReadPerMillion: form.cacheReadPerMillion ? parseFloat(form.cacheReadPerMillion) : null,
      priceSource: "custom",
    });
  }

  return (
    <>
    <Card className="border-dashed border-2 border-border/60">
      <CardContent className="space-y-3 pt-4">
        <div className="text-sm font-medium">{t("settings.add_custom_model")}</div>

        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="text-[10px] text-muted-foreground">{t("settings.model_id")} <span className="text-red-500">*</span></label>
            <Input
              className={`h-7 text-xs ${touched && !modelIdValid ? "border-red-500 focus-visible:ring-red-500" : ""} ${isDuplicate ? "border-amber-500 focus-visible:ring-amber-500" : ""}`}
              value={form.modelId}
              onChange={(e) => setForm({ ...form, modelId: e.target.value })}
              placeholder={t("settings.model_id_placeholder")}
            />
            {touched && !modelIdValid && (
              <p className="text-[10px] text-red-500 mt-0.5">{t("settings.required_field")}</p>
            )}
            {isDuplicate && (
              <p className="text-[10px] text-amber-600 dark:text-amber-400 mt-0.5">
                {t("settings.model_exists")}
              </p>
            )}
          </div>
          <div>
            <label className="text-[10px] text-muted-foreground">{t("settings.display_name")} <span className="text-red-500">*</span></label>
            <Input
              className={`h-7 text-xs ${touched && !displayNameValid ? "border-red-500 focus-visible:ring-red-500" : ""}`}
              value={form.displayName}
              onChange={(e) => setForm({ ...form, displayName: e.target.value })}
              placeholder={t("settings.display_name_placeholder")}
            />
            {touched && !displayNameValid && (
              <p className="text-[10px] text-red-500 mt-0.5">{t("settings.required_field")}</p>
            )}
          </div>
        </div>

        <div>
          <label className="text-[10px] text-muted-foreground">{t("settings.data_source")}</label>
          <Select
            value={form.source}
            onValueChange={(v) => setForm({ ...form, source: v })}
          >
            <SelectTrigger className="h-7 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {DATA_SOURCES.map((ds) => (
                <SelectItem key={ds.key} value={ds.key}>
                  {ds.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="text-[10px] text-muted-foreground">{t("settings.input_price")} <span className="text-red-500">*</span></label>
            <Input
              className={`h-7 text-xs ${touched && !inputValid ? "border-red-500 focus-visible:ring-red-500" : ""}`}
              type="number"
              step="0.01"
              min="0"
              value={form.inputPerMillion}
              onChange={(e) => setForm({ ...form, inputPerMillion: e.target.value })}
              placeholder={t("settings.price_placeholder")}
            />
            {touched && !inputValid && (
              <p className="text-[10px] text-red-500 mt-0.5">{t("settings.invalid_price")}</p>
            )}
          </div>
          <div>
            <label className="text-[10px] text-muted-foreground">{t("settings.output_price")} <span className="text-red-500">*</span></label>
            <Input
              className={`h-7 text-xs ${touched && !outputValid ? "border-red-500 focus-visible:ring-red-500" : ""}`}
              type="number"
              step="0.01"
              min="0"
              value={form.outputPerMillion}
              onChange={(e) => setForm({ ...form, outputPerMillion: e.target.value })}
              placeholder={t("settings.price_placeholder")}
            />
            {touched && !outputValid && (
              <p className="text-[10px] text-red-500 mt-0.5">{t("settings.invalid_price")}</p>
            )}
          </div>
        </div>

        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="text-[10px] text-muted-foreground">{t("settings.cache_write_price")}</label>
            <Input
              className={`h-7 text-xs ${touched && !cacheWriteValid ? "border-red-500 focus-visible:ring-red-500" : ""}`}
              type="number"
              step="0.01"
              min="0"
              value={form.cacheWritePerMillion}
              onChange={(e) => setForm({ ...form, cacheWritePerMillion: e.target.value })}
              placeholder={t("common.optional")}
            />
            {touched && !cacheWriteValid && (
              <p className="text-[10px] text-red-500 mt-0.5">{t("settings.invalid_price")}</p>
            )}
          </div>
          <div>
            <label className="text-[10px] text-muted-foreground">{t("settings.cache_read_price")}</label>
            <Input
              className={`h-7 text-xs ${touched && !cacheReadValid ? "border-red-500 focus-visible:ring-red-500" : ""}`}
              type="number"
              step="0.01"
              min="0"
              value={form.cacheReadPerMillion}
              onChange={(e) => setForm({ ...form, cacheReadPerMillion: e.target.value })}
              placeholder={t("common.optional")}
            />
            {touched && !cacheReadValid && (
              <p className="text-[10px] text-red-500 mt-0.5">{t("settings.invalid_price")}</p>
            )}
          </div>
        </div>

        <div className="flex gap-2 pt-1">
          <Button size="sm" className="h-7 text-xs" onClick={handleSave} disabled={touched && !formValid}>
            {t("settings.save")}
          </Button>
          <Button size="sm" variant="ghost" className="h-7 text-xs" onClick={onCancel}>
            {t("common.cancel")}
          </Button>
        </div>
      </CardContent>
    </Card>

    {/* Duplicate model ID confirmation dialog */}
    <Dialog open={showDupConfirm} onOpenChange={setShowDupConfirm}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>{t("settings.model_exists_dialog_title")}</DialogTitle>
          <DialogDescription>
            {t("settings.model_exists_confirm", { modelId: form.modelId.trim() })}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className="mt-2">
          <Button variant="outline" size="sm" onClick={() => setShowDupConfirm(false)}>
            {t("common.cancel")}
          </Button>
          <Button size="sm" onClick={doSave}>
            {t("settings.confirm_overwrite")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
    </>
  );
}
