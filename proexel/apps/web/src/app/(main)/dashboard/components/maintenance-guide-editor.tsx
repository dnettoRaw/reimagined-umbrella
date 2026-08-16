"use client";

import { ArrowDown, ArrowUp, Plus, ShieldCheck, Trash2 } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { useI18n } from "@/lib/i18n/provider";
import type { GuideStepType, MaintenanceGuideStep } from "@/lib/proexel/types";

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

export function MaintenanceGuideEditor({
  steps,
  onChange,
}: {
  steps: MaintenanceGuideStep[];
  onChange: (steps: MaintenanceGuideStep[]) => void;
}) {
  const { t } = useI18n();
  const update = (index: number, patch: Partial<MaintenanceGuideStep>) =>
    onChange(steps.map((step, stepIndex) => (stepIndex === index ? { ...step, ...patch } : step)));
  const move = (index: number, direction: -1 | 1) => {
    const next = [...steps];
    const target = index + direction;
    [next[index], next[target]] = [next[target], next[index]];
    onChange(next.map((step, order) => ({ ...step, order })));
  };

  return (
    <div className="space-y-4">
      <Alert>
        <ShieldCheck />
        <AlertTitle>{t("components.standardWorkflow")}</AlertTitle>
        <AlertDescription>{t("components.standardWorkflowDescription")}</AlertDescription>
      </Alert>
      {steps.map((step, index) => (
        <section key={step.id} className="space-y-3 border-b pb-5">
          <div className="flex items-center justify-between gap-3">
            <h3 className="font-semibold text-sm">{t("guide.step", { number: index + 1 })}</h3>
            <div className="flex gap-1">
              <Button
                type="button"
                size="icon-sm"
                variant="ghost"
                disabled={index === 0}
                title={t("common.previous")}
                onClick={() => move(index, -1)}
              >
                <ArrowUp />
              </Button>
              <Button
                type="button"
                size="icon-sm"
                variant="ghost"
                disabled={index === steps.length - 1}
                title={t("common.next")}
                onClick={() => move(index, 1)}
              >
                <ArrowDown />
              </Button>
              <Button
                type="button"
                size="icon-sm"
                variant="ghost"
                title={t("common.delete")}
                onClick={() => onChange(steps.filter((_, stepIndex) => stepIndex !== index))}
              >
                <Trash2 />
              </Button>
            </div>
          </div>
          <div className="grid gap-3 sm:grid-cols-2">
            <Field label={t("guide.title")}>
              <Input value={step.title} required onChange={(event) => update(index, { title: event.target.value })} />
            </Field>
            <Field label={t("common.type")}>
              <select
                className="h-9 w-full rounded-md border bg-background px-3 text-sm"
                value={step.step_type}
                onChange={(event) => update(index, { step_type: event.target.value as GuideStepType })}
              >
                {STEP_TYPES.map((type) => (
                  <option key={type} value={type}>
                    {t(`guide.type.${type}`)}
                  </option>
                ))}
              </select>
            </Field>
            <Field label={t("guide.instructions")} wide>
              <Textarea
                value={step.instructions}
                required
                onChange={(event) => update(index, { instructions: event.target.value })}
              />
            </Field>
            <Field label={t("common.description")}>
              <Textarea
                className="min-h-16"
                value={step.description ?? ""}
                onChange={(event) => update(index, { description: event.target.value || null })}
              />
            </Field>
            <Field label={t("guide.safetyWarning")}>
              <Textarea
                className="min-h-16"
                value={step.safety_warning ?? ""}
                onChange={(event) => update(index, { safety_warning: event.target.value || null })}
              />
            </Field>
            {step.step_type === "choice" ? (
              <Field label={t("categories.options")} wide>
                <Input
                  value={step.options.join(", ")}
                  onChange={(event) =>
                    update(index, {
                      options: event.target.value
                        .split(",")
                        .map((value) => value.trim())
                        .filter(Boolean),
                    })
                  }
                />
              </Field>
            ) : null}
            {step.step_type === "numeric" || step.step_type === "measurement" ? (
              <div className="grid gap-3 sm:col-span-2 sm:grid-cols-3">
                <Field label={t("common.unit")}>
                  <Input
                    value={step.expected_value?.unit ?? ""}
                    onChange={(event) =>
                      update(index, { expected_value: { ...step.expected_value, unit: event.target.value || null } })
                    }
                  />
                </Field>
                <Field label={t("common.minimum")}>
                  <Input
                    type="number"
                    value={step.expected_value?.minimum ?? ""}
                    onChange={(event) =>
                      update(index, {
                        expected_value: {
                          ...step.expected_value,
                          minimum: event.target.value ? Number(event.target.value) : null,
                        },
                      })
                    }
                  />
                </Field>
                <Field label={t("common.maximum")}>
                  <Input
                    type="number"
                    value={step.expected_value?.maximum ?? ""}
                    onChange={(event) =>
                      update(index, {
                        expected_value: {
                          ...step.expected_value,
                          maximum: event.target.value ? Number(event.target.value) : null,
                        },
                      })
                    }
                  />
                </Field>
              </div>
            ) : null}
            <label className="flex items-center gap-2 text-sm sm:col-span-2">
              <input
                type="checkbox"
                checked={step.required}
                onChange={(event) => update(index, { required: event.target.checked })}
              />
              {t("common.required")}
            </label>
          </div>
        </section>
      ))}
      <Button type="button" variant="outline" onClick={() => onChange([...steps, newStep(steps.length)])}>
        <Plus />
        {t("guide.addStep")}
      </Button>
    </div>
  );
}

function Field({ label, children, wide = false }: { label: string; children: React.ReactNode; wide?: boolean }) {
  return (
    <div className={wide ? "sm:col-span-2" : undefined}>
      <Label className="mb-1.5">{label}</Label>
      {children}
    </div>
  );
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
