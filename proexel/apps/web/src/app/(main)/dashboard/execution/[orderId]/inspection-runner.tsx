"use client";

import { useState } from "react";

import Image from "next/image";
import { useRouter } from "next/navigation";

import { AlertTriangle, Camera, Check, ChevronLeft, ChevronRight, Circle, Info, Play } from "lucide-react";
import { toast } from "sonner";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import { Textarea } from "@/components/ui/textarea";
import { useI18n } from "@/lib/i18n/provider";
import type {
  InspectionStepResult,
  ItemInspection,
  MachineItem,
  MaintenanceGuideStep,
  OperationalStatus,
  PhotoPurpose,
  Role,
  ServiceOrder,
  ServiceOrderTask,
} from "@/lib/proexel/types";

export function InspectionRunner({
  order,
  inspections,
  currentItems,
  session,
}: {
  order: ServiceOrder;
  inspections: ItemInspection[];
  currentItems: MachineItem[];
  session: { id: string; role: Role; maximumRepairLevel: number };
}) {
  const { t } = useI18n();
  const router = useRouter();
  const [pending, setPending] = useState(false);
  const [selectedTaskId, setSelectedTaskId] = useState(
    order.tasks.find((task) => task.status !== "completed")?.id ?? order.tasks[0]?.id ?? "",
  );
  const selectedTask = order.tasks.find((task) => task.id === selectedTaskId);
  const inspection = inspections.find(
    (entry) => entry.service_order_task_id === selectedTaskId && entry.status === "in_progress",
  );

  async function command(endpoint: string, method: string, data: Record<string, unknown>) {
    setPending(true);
    try {
      const response = await fetch(endpoint, {
        method,
        headers: { "content-type": "application/json" },
        body: JSON.stringify(data),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string };
      if (!response.ok || !result.accepted) throw new Error(result.message ?? t("command.rejected"));
      toast.success(t("command.success"));
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t("command.failed"));
    } finally {
      setPending(false);
    }
  }

  if (order.status === "pending")
    return (
      <Card>
        <CardHeader>
          <CardTitle>{order.machine_snapshot.name}</CardTitle>
          <CardDescription>{t("orders.maxLevel", { level: order.maximum_complexity_level })}</CardDescription>
        </CardHeader>
        <CardContent>
          <Button
            disabled={pending || session.maximumRepairLevel < order.maximum_complexity_level}
            onClick={() => command("/api/proexel/orders", "PATCH", { id: order.id })}
          >
            <Play />
            {t("execution.startOrder")}
          </Button>
          {session.maximumRepairLevel < order.maximum_complexity_level ? (
            <p className="mt-2 text-destructive text-sm">{t("execution.levelBlocked")}</p>
          ) : null}
        </CardContent>
      </Card>
    );

  const completed = order.tasks.filter((task) => task.status === "completed").length;
  return (
    <div className="grid gap-4 xl:grid-cols-[300px_minmax(0,1fr)]">
      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t("common.components")}</CardTitle>
          <CardDescription>{t("orders.progress", { completed, total: order.tasks.length })}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <Progress value={order.tasks.length ? (completed / order.tasks.length) * 100 : 0} />
          <div className="divide-y rounded-md border">
            {order.tasks.map((task) => (
              <button
                key={task.id}
                type="button"
                className={`flex w-full items-center gap-2 p-3 text-left text-sm ${task.id === selectedTaskId ? "bg-muted" : ""}`}
                onClick={() => setSelectedTaskId(task.id)}
              >
                {task.status === "completed" ? (
                  <Check className="size-4 text-emerald-600" />
                ) : task.status === "in_progress" ? (
                  <Play className="size-4" />
                ) : (
                  <Circle className="size-4" />
                )}
                <span className="min-w-0 flex-1">
                  <strong className="block truncate">{task.item_snapshot.code}</strong>
                  <span className="block truncate text-muted-foreground text-xs">{task.item_snapshot.name}</span>
                </span>
                <Badge variant="secondary">{task.complexity_snapshot}/5</Badge>
              </button>
            ))}
          </div>
          {completed === order.tasks.length && order.status === "in_progress" ? (
            <Button
              className="w-full"
              disabled={pending}
              onClick={() => command("/api/proexel/orders", "PUT", { id: order.id })}
            >
              <Check />
              {t("execution.completeOrder")}
            </Button>
          ) : null}
        </CardContent>
      </Card>
      {selectedTask ? (
        <Task
          task={selectedTask}
          order={order}
          inspection={inspection}
          currentItem={currentItems.find((item) => item.id === selectedTask.machine_item_id)}
          session={session}
          pending={pending}
          start={() => command("/api/proexel/inspections", "POST", { order_id: order.id, task_id: selectedTask.id })}
          complete={(data) => command("/api/proexel/inspections", "PATCH", data)}
        />
      ) : null}
    </div>
  );
}

