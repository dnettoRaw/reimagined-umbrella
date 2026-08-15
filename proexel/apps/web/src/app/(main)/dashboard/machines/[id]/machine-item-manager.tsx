"use client";

import { useState } from "react";

import { useRouter } from "next/navigation";

import { ArrowDown, ArrowUp, Pencil, Plus, RefreshCw, Trash2 } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { useI18n } from "@/lib/i18n/provider";
import type { EquivalentPart, ItemCategory, Machine, MachineItem } from "@/lib/proexel/types";

type Mode = "create" | "edit" | "replace";
type SpecificationRow = { id: string; key: string; value: string };
type EquivalentDraft = EquivalentPart & { id: string };

export function MachineItemManager({
  machine,
  categories,
  canManage,
}: {
  machine: Machine;
  categories: ItemCategory[];
  canManage: boolean;
}) {
  const { t } = useI18n();
  const router = useRouter();
  const [editor, setEditor] = useState<{ mode: Mode; item?: MachineItem } | null>(null);
  const [pending, setPending] = useState(false);

  async function command(method: string, data: Record<string, unknown>) {
    setPending(true);
    try {
      const response = await fetch("/api/proexel/machine-items", {
        method,
        headers: { "content-type": "application/json" },
        body: JSON.stringify(data),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string };
      if (!response.ok || !result.accepted) throw new Error(result.message ?? t("command.rejected"));
      toast.success(t("command.success"));
      setEditor(null);
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t("command.failed"));
    } finally {
      setPending(false);
    }
  }

  function reorder(index: number, direction: -1 | 1) {
    const ids = machine.items.map((item) => item.id);
    const target = index + direction;
    [ids[index], ids[target]] = [ids[target], ids[index]];
    return command("PUT", { action: "reorder", machine_id: machine.id, item_ids: ids });
  }

  if (!canManage) return null;
  return (
    <>
      <div className="flex justify-end">
        <Button onClick={() => setEditor({ mode: "create" })}>
          <Plus />
          {t("machines.addItem")}
        </Button>
      </div>
      <div className="mt-3 divide-y rounded-md border">
        {machine.items.map((item, index) => (
          <div key={item.id} className="flex flex-wrap items-center gap-2 p-2">
            <span className="min-w-0 flex-1 truncate font-medium text-sm">
              {item.code} · {item.name}
            </span>
            <Button
              size="icon-sm"
              variant="ghost"
              disabled={index === 0 || pending}
              title={t("common.previous")}
              onClick={() => reorder(index, -1)}
            >
              <ArrowUp />
            </Button>
            <Button
              size="icon-sm"
              variant="ghost"
              disabled={index === machine.items.length - 1 || pending}
              title={t("common.next")}
              onClick={() => reorder(index, 1)}
            >
              <ArrowDown />
            </Button>
            <Button
              size="icon-sm"
              variant="ghost"
              title={t("machines.editItem")}
              onClick={() => setEditor({ mode: "edit", item })}
            >
              <Pencil />
            </Button>
            <Button
              size="icon-sm"
              variant="ghost"
              title={t("machines.replaceItem")}
              onClick={() => setEditor({ mode: "replace", item })}
            >
              <RefreshCw />
            </Button>
            <Button
              size="icon-sm"
              variant="ghost"
              title={t("machines.removeItem")}
              onClick={() => confirm(t("machines.removeItemConfirm")) && command("DELETE", { id: item.id })}
            >
              <Trash2 />
            </Button>
          </div>
        ))}
      </div>
      <Dialog open={editor !== null} onOpenChange={(open) => !open && setEditor(null)}>
        <DialogContent className="max-h-[94vh] overflow-y-auto sm:max-w-3xl">
          <DialogHeader>
            <DialogTitle>
              {editor?.mode === "create"
                ? t("machines.addItem")
                : editor?.mode === "edit"
                  ? t("machines.editItem")
                  : t("machines.replaceItem")}
            </DialogTitle>
            <DialogDescription>
              {editor?.mode === "replace" ? t("machines.replacementReason") : t("machines.detailDescription")}
            </DialogDescription>
          </DialogHeader>
          {editor?.mode === "replace" && editor.item ? (
            <ReplacementForm item={editor.item} pending={pending} submit={(data) => command("PUT", data)} />
          ) : editor ? (
            <ItemForm
              machine={machine}
              categories={categories.filter((category) => category.active || category.id === editor.item?.category_id)}
              item={editor.item}
              pending={pending}
              submit={(data) => command(editor.mode === "create" ? "POST" : "PATCH", data)}
            />
          ) : null}
        </DialogContent>
      </Dialog>
    </>
  );
}

