"use client";

import { useState } from "react";

import { useRouter } from "next/navigation";

import { ArrowDown, ArrowUp, Pencil, Plus, Tags, Trash2 } from "lucide-react";
import { toast } from "sonner";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { useI18n } from "@/lib/i18n/provider";
import type {
  ComplexityLevel,
  CustomFieldDefinition,
  CustomFieldType,
  GuideStepType,
  ItemCategory,
  MaintenanceGuideStep,
  PhotoAsset,
  RecommendedPart,
} from "@/lib/proexel/types";

import { AssetPhotoManager } from "../machines/[id]/asset-photo-manager";

type Draft = {
  id?: string;
  code: string;
  name: string;
  description: string;
  icon: string;
  default_complexity_level: ComplexityLevel;
  active: boolean;
  guideVersion: number;
  fields: CustomFieldDefinition[];
  steps: MaintenanceGuideStep[];
  parts: RecommendedPart[];
  guidePhotos: PhotoAsset[];
};

const EMPTY: Draft = {
  code: "",
  name: "",
  description: "",
  icon: "",
  default_complexity_level: 3,
  active: true,
  guideVersion: 1,
  fields: [],
  steps: [],
  parts: [],
  guidePhotos: [],
};

const FIELD_TYPES: CustomFieldType[] = ["text", "number", "boolean", "choice", "date"];
const STEP_TYPES: GuideStepType[] = [
  "confirmation",
  "boolean",
  "choice",
  "numeric",
  "text",
  "photo",
  "measurement",
  "information",
  "warning",
];