function Task({
  task,
  inspection,
  currentItem,
  session,
  pending,
  start,
  complete,
}: {
  task: ServiceOrderTask;
  order: ServiceOrder;
  inspection?: ItemInspection;
  currentItem?: MachineItem;
  session: { id: string; role: Role; maximumRepairLevel: number };
  pending: boolean;
  start: () => void;
  complete: (data: Record<string, unknown>) => void;
}) {
  const { t } = useI18n();
  const blocked =
    task.complexity_snapshot > session.maximumRepairLevel ||
    Boolean(task.assigned_operator_id && task.assigned_operator_id !== session.id && session.role === "tecnico");
  return (
    <Card>
      <CardHeader>
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <CardTitle>
              {task.item_snapshot.code} · {task.item_snapshot.name}
            </CardTitle>
            <CardDescription>
              {task.item_snapshot.category.name} · {t("common.complexity")} {task.complexity_snapshot}/5
            </CardDescription>
          </div>
          <Badge
            variant={task.status === "completed" ? "outline" : task.status === "in_progress" ? "default" : "secondary"}
          >
            {task.status === "completed"
              ? t("common.completed")
              : task.status === "in_progress"
                ? t("common.inProgress")
                : t("common.pending")}
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        {task.item_snapshot.location_description ? (
          <p className="mb-4 whitespace-pre-wrap rounded-md border p-3 text-sm">
            <strong>{t("common.location")}:</strong> {task.item_snapshot.location_description}
          </p>
        ) : null}
        {currentItem?.photos.length ? (
          <div className="mb-4">
            <h3 className="mb-2 font-semibold text-sm">{t("execution.referencePhotos")}</h3>
            <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
              {currentItem.photos
                .filter((photo) => photo.purpose === "reference" || photo.purpose === "general")
                .map((photo) => (
                  <Image
                    key={photo.id}
                    unoptimized
                    src={`/api/proexel/attachments?ref=${encodeURIComponent(photo.blob_ref)}`}
                    alt={photo.description ?? t("common.photo")}
                    width={400}
                    height={280}
                    className="aspect-[3/2] w-full rounded-md border object-cover"
                  />
                ))}
            </div>
          </div>
        ) : null}
        {blocked ? (
          <p className="text-destructive text-sm">{t("execution.levelBlocked")}</p>
        ) : task.status === "pending" ? (
          <Button disabled={pending} onClick={start}>
            <Play />
            {t("execution.startInspection")}
          </Button>
        ) : task.status === "in_progress" && inspection ? (
          <Guide inspection={inspection} complete={complete} pending={pending} />
        ) : null}
      </CardContent>
    </Card>
  );
}