function ItemForm({
  machine,
  categories,
  item,
  pending,
  submit,
}: {
  machine: Machine;
  categories: ItemCategory[];
  item?: MachineItem;
  pending: boolean;
  submit: (data: Record<string, unknown>) => void;
}) {
  const { t } = useI18n();
  const [categoryId, setCategoryId] = useState(item?.category_id ?? categories[0]?.id ?? "");
  const category = categories.find((entry) => entry.id === categoryId);
  const [values, setValues] = useState<Record<string, unknown>>(item?.custom_field_values ?? {});
  const [installedSpecifications, setInstalledSpecifications] = useState<SpecificationRow[]>([]);
  const [replacementSpecifications, setReplacementSpecifications] = useState<SpecificationRow[]>(() =>
    specificationRows(item?.replacement_specification.technical_specifications),
  );
  const [equivalentParts, setEquivalentParts] = useState<EquivalentDraft[]>(() =>
    (item?.replacement_specification.equivalent_parts ?? []).map((part) => ({
      ...part,
      id: crypto.randomUUID(),
    })),
  );
  function send(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = Object.fromEntries(new FormData(event.currentTarget));
    const complexity = Number(data.complexity_level || category?.default_complexity_level || 3);
    const payload: Record<string, unknown> = {
      ...(item ? { id: item.id } : { machine_id: machine.id }),
      category_id: categoryId,
      name: data.name,
      code: data.code,
      complexity_level: complexity,
      status: data.status || "unknown",
      location_description: data.location_description || null,
      custom_field_values: values,
      notes: data.notes || null,
      replacement_specification: {
        manufacturer: data.replacement_manufacturer || null,
        part_number: data.replacement_part_number || null,
        model: data.replacement_model || null,
        serial_number: data.replacement_serial_number || null,
        technical_specifications: specificationRecord(replacementSpecifications),
        compatibility_notes: data.compatibility_notes || null,
        equivalent_parts: equivalentParts.filter((part) => part.part_number.trim()).map(({ id: _id, ...part }) => part),
        supplier_reference: data.supplier_reference || null,
        photo_ids: item?.replacement_specification.photo_ids ?? [],
      },
    };
    if (!item)
      payload.installed_component = {
        manufacturer: data.manufacturer || null,
        model: data.model || null,
        part_number: data.part_number || null,
        serial_number: data.serial_number || null,
        installed_at: data.installed_at || null,
        technical_specifications: specificationRecord(installedSpecifications),
      };
    submit(payload);
  }
  return (
    <form onSubmit={send} className="space-y-5">
      <div className="grid gap-3 sm:grid-cols-2">
        <Field label={t("common.category")}>
          <select
            name="category_id"
            className="h-9 rounded-md border bg-background px-3 text-sm"
            value={categoryId}
            onChange={(event) => {
              setCategoryId(event.target.value);
              setValues({});
            }}
            required
          >
            {categories.map((entry) => (
              <option key={entry.id} value={entry.id}>
                {entry.name}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t("common.complexity")}>
          <select
            name="complexity_level"
            defaultValue={item?.complexity_level ?? category?.default_complexity_level ?? 3}
            className="h-9 rounded-md border bg-background px-3 text-sm"
          >
            {([1, 2, 3, 4, 5] as const).map((level) => (
              <option key={level} value={level}>
                {level} - {t(`complexity.${level}`)}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t("common.code")}>
          <Input name="code" defaultValue={item?.code} required />
        </Field>
        <Field label={t("common.name")}>
          <Input name="name" defaultValue={item?.name} required />
        </Field>
        <Field label={t("common.status")}>
          <select
            name="status"
            defaultValue={item?.status ?? "unknown"}
            className="h-9 rounded-md border bg-background px-3 text-sm"
          >
            {(
              [
                "unknown",
                "ok",
                "attention",
                "critical",
                "maintenance_required",
                "under_maintenance",
                "disabled",
              ] as const
            ).map((status) => (
              <option key={status} value={status}>
                {t(`status.${status}`)}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t("machines.locationDescription")}>
          <Input name="location_description" defaultValue={item?.location_description ?? ""} />
        </Field>
      </div>
      {!item ? (
        <fieldset className="grid gap-3 border-t pt-4 sm:grid-cols-2">
          <legend className="mb-2 font-semibold text-sm">{t("machines.installedComponent")}</legend>
          <Field label={t("common.manufacturer")}>
            <Input name="manufacturer" />
          </Field>
          <Field label={t("common.model")}>
            <Input name="model" />
          </Field>
          <Field label={t("common.partNumber")}>
            <Input name="part_number" />
          </Field>
          <Field label={t("common.serialNumber")}>
            <Input name="serial_number" />
          </Field>
          <Field label={t("common.date")}>
            <Input name="installed_at" type="date" />
          </Field>
          <div className="sm:col-span-2">
            <TechnicalSpecifications rows={installedSpecifications} update={setInstalledSpecifications} />
          </div>
        </fieldset>
      ) : null}
      {category?.custom_field_definitions.length ? (
        <fieldset className="grid gap-3 border-t pt-4 sm:grid-cols-2">
          <legend className="mb-2 font-semibold text-sm">{t("machines.customFields")}</legend>
          {category.custom_field_definitions.map((definition) => (
            <CustomField
              key={definition.id}
              definition={definition}
              value={values[definition.key]}
              update={(value) => setValues((current) => ({ ...current, [definition.key]: value }))}
            />
          ))}
        </fieldset>
      ) : null}
      <fieldset className="grid gap-3 border-t pt-4 sm:grid-cols-2">
        <legend className="mb-2 font-semibold text-sm">{t("machines.replacementSpec")}</legend>
        <Field label={t("common.manufacturer")}>
          <Input name="replacement_manufacturer" defaultValue={item?.replacement_specification.manufacturer ?? ""} />
        </Field>
        <Field label={t("common.model")}>
          <Input name="replacement_model" defaultValue={item?.replacement_specification.model ?? ""} />
        </Field>
        <Field label={t("common.partNumber")}>
          <Input name="replacement_part_number" defaultValue={item?.replacement_specification.part_number ?? ""} />
        </Field>
        <Field label={t("common.serialNumber")}>
          <Input name="replacement_serial_number" defaultValue={item?.replacement_specification.serial_number ?? ""} />
        </Field>
        <Field label={t("machines.supplierReference")}>
          <Input name="supplier_reference" defaultValue={item?.replacement_specification.supplier_reference ?? ""} />
        </Field>
        <Field label={t("machines.compatibilityNotes")} wide>
          <Textarea
            name="compatibility_notes"
            defaultValue={item?.replacement_specification.compatibility_notes ?? ""}
          />
        </Field>
        <div className="sm:col-span-2">
          <TechnicalSpecifications rows={replacementSpecifications} update={setReplacementSpecifications} />
        </div>
        <div className="sm:col-span-2">
          <EquivalentParts parts={equivalentParts} update={setEquivalentParts} />
        </div>
        <Field label={t("common.notes")} wide>
          <Textarea name="notes" defaultValue={item?.notes ?? ""} />
        </Field>
      </fieldset>
      <DialogFooter>
        <Button type="submit" disabled={pending || !categoryId}>
          {pending ? t("common.saving") : t("common.save")}
        </Button>
      </DialogFooter>
    </form>
  );
}

function ReplacementForm({
  item,
  pending,
  submit,
}: {
  item: MachineItem;
  pending: boolean;
  submit: (data: Record<string, unknown>) => void;
}) {
  const { t } = useI18n();
  const [technicalSpecifications, setTechnicalSpecifications] = useState<SpecificationRow[]>([]);
  function send(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = Object.fromEntries(new FormData(event.currentTarget));
    submit({
      id: item.id,
      reason: data.reason,
      installed_component: {
        manufacturer: data.manufacturer || null,
        model: data.model || null,
        part_number: data.part_number || null,
        serial_number: data.serial_number || null,
        installed_at: data.installed_at || null,
        technical_specifications: specificationRecord(technicalSpecifications),
      },
    });
  }
  return (
    <form className="grid gap-3 sm:grid-cols-2" onSubmit={send}>
      <Field label={t("machines.replacementReason")} wide>
        <Textarea name="reason" required />
      </Field>
      <Field label={t("common.manufacturer")}>
        <Input name="manufacturer" required />
      </Field>
      <Field label={t("common.model")}>
        <Input name="model" />
      </Field>
      <Field label={t("common.partNumber")}>
        <Input name="part_number" required />
      </Field>
      <Field label={t("common.serialNumber")}>
        <Input name="serial_number" required />
      </Field>
      <Field label={t("common.date")}>
        <Input name="installed_at" type="date" required />
      </Field>
      <div className="sm:col-span-2">
        <TechnicalSpecifications rows={technicalSpecifications} update={setTechnicalSpecifications} />
      </div>
      <DialogFooter className="sm:col-span-2">
        <Button type="submit" disabled={pending}>
          {pending ? t("common.saving") : t("common.replace")}
        </Button>
      </DialogFooter>
    </form>
  );
}

function TechnicalSpecifications({
  rows,
  update,
}: {
  rows: SpecificationRow[];
  update: (rows: SpecificationRow[]) => void;
}) {
  const { t } = useI18n();
  return (
    <fieldset className="space-y-2 border-t pt-4">
      <legend className="mb-2 font-semibold text-sm">{t("machines.technicalSpecifications")}</legend>
      {rows.map((row, index) => (
        <div key={row.id} className="grid gap-2 sm:grid-cols-[1fr_1fr_auto]">
          <Input
            value={row.key}
            placeholder={t("common.key")}
            aria-label={t("common.key")}
            onChange={(event) =>
              update(
                rows.map((entry, entryIndex) => (entryIndex === index ? { ...entry, key: event.target.value } : entry)),
              )
            }
          />
          <Input
            value={row.value}
            placeholder={t("common.value")}
            aria-label={t("common.value")}
            onChange={(event) =>
              update(
                rows.map((entry, entryIndex) =>
                  entryIndex === index ? { ...entry, value: event.target.value } : entry,
                ),
              )
            }
          />
          <Button
            type="button"
            size="icon"
            variant="ghost"
            title={t("common.remove")}
            onClick={() => update(rows.filter((_, entryIndex) => entryIndex !== index))}
          >
            <Trash2 />
          </Button>
        </div>
      ))}
      <Button
        type="button"
        variant="outline"
        size="sm"
        onClick={() => update([...rows, { id: crypto.randomUUID(), key: "", value: "" }])}
      >
        <Plus />
        {t("machines.addSpecification")}
      </Button>
    </fieldset>
  );
}

function EquivalentParts({ parts, update }: { parts: EquivalentDraft[]; update: (parts: EquivalentDraft[]) => void }) {
  const { t } = useI18n();
  return (
    <fieldset className="space-y-3 border-t pt-4">
      <legend className="mb-2 font-semibold text-sm">{t("machines.equivalentParts")}</legend>
      {parts.map((part, index) => (
        <div key={part.id} className="grid gap-2 rounded-md border p-3 sm:grid-cols-2">
          <Input
            value={part.manufacturer ?? ""}
            placeholder={t("common.manufacturer")}
            aria-label={t("common.manufacturer")}
            onChange={(event) =>
              update(
                parts.map((entry, entryIndex) =>
                  entryIndex === index ? { ...entry, manufacturer: event.target.value || null } : entry,
                ),
              )
            }
          />
          <Input
            value={part.part_number}
            placeholder={t("common.partNumber")}
            aria-label={t("common.partNumber")}
            onChange={(event) =>
              update(
                parts.map((entry, entryIndex) =>
                  entryIndex === index ? { ...entry, part_number: event.target.value } : entry,
                ),
              )
            }
          />
          <Input
            value={part.model ?? ""}
            placeholder={t("common.model")}
            aria-label={t("common.model")}
            onChange={(event) =>
              update(
                parts.map((entry, entryIndex) =>
                  entryIndex === index ? { ...entry, model: event.target.value || null } : entry,
                ),
              )
            }
          />
          <div className="flex gap-2">
            <Input
              value={part.notes ?? ""}
              placeholder={t("common.notes")}
              aria-label={t("common.notes")}
              onChange={(event) =>
                update(
                  parts.map((entry, entryIndex) =>
                    entryIndex === index ? { ...entry, notes: event.target.value || null } : entry,
                  ),
                )
              }
            />
            <Button
              type="button"
              size="icon"
              variant="ghost"
              title={t("common.remove")}
              onClick={() => update(parts.filter((_, entryIndex) => entryIndex !== index))}
            >
              <Trash2 />
            </Button>
          </div>
        </div>
      ))}
      <Button
        type="button"
        variant="outline"
        size="sm"
        onClick={() =>
          update([...parts, { id: crypto.randomUUID(), manufacturer: null, part_number: "", model: null, notes: null }])
        }
      >
        <Plus />
        {t("machines.addEquivalent")}
      </Button>
    </fieldset>
  );
}

function specificationRows(values?: Record<string, unknown>): SpecificationRow[] {
  return Object.entries(values ?? {}).map(([key, value]) => ({
    id: crypto.randomUUID(),
    key,
    value: String(value),
  }));
}

function specificationRecord(rows: SpecificationRow[]): Record<string, unknown> {
  return Object.fromEntries(
    rows.filter((row) => row.key.trim()).map((row) => [row.key.trim(), specificationValue(row.value)]),
  );
}

function specificationValue(value: string): string | number | boolean {
  const trimmed = value.trim();
  if (trimmed === "true") return true;
  if (trimmed === "false") return false;
  const numeric = Number(trimmed);
  return trimmed !== "" && Number.isFinite(numeric) ? numeric : trimmed;
}

function CustomField({
  definition,
  value,
  update,
}: {
  definition: ItemCategory["custom_field_definitions"][number];
  value: unknown;
  update: (value: unknown) => void;
}) {
  if (definition.field_type === "boolean")
    return (
      <label className="flex items-center gap-2 self-end pb-2 font-medium text-sm">
        <input type="checkbox" checked={value === true} onChange={(event) => update(event.target.checked)} />
        {definition.label}
      </label>
    );
  if (definition.field_type === "choice")
    return (
      <Field label={definition.label}>
        <select
          className="h-9 rounded-md border bg-background px-3 text-sm"
          value={String(value ?? "")}
          onChange={(event) => update(event.target.value)}
          required={definition.required}
        >
          <option value="" />
          {definition.options.map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
        </select>
      </Field>
    );
  return (
    <Field label={`${definition.label}${definition.unit ? ` (${definition.unit})` : ""}`}>
      <Input
        type={definition.field_type === "number" ? "number" : definition.field_type === "date" ? "date" : "text"}
        value={String(value ?? "")}
        required={definition.required}
        min={definition.minimum ?? undefined}
        max={definition.maximum ?? undefined}
        onChange={(event) =>
          update(definition.field_type === "number" ? Number(event.target.value) : event.target.value)
        }
      />
    </Field>
  );
}

function Field({ label, children, wide }: { label: string; children: React.ReactNode; wide?: boolean }) {
  return (
    <div className={`grid gap-2 ${wide ? "sm:col-span-2" : ""}`}>
      <Label>{label}</Label>
      {children}
    </div>
  );
}