export function CategoryEditor({ categories }: { readonly categories: ItemCategory[] }) {
  const { t } = useI18n();
  const router = useRouter();
  const [draft, setDraft] = useState<Draft | null>(null);
  const [pending, setPending] = useState(false);

  function edit(category?: ItemCategory) {
    setDraft(
      category
        ? {
            id: category.id,
            code: category.code,
            name: category.name,
            description: category.description ?? "",
            icon: category.icon ?? "",
            default_complexity_level: category.default_complexity_level,
            active: category.active,
            guideVersion: category.maintenance_guide.version,
            fields: structuredClone(category.custom_field_definitions),
            steps: structuredClone(category.maintenance_guide.steps),
            parts: structuredClone(category.recommended_parts),
            guidePhotos: structuredClone(category.guide_photos),
          }
        : structuredClone(EMPTY),
    );
  }

  async function save() {
    if (!draft) return;
    setPending(true);
    const payload = {
      ...(draft.id ? { id: draft.id } : {}),
      code: draft.code,
      name: draft.name,
      description: draft.description || null,
      icon: draft.icon || null,
      default_complexity_level: draft.default_complexity_level,
      active: draft.active,
      custom_field_definitions: draft.fields.map((field, order) => ({ ...field, order })),
      maintenance_guide: {
        version: draft.guideVersion,
        steps: draft.steps.map((step, order) => ({ ...step, order })),
      },
      recommended_parts: draft.parts,
    };
    try {
      const response = await fetch("/api/proexel/categories", {
        method: draft.id ? "PATCH" : "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(payload),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string };
      if (!response.ok || !result.accepted) throw new Error(result.message ?? t("command.rejected"));
      toast.success(t("command.success"));
      setDraft(null);
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t("command.failed"));
    } finally {
      setPending(false);
    }
  }

  return (
    <>
      <div className="mb-4 flex justify-end">
        <Button onClick={() => edit()}>
          <Plus />
          {t("categories.new")}
        </Button>
      </div>
      {categories.length ? (
        <div className="grid gap-3 lg:grid-cols-2 2xl:grid-cols-3">
          {categories.map((category) => (
            <Card key={category.id}>
              <CardHeader className="flex-row items-start justify-between gap-3">
                <div className="min-w-0">
                  <CardTitle className="truncate text-base">{category.name}</CardTitle>
                  <CardDescription>{category.code}</CardDescription>
                </div>
                <Button size="icon-sm" variant="ghost" title={t("categories.edit")} onClick={() => edit(category)}>
                  <Pencil />
                </Button>
              </CardHeader>
              <CardContent className="space-y-3 text-sm">
                <p className="line-clamp-2 text-muted-foreground">{category.description || "-"}</p>
                <div className="flex flex-wrap gap-2">
                  <Badge variant={category.active ? "outline" : "secondary"}>
                    {category.active ? t("common.active") : t("common.inactive")}
                  </Badge>
                  <Badge variant="secondary">
                    {t("common.complexity")} {category.default_complexity_level}/5
                  </Badge>
                  <Badge variant="secondary">
                    {category.custom_field_definitions.length} {t("categories.fields")}
                  </Badge>
                  <Badge variant="secondary">
                    {category.maintenance_guide.steps.length} {t("categories.guide")}
                  </Badge>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <Card>
          <CardContent className="flex min-h-56 flex-col items-center justify-center text-center">
            <Tags className="mb-3 size-8 text-muted-foreground" />
            <h2 className="font-semibold">{t("categories.none")}</h2>
            <p className="mt-1 text-muted-foreground text-sm">{t("categories.noneDescription")}</p>
          </CardContent>
        </Card>
      )}

      <Dialog open={draft !== null} onOpenChange={(open) => !open && setDraft(null)}>
        <DialogContent className="max-h-[94vh] overflow-y-auto sm:max-w-5xl">
          <DialogHeader>
            <DialogTitle>{draft?.id ? t("categories.edit") : t("categories.new")}</DialogTitle>
            <DialogDescription>{t("categories.description")}</DialogDescription>
          </DialogHeader>
          {draft ? <Editor draft={draft} onChange={setDraft} /> : null}
          <DialogFooter>
            <Button variant="outline" onClick={() => setDraft(null)}>
              {t("common.cancel")}
            </Button>
            <Button disabled={pending || !draft?.code.trim() || !draft.name.trim()} onClick={save}>
              {pending ? t("common.saving") : t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

function Editor({ draft, onChange }: { draft: Draft; onChange: (draft: Draft) => void }) {
  const { t } = useI18n();
  const update = <K extends keyof Draft>(key: K, value: Draft[K]) => onChange({ ...draft, [key]: value });
  return (
    <Tabs defaultValue="properties" className="min-h-[520px] gap-4">
      <TabsList className="flex h-auto flex-wrap justify-start">
        <TabsTrigger value="properties">{t("categories.properties")}</TabsTrigger>
        <TabsTrigger value="fields">{t("categories.fields")}</TabsTrigger>
        <TabsTrigger value="guide">{t("categories.guide")}</TabsTrigger>
        <TabsTrigger value="parts">{t("categories.parts")}</TabsTrigger>
      </TabsList>
      <TabsContent value="properties" className="grid gap-4 md:grid-cols-2">
        <Field label={t("common.code")}>
          <Input value={draft.code} onChange={(event) => update("code", event.target.value)} />
        </Field>
        <Field label={t("common.name")}>
          <Input value={draft.name} onChange={(event) => update("name", event.target.value)} />
        </Field>
        <Field label={t("categories.icon")}>
          <Input value={draft.icon} onChange={(event) => update("icon", event.target.value)} />
        </Field>
        <Field label={t("categories.defaultComplexity")}>
          <ComplexitySelect
            value={draft.default_complexity_level}
            onChange={(value) => update("default_complexity_level", value)}
          />
        </Field>
        <Field label={t("common.description")} wide>
          <Textarea value={draft.description} onChange={(event) => update("description", event.target.value)} />
        </Field>
        <label className="flex items-center gap-2 font-medium text-sm">
          <input type="checkbox" checked={draft.active} onChange={(event) => update("active", event.target.checked)} />
          {t("common.active")}
        </label>
      </TabsContent>
      <TabsContent value="fields" className="space-y-3">
        {draft.fields.map((field, index) => (
          <div key={field.id} className="grid gap-3 border-b pb-4 md:grid-cols-12">
            <div className="md:col-span-2">
              <Field label={t("categories.fieldKey")}>
                <Input
                  value={field.key}
                  onChange={(event) => updateField(draft, onChange, index, { key: event.target.value })}
                />
              </Field>
            </div>
            <div className="md:col-span-3">
              <Field label={t("categories.fieldLabel")}>
                <Input
                  value={field.label}
                  onChange={(event) => updateField(draft, onChange, index, { label: event.target.value })}
                />
              </Field>
            </div>
            <div className="md:col-span-2">
              <Field label={t("categories.fieldType")}>
                <select
                  className="h-9 w-full rounded-md border bg-background px-3 text-sm"
                  value={field.field_type}
                  onChange={(event) =>
                    updateField(draft, onChange, index, { field_type: event.target.value as CustomFieldType })
                  }
                >
                  {FIELD_TYPES.map((type) => (
                    <option key={type} value={type}>
                      {t(`categories.field.${type}`)}
                    </option>
                  ))}
                </select>
              </Field>
            </div>
            <div className="md:col-span-3">
              <Field label={t("categories.options")}>
                <Input
                  value={field.options.join(", ")}
                  onChange={(event) => updateField(draft, onChange, index, { options: csv(event.target.value) })}
                />
              </Field>
            </div>
            <div className="flex items-end gap-1 md:col-span-2">
              <label className="mb-2 flex items-center gap-1 text-sm">
                <input
                  type="checkbox"
                  checked={field.required}
                  onChange={(event) => updateField(draft, onChange, index, { required: event.target.checked })}
                />
                {t("common.required")}
              </label>
              <MoveButtons
                index={index}
                length={draft.fields.length}
                move={(direction) => update("fields", move(draft.fields, index, direction))}
                remove={() =>
                  update(
                    "fields",
                    draft.fields.filter((_, itemIndex) => itemIndex !== index),
                  )
                }
              />
            </div>
            {field.field_type === "number" ? (
              <>
                <div className="md:col-span-2">
                  <Field label={t("common.unit")}>
                    <Input
                      value={field.unit ?? ""}
                      onChange={(event) => updateField(draft, onChange, index, { unit: event.target.value || null })}
                    />
                  </Field>
                </div>
                <div className="md:col-span-2">
                  <Field label={t("common.minimum")}>
                    <Input
                      type="number"
                      value={field.minimum ?? ""}
                      onChange={(event) =>
                        updateField(draft, onChange, index, { minimum: optionalNumber(event.target.value) })
                      }
                    />
                  </Field>
                </div>
                <div className="md:col-span-2">
                  <Field label={t("common.maximum")}>
                    <Input
                      type="number"
                      value={field.maximum ?? ""}
                      onChange={(event) =>
                        updateField(draft, onChange, index, { maximum: optionalNumber(event.target.value) })
                      }
                    />
                  </Field>
                </div>
              </>
            ) : null}
          </div>
        ))}
        <Button variant="outline" onClick={() => update("fields", [...draft.fields, newField(draft.fields.length)])}>
          <Plus />
          {t("categories.addField")}
        </Button>
      </TabsContent>
      <TabsContent value="guide" className="space-y-3">
        {draft.steps.map((step, index) => (
          <div key={step.id} className="space-y-3 border-b pb-5">
            <div className="flex items-center justify-between gap-3">
              <h3 className="font-semibold text-sm">{t("guide.step", { number: index + 1 })}</h3>
              <MoveButtons
                index={index}
                length={draft.steps.length}
                move={(direction) => update("steps", move(draft.steps, index, direction))}
                remove={() =>
                  update(
                    "steps",
                    draft.steps.filter((_, stepIndex) => stepIndex !== index),
                  )
                }
              />
            </div>
            <div className="grid gap-3 md:grid-cols-3">
              <Field label={t("guide.title")}>
                <Input
                  value={step.title}
                  onChange={(event) => updateStep(draft, onChange, index, { title: event.target.value })}
                />
              </Field>
              <Field label={t("common.type")}>
                <select
                  className="h-9 rounded-md border bg-background px-3 text-sm"
                  value={step.step_type}
                  onChange={(event) =>
                    updateStep(draft, onChange, index, { step_type: event.target.value as GuideStepType })
                  }
                >
                  {STEP_TYPES.map((type) => (
                    <option key={type} value={type}>
                      {t(`guide.type.${type}`)}
                    </option>
                  ))}
                </select>
              </Field>
              <label className="flex items-end gap-2 pb-2 font-medium text-sm">
                <input
                  type="checkbox"
                  checked={step.required}
                  onChange={(event) => updateStep(draft, onChange, index, { required: event.target.checked })}
                />
                {t("common.required")}
              </label>
              <Field label={t("guide.instructions")} wide>
                <Textarea
                  value={step.instructions}
                  onChange={(event) => updateStep(draft, onChange, index, { instructions: event.target.value })}
                />
              </Field>
              <Field label={t("common.description")}>
                <Input
                  value={step.description ?? ""}
                  onChange={(event) => updateStep(draft, onChange, index, { description: event.target.value || null })}
                />
              </Field>
              <Field label={t("guide.safetyWarning")}>
                <Input
                  value={step.safety_warning ?? ""}
                  onChange={(event) =>
                    updateStep(draft, onChange, index, { safety_warning: event.target.value || null })
                  }
                />
              </Field>
              {step.step_type === "choice" ? (
                <Field label={t("categories.options")}>
                  <Input
                    value={step.options.join(", ")}
                    onChange={(event) => updateStep(draft, onChange, index, { options: csv(event.target.value) })}
                  />
                </Field>
              ) : null}
              {step.step_type === "numeric" || step.step_type === "measurement" ? (
                <ExpectedFields
                  step={step}
                  update={(expected_value) => updateStep(draft, onChange, index, { expected_value })}
                />
              ) : null}
            </div>
            {draft.id ? (
              <AssetPhotoManager
                ownerType="guide_step"
                ownerId={step.id}
                photos={draft.guidePhotos.filter((photo) => photo.owner_id === step.id)}
                kind="guide-photos"
                canEdit
                defaultPurpose="reference"
                onPhotoAdded={(photo) =>
                  onChange({
                    ...draft,
                    guidePhotos: [...draft.guidePhotos, photo],
                    steps: draft.steps.map((entry, stepIndex) =>
                      stepIndex === index
                        ? { ...entry, reference_photo_ids: [...entry.reference_photo_ids, photo.id] }
                        : entry,
                    ),
                  })
                }
                onPhotoRemoved={(photo) =>
                  onChange({
                    ...draft,
                    guidePhotos: draft.guidePhotos.filter((entry) => entry.id !== photo.id),
                    steps: draft.steps.map((entry, stepIndex) =>
                      stepIndex === index
                        ? {
                            ...entry,
                            reference_photo_ids: entry.reference_photo_ids.filter((id) => id !== photo.id),
                          }
                        : entry,
                    ),
                  })
                }
              />
            ) : (
              <p className="text-muted-foreground text-sm">{t("guide.saveBeforePhotos")}</p>
            )}
          </div>
        ))}
        <Button variant="outline" onClick={() => update("steps", [...draft.steps, newStep(draft.steps.length)])}>
          <Plus />
          {t("guide.addStep")}
        </Button>
      </TabsContent>
      <TabsContent value="parts" className="space-y-3">
        {draft.parts.map((part, index) => (
          <div
            key={`${part.manufacturer ?? ""}-${part.part_number}-${part.description ?? ""}`}
            className="grid gap-3 border-b pb-4 md:grid-cols-[1fr_1fr_2fr_auto]"
          >
            <Field label={t("common.manufacturer")}>
              <Input
                value={part.manufacturer ?? ""}
                onChange={(event) => updatePart(draft, onChange, index, { manufacturer: event.target.value || null })}
              />
            </Field>
            <Field label={t("common.partNumber")}>
              <Input
                value={part.part_number}
                onChange={(event) => updatePart(draft, onChange, index, { part_number: event.target.value })}
              />
            </Field>
            <Field label={t("categories.partDescription")}>
              <Input
                value={part.description ?? ""}
                onChange={(event) => updatePart(draft, onChange, index, { description: event.target.value || null })}
              />
            </Field>
            <Button
              className="self-end"
              size="icon-sm"
              variant="ghost"
              title={t("common.remove")}
              onClick={() =>
                update(
                  "parts",
                  draft.parts.filter((_, partIndex) => partIndex !== index),
                )
              }
            >
              <Trash2 />
            </Button>
          </div>
        ))}
        <Button
          variant="outline"
          onClick={() => update("parts", [...draft.parts, { manufacturer: null, part_number: "", description: null }])}
        >
          <Plus />
          {t("categories.addPart")}
        </Button>
      </TabsContent>
    </Tabs>
  );
}

function ExpectedFields({
  step,
  update,
}: {
  step: MaintenanceGuideStep;
  update: (value: MaintenanceGuideStep["expected_value"]) => void;
}) {
  const { t } = useI18n();
  const value = step.expected_value ?? { unit: null, minimum: null, maximum: null, target: null };
  return (
    <>
      <Field label={t("common.unit")}>
        <Input value={value.unit ?? ""} onChange={(event) => update({ ...value, unit: event.target.value || null })} />
      </Field>
      <Field label={t("common.minimum")}>
        <Input
          type="number"
          value={value.minimum ?? ""}
          onChange={(event) => update({ ...value, minimum: optionalNumber(event.target.value) })}
        />
      </Field>
      <Field label={t("common.maximum")}>
        <Input
          type="number"
          value={value.maximum ?? ""}
          onChange={(event) => update({ ...value, maximum: optionalNumber(event.target.value) })}
        />
      </Field>
    </>
  );
}

function MoveButtons({
  index,
  length,
  move: onMove,
  remove,
}: {
  index: number;
  length: number;
  move: (direction: -1 | 1) => void;
  remove: () => void;
}) {
  const { t } = useI18n();
  return (
    <div className="ml-auto flex gap-1">
      <Button
        size="icon-sm"
        variant="ghost"
        disabled={index === 0}
        title={t("common.previous")}
        onClick={() => onMove(-1)}
      >
        <ArrowUp />
      </Button>
      <Button
        size="icon-sm"
        variant="ghost"
        disabled={index === length - 1}
        title={t("common.next")}
        onClick={() => onMove(1)}
      >
        <ArrowDown />
      </Button>
      <Button size="icon-sm" variant="ghost" title={t("common.remove")} onClick={remove}>
        <Trash2 />
      </Button>
    </div>
  );
}

function Field({ label, children, wide }: { label: string; children: React.ReactNode; wide?: boolean }) {
  return (
    <div className={`grid gap-2 ${wide ? "md:col-span-2" : ""}`}>
      <Label>{label}</Label>
      {children}
    </div>
  );
}

function ComplexitySelect({ value, onChange }: { value: ComplexityLevel; onChange: (value: ComplexityLevel) => void }) {
  const { t } = useI18n();
  return (
    <select
      className="h-9 rounded-md border bg-background px-3 text-sm"
      value={value}
      onChange={(event) => onChange(Number(event.target.value) as ComplexityLevel)}
    >
      {([1, 2, 3, 4, 5] as const).map((level) => (
        <option key={level} value={level}>
          {level} - {t(`complexity.${level}`)}
        </option>
      ))}
    </select>
  );
}

function newField(order: number): CustomFieldDefinition {
  return {
    id: `field-${crypto.randomUUID()}`,
    key: "",
    label: "",
    field_type: "text",
    required: false,
    unit: null,
    options: [],
    minimum: null,
    maximum: null,
    order,
  };
}

function newStep(order: number): MaintenanceGuideStep {
  return {
    id: `step-${crypto.randomUUID()}`,
    title: "",
    description: null,
    instructions: "",
    step_type: "confirmation",
    required: true,
    reference_photo_ids: [],
    safety_warning: null,
    expected_value: null,
    options: [],
    order,
  };
}

function updateField(
  draft: Draft,
  onChange: (draft: Draft) => void,
  index: number,
  patch: Partial<CustomFieldDefinition>,
) {
  onChange({
    ...draft,
    fields: draft.fields.map((field, itemIndex) => (itemIndex === index ? { ...field, ...patch } : field)),
  });
}
function updateStep(
  draft: Draft,
  onChange: (draft: Draft) => void,
  index: number,
  patch: Partial<MaintenanceGuideStep>,
) {
  onChange({
    ...draft,
    steps: draft.steps.map((step, stepIndex) => (stepIndex === index ? { ...step, ...patch } : step)),
  });
}
function updatePart(draft: Draft, onChange: (draft: Draft) => void, index: number, patch: Partial<RecommendedPart>) {
  onChange({
    ...draft,
    parts: draft.parts.map((part, partIndex) => (partIndex === index ? { ...part, ...patch } : part)),
  });
}

function move<T>(items: T[], index: number, direction: -1 | 1) {
  const next = [...items];
  const target = index + direction;
  [next[index], next[target]] = [next[target], next[index]];
  return next;
}
function csv(value: string) {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}
function optionalNumber(value: string) {
  return value === "" ? null : Number(value);
}