function Guide({
  inspection,
  complete,
  pending,
}: {
  inspection: ItemInspection;
  complete: (data: Record<string, unknown>) => void;
  pending: boolean;
}) {
  const { t } = useI18n();
  const steps = inspection.category_snapshot.maintenance_guide.steps;
  const [index, setIndex] = useState(0);
  const [results, setResults] = useState<Record<string, InspectionStepResult>>(() =>
    Object.fromEntries(
      steps
        .filter((step) => step.step_type === "information" || step.step_type === "warning")
        .map((step) => [step.id, { step_id: step.id, value: true, unit: null, photo_ids: [] }]),
    ),
  );
  const [statusAfter, setStatusAfter] = useState<OperationalStatus>("ok");
  const [notes, setNotes] = useState("");
  const [action, setAction] = useState("");
  const [finding, setFinding] = useState("");
  const [findingSeverity, setFindingSeverity] = useState<OperationalStatus>("attention");
  const step = steps[index];
  const current = step ? results[step.id] : undefined;
  const requiredComplete = steps.every(
    (entry) =>
      !entry.required ||
      (results[entry.id] !== undefined && (entry.step_type !== "photo" || results[entry.id]?.photo_ids.length)),
  );
  function setResult(value: unknown, unit: string | null = null, photoIds: string[] = current?.photo_ids ?? []) {
    if (!step) return;
    setResults((all) => ({ ...all, [step.id]: { step_id: step.id, value, unit, photo_ids: photoIds } }));
  }
  function submit() {
    complete({
      id: inspection.id,
      status_after: statusAfter,
      step_results: Object.values(results),
      findings: finding.trim()
        ? [{ description: finding.trim(), severity: findingSeverity, action_required: action || null }]
        : [],
      photo_ids: Object.values(results).flatMap((result) => result.photo_ids),
      notes: notes || null,
      maintenance_action: action || null,
    });
  }
  return (
    <div className="space-y-5">
      {steps.length && step ? (
        <>
          <div>
            <div className="mb-1 flex items-center justify-between text-sm">
              <span>{t("execution.stepProgress", { current: index + 1, total: steps.length })}</span>
              <span>{Math.round(((index + 1) / steps.length) * 100)}%</span>
            </div>
            <Progress value={((index + 1) / steps.length) * 100} />
          </div>
          <section className="min-h-72 rounded-md border p-4">
            {step.safety_warning ? (
              <div className="mb-4 flex gap-2 rounded-md border border-amber-500/50 bg-amber-500/10 p-3 text-sm">
                <AlertTriangle className="size-5 shrink-0" />
                {step.safety_warning}
              </div>
            ) : null}
            <div className="mb-3 flex items-center gap-2">
              {step.step_type === "warning" ? (
                <AlertTriangle className="size-5" />
              ) : step.step_type === "information" ? (
                <Info className="size-5" />
              ) : null}
              <h3 className="font-semibold">{step.title}</h3>
              {step.required ? <Badge variant="secondary">{t("common.required")}</Badge> : null}
            </div>
            {step.description ? <p className="mb-2 text-muted-foreground text-sm">{step.description}</p> : null}
            <p className="mb-5 whitespace-pre-wrap text-sm">{step.instructions}</p>
            {inspection.category_snapshot.guide_reference_photos.some(
              (photo) => photo.owner_id === step.id && step.reference_photo_ids.includes(photo.id),
            ) ? (
              <div className="mb-5">
                <h4 className="mb-2 font-medium text-sm">{t("execution.referencePhotos")}</h4>
                <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                  {inspection.category_snapshot.guide_reference_photos
                    .filter((photo) => photo.owner_id === step.id && step.reference_photo_ids.includes(photo.id))
                    .map((photo) => (
                      <Image
                        key={photo.id}
                        unoptimized
                        src={`/api/proexel/attachments?ref=${encodeURIComponent(photo.blob_ref)}`}
                        alt={photo.description ?? t("common.photo")}
                        width={400}
                        height={280}
                        className="aspect-[3/2] w-full rounded-md border object-cover"
                      />
                    ))}
                </div>
              </div>
            ) : null}
            <StepInput step={step} result={current} setResult={setResult} inspectionId={inspection.id} />
          </section>
          <div className="flex justify-between">
            <Button variant="outline" disabled={index === 0} onClick={() => setIndex((value) => value - 1)}>
              <ChevronLeft />
              {t("common.previous")}
            </Button>
            <Button
              disabled={index === steps.length - 1 || (step.required && !current)}
              onClick={() => setIndex((value) => value + 1)}
            >
              {t("common.next")}
              <ChevronRight />
            </Button>
          </div>
        </>
      ) : null}
      <section className="grid gap-3 border-t pt-4 sm:grid-cols-2">
        <h3 className="font-semibold sm:col-span-2">{t("execution.result")}</h3>
        <Field label={t("common.status")}>
          <select
            className="h-9 rounded-md border bg-background px-3 text-sm"
            value={statusAfter}
            onChange={(event) => setStatusAfter(event.target.value as OperationalStatus)}
          >
            {(["unknown", "ok", "attention", "critical", "maintenance_required", "disabled"] as const).map((status) => (
              <option key={status} value={status}>
                {t(`status.${status}`)}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t("execution.action")}>
          <Input value={action} onChange={(event) => setAction(event.target.value)} />
        </Field>
        <Field label={t("execution.finding")}>
          <Textarea value={finding} onChange={(event) => setFinding(event.target.value)} />
        </Field>
        <Field label={t("common.status")}>
          <select
            className="h-9 rounded-md border bg-background px-3 text-sm"
            value={findingSeverity}
            onChange={(event) => setFindingSeverity(event.target.value as OperationalStatus)}
          >
            {(["attention", "critical", "maintenance_required"] as const).map((status) => (
              <option key={status} value={status}>
                {t(`status.${status}`)}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t("common.notes")}>
          <Textarea value={notes} onChange={(event) => setNotes(event.target.value)} />
        </Field>
      </section>
      <div className="flex justify-end">
        <Button disabled={pending || !requiredComplete} onClick={submit}>
          <Check />
          {t("execution.completeInspection")}
        </Button>
      </div>
    </div>
  );
}

function StepInput({
  step,
  result,
  setResult,
  inspectionId,
}: {
  step: MaintenanceGuideStep;
  result?: InspectionStepResult;
  setResult: (value: unknown, unit?: string | null, photos?: string[]) => void;
  inspectionId: string;
}) {
  const { t } = useI18n();
  if (step.step_type === "information" || step.step_type === "warning")
    return (
      <Button variant="outline" onClick={() => setResult(true)}>
        <Check />
        {t("common.confirm")}
      </Button>
    );
  if (step.step_type === "confirmation" || step.step_type === "boolean")
    return (
      <select
        className="h-9 rounded-md border bg-background px-3 text-sm"
        value={result?.value === true ? "true" : result?.value === false ? "false" : ""}
        onChange={(event) => setResult(event.target.value === "true")}
      >
        <option value="">{t("common.select")}</option>
        <option value="true">{t("common.confirm")}</option>
        <option value="false">{t("common.cancel")}</option>
      </select>
    );
  if (step.step_type === "choice")
    return (
      <select
        className="h-9 rounded-md border bg-background px-3 text-sm"
        value={String(result?.value ?? "")}
        onChange={(event) => setResult(event.target.value)}
      >
        <option value="">{t("common.select")}</option>
        {step.options.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    );
  if (step.step_type === "numeric" || step.step_type === "measurement")
    return (
      <div className="flex max-w-sm items-center gap-2">
        <Input
          type="number"
          value={String(result?.value ?? "")}
          min={step.expected_value?.minimum ?? undefined}
          max={step.expected_value?.maximum ?? undefined}
          onChange={(event) => setResult(Number(event.target.value), step.expected_value?.unit ?? null)}
        />
        {step.expected_value?.unit ? <span className="text-sm">{step.expected_value.unit}</span> : null}
      </div>
    );
  if (step.step_type === "photo")
    return (
      <InspectionPhotoInput
        inspectionId={inspectionId}
        photoIds={result?.photo_ids ?? []}
        update={(ids) => setResult(true, null, ids)}
      />
    );
  return <Textarea value={String(result?.value ?? "")} onChange={(event) => setResult(event.target.value)} />;
}

function InspectionPhotoInput({
  inspectionId,
  photoIds,
  update,
}: {
  inspectionId: string;
  photoIds: string[];
  update: (ids: string[]) => void;
}) {
  const { t } = useI18n();
  const [pending, setPending] = useState(false);
  const [purpose, setPurpose] = useState<PhotoPurpose>("evidence");
  async function upload(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) return;
    setPending(true);
    let ref: string | undefined;
    try {
      const form = new FormData();
      form.set("kind", "inspection-photos");
      form.set("file", file);
      const uploadResponse = await fetch("/api/proexel/attachments", { method: "POST", body: form });
      const upload = (await uploadResponse.json()) as { ref?: string };
      if (!uploadResponse.ok || !upload.ref) throw new Error(t("photos.invalid"));
      ref = upload.ref;
      const response = await fetch("/api/proexel/photos", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          owner_type: "inspection",
          owner_id: inspectionId,
          purpose,
          blob_ref: ref,
          description: null,
        }),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string; resource_id?: string };
      if (!response.ok || !result.accepted || !result.resource_id)
        throw new Error(result.message ?? t("command.rejected"));
      update([...photoIds, result.resource_id]);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t("command.failed"));
    } finally {
      setPending(false);
      event.target.value = "";
    }
  }
  return (
    <div className="flex flex-wrap items-center gap-2">
      <select
        className="h-9 rounded-md border bg-background px-3 text-sm"
        value={purpose}
        onChange={(event) => setPurpose(event.target.value as PhotoPurpose)}
        aria-label={t("photos.purpose")}
      >
        {(["before", "during", "after", "defect", "evidence"] as const).map((value) => (
          <option key={value} value={value}>
            {t(`photos.${value}`)}
          </option>
        ))}
      </select>
      <Button asChild variant="outline" disabled={pending}>
        <label className="cursor-pointer">
          <Camera />
          {t("photos.add")}
          <input type="file" className="sr-only" accept="image/png,image/jpeg,image/webp" onChange={upload} />
        </label>
      </Button>
      {photoIds.length ? (
        <Badge className="ml-2" variant="outline">
          {photoIds.length}
        </Badge>
      ) : null}
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="grid gap-2">
      <Label>{label}</Label>
      {children}
    </div>
  );
}
