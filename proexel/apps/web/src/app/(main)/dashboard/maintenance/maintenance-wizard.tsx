"use client";

import { forwardRef, useImperativeHandle, useRef, useState } from "react";

import { useRouter } from "next/navigation";

import { Check, ChevronLeft, ChevronRight, Eraser, Loader2, Plus } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { useI18n } from "@/lib/i18n/provider";
import type { StockItem, Valve } from "@/lib/proexel/types";

type Draft = {
  valve_id: string;
  performed_at: string;
  maintenance_type: "preventive" | "corrective";
  service: string;
  notes: string;
  kit_changed: boolean;
};

const INITIAL: Draft = {
  valve_id: "",
  performed_at: new Date().toISOString().slice(0, 10),
  maintenance_type: "preventive",
  service: "",
  notes: "",
  kit_changed: false,
};

export function MaintenanceWizard({
  valves,
  stock,
  technician,
}: {
  valves: Valve[];
  stock: StockItem[];
  technician: string;
}) {
  const { t } = useI18n();
  const router = useRouter();
  const signature = useRef<SignaturePadHandle>(null);
  const [open, setOpen] = useState(false);
  const [step, setStep] = useState(0);
  const [draft, setDraft] = useState<Draft>(INITIAL);
  const [signed, setSigned] = useState(false);
  const [pending, setPending] = useState(false);
  const selectedValve = valves.find((valve) => valve.id === draft.valve_id);
  const stockItem = stock.find((item) => item.reference === selectedValve?.kit_reference);
  const steps = [
    t("maintenance.stepValve"),
    t("maintenance.stepWork"),
    t("maintenance.stepSignature"),
    t("maintenance.stepReview"),
  ];

  function update<K extends keyof Draft>(key: K, value: Draft[K]) {
    setDraft((current) => ({ ...current, [key]: value }));
  }

  function next() {
    if (step === 0 && (!draft.valve_id || !draft.performed_at)) return;
    if (step === 1 && !draft.service.trim()) return;
    if (step === 2 && !signed) {
      toast.error(t("maintenance.signatureRequired"));
      return;
    }
    setStep((current) => Math.min(3, current + 1));
  }

  async function submit() {
    const blob = await signature.current?.toBlob();
    if (!blob) {
      toast.error(t("maintenance.signatureRequired"));
      setStep(2);
      return;
    }
    setPending(true);
    let signatureRef: string | undefined;
    try {
      const form = new FormData();
      form.set("kind", "signatures");
      form.set("file", new File([blob], "signature.png", { type: "image/png" }));
      const uploaded = await fetch("/api/proexel/attachments", { method: "POST", body: form });
      const uploadBody = (await uploaded.json()) as { ref?: string };
      if (!uploaded.ok || !uploadBody.ref) throw new Error(t("maintenance.signatureRequired"));
      signatureRef = uploadBody.ref;
      const response = await fetch("/api/proexel/maintenance", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ ...draft, technician, signature_ref: signatureRef }),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string };
      if (!response.ok || !result.accepted) throw new Error(result.message ?? t("command.rejected"));
      toast.success(t("maintenance.completeSuccess"));
      setOpen(false);
      setStep(0);
      setDraft(INITIAL);
      setSigned(false);
      router.refresh();
    } catch (error) {
      if (signatureRef) {
        await fetch("/api/proexel/attachments", {
          method: "DELETE",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ ref: signatureRef }),
        });
      }
      toast.error(error instanceof Error ? error.message : t("command.failed"));
    } finally {
      setPending(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button disabled={valves.length === 0}>
          <Plus />
          {t("maintenance.start")}
        </Button>
      </DialogTrigger>
      <DialogContent className="max-h-[92vh] overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t("maintenance.register")}</DialogTitle>
          <DialogDescription>{t("maintenance.wizardDescription")}</DialogDescription>
        </DialogHeader>
        <nav className="grid grid-cols-4 gap-1" aria-label={t("maintenance.register")}>
          {steps.map((label, index) => (
            <div key={label} className="min-w-0 text-center">
              <div
                className={`mx-auto flex size-7 items-center justify-center rounded-full border text-xs ${index <= step ? "border-primary bg-primary text-primary-foreground" : "text-muted-foreground"}`}
              >
                {index < step ? <Check className="size-4" /> : index + 1}
              </div>
              <div className="mt-1 truncate text-xs" title={label}>
                {label}
              </div>
            </div>
          ))}
        </nav>

        <div className="min-h-80 py-2">
          {step === 0 ? (
            <div className="grid gap-4 sm:grid-cols-2">
              <Field label={t("maintenance.valve")}>
                <select
                  className="h-9 rounded-lg border bg-background px-3 text-sm"
                  value={draft.valve_id}
                  onChange={(event) => update("valve_id", event.target.value)}
                  required
                >
                  <option value="">{t("common.select")}</option>
                  {valves.map((valve) => (
                    <option key={valve.id} value={valve.id}>
                      {valve.tag} · {valve.zone}
                    </option>
                  ))}
                </select>
              </Field>
              <Field label={t("common.date")}>
                <Input
                  type="date"
                  value={draft.performed_at}
                  onChange={(event) => update("performed_at", event.target.value)}
                  required
                />
              </Field>
              <Field label={t("common.type")}>
                <select
                  className="h-9 rounded-lg border bg-background px-3 text-sm"
                  value={draft.maintenance_type}
                  onChange={(event) => update("maintenance_type", event.target.value as Draft["maintenance_type"])}
                >
                  <option value="preventive">{t("maintenance.preventive")}</option>
                  <option value="corrective">{t("maintenance.corrective")}</option>
                </select>
              </Field>
              <Field label={t("common.technician")}>
                <Input value={technician} readOnly />
              </Field>
            </div>
          ) : null}

          {step === 1 ? (
            <div className="space-y-4">
              <Field label={t("maintenance.service")}>
                <Textarea value={draft.service} onChange={(event) => update("service", event.target.value)} required />
              </Field>
              <Field label={t("common.notes")}>
                <Textarea value={draft.notes} onChange={(event) => update("notes", event.target.value)} />
              </Field>
              <label className="flex items-center gap-2 font-medium text-sm">
                <input
                  type="checkbox"
                  checked={draft.kit_changed}
                  onChange={(event) => update("kit_changed", event.target.checked)}
                />
                {t("maintenance.kitChanged")}
              </label>
              {draft.kit_changed ? (
                <div className="rounded-md border p-3 text-sm">
                  <div className="mb-1 font-medium">{t("maintenance.stockImpact")}</div>
                  {!selectedValve?.kit_reference
                    ? t("maintenance.noKitReference")
                    : stockItem && stockItem.quantity > 0
                      ? t("maintenance.stockAvailable", {
                          reference: selectedValve.kit_reference,
                          quantity: stockItem.quantity,
                        })
                      : t("maintenance.stockPending", { reference: selectedValve.kit_reference })}
                </div>
              ) : null}
            </div>
          ) : null}

          <div className={step === 2 || step === 3 ? "block" : "hidden"}>
            {step === 2 ? <p className="mb-3 text-muted-foreground text-sm">{t("maintenance.drawSignature")}</p> : null}
            <SignaturePad ref={signature} onSignedChange={setSigned} hidden={step !== 2} />
          </div>

          {step === 3 ? (
            <div className="space-y-4">
              <h3 className="font-semibold">{t("maintenance.reviewTitle")}</h3>
              <div className="grid gap-3 rounded-md border p-4 sm:grid-cols-2">
                <Review
                  label={t("maintenance.valve")}
                  value={selectedValve ? `${selectedValve.tag} · ${selectedValve.zone}` : "-"}
                />
                <Review label={t("common.date")} value={draft.performed_at} />
                <Review
                  label={t("common.type")}
                  value={
                    draft.maintenance_type === "preventive" ? t("maintenance.preventive") : t("maintenance.corrective")
                  }
                />
                <Review label={t("common.technician")} value={technician} />
                <Review label={t("maintenance.service")} value={draft.service} wide />
                <Review label={t("common.notes")} value={draft.notes || "-"} wide />
                <Review
                  label={t("maintenance.kitChanged")}
                  value={draft.kit_changed ? t("common.confirm") : t("maintenance.noChange")}
                />
                <Review
                  label={t("maintenance.stepSignature")}
                  value={signed ? t("common.confirm") : t("maintenance.signatureRequired")}
                />
              </div>
            </div>
          ) : null}
        </div>

        <DialogFooter className="flex-row justify-between sm:justify-between">
          <Button
            type="button"
            variant="outline"
            disabled={step === 0 || pending}
            onClick={() => setStep((current) => current - 1)}
          >
            <ChevronLeft />
            {t("common.previous")}
          </Button>
          {step < 3 ? (
            <Button type="button" disabled={pending} onClick={next}>
              {t("common.next")}
              <ChevronRight />
            </Button>
          ) : (
            <Button type="button" disabled={pending} onClick={submit}>
              {pending ? <Loader2 className="animate-spin" /> : <Check />}
              {t("common.confirm")}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

type SignaturePadHandle = { toBlob: () => Promise<Blob | null> };

const SignaturePad = forwardRef<SignaturePadHandle, { onSignedChange: (signed: boolean) => void; hidden: boolean }>(
  function SignaturePad({ onSignedChange, hidden }, ref) {
    const { t } = useI18n();
    const canvas = useRef<HTMLCanvasElement>(null);
    const drawing = useRef(false);
    useImperativeHandle(ref, () => ({
      toBlob: () =>
        new Promise((resolve) => {
          if (!canvas.current) {
            resolve(null);
            return;
          }
          canvas.current.toBlob(resolve, "image/png");
        }),
    }));
    function point(event: React.PointerEvent<HTMLCanvasElement>) {
      const element = canvas.current;
      if (!element) return { x: 0, y: 0 };
      const rect = element.getBoundingClientRect();
      return {
        x: (event.clientX - rect.left) * (element.width / rect.width),
        y: (event.clientY - rect.top) * (element.height / rect.height),
      };
    }
    function start(event: React.PointerEvent<HTMLCanvasElement>) {
      const context = canvas.current?.getContext("2d");
      if (!context || !canvas.current) return;
      drawing.current = true;
      canvas.current.setPointerCapture(event.pointerId);
      const current = point(event);
      context.beginPath();
      context.moveTo(current.x, current.y);
    }
    function move(event: React.PointerEvent<HTMLCanvasElement>) {
      if (!drawing.current) return;
      const context = canvas.current?.getContext("2d");
      if (!context) return;
      const current = point(event);
      context.lineWidth = 3;
      context.lineCap = "round";
      context.strokeStyle = "#111827";
      context.lineTo(current.x, current.y);
      context.stroke();
      onSignedChange(true);
    }
    function clear() {
      const element = canvas.current;
      element?.getContext("2d")?.clearRect(0, 0, element.width, element.height);
      onSignedChange(false);
    }
    return (
      <div className={hidden ? "sr-only" : "space-y-3"}>
        <canvas
          ref={canvas}
          width={900}
          height={280}
          className="h-56 w-full touch-none rounded-md border bg-white"
          onPointerDown={start}
          onPointerMove={move}
          onPointerUp={() => {
            drawing.current = false;
          }}
          onPointerCancel={() => {
            drawing.current = false;
          }}
        />
        <Button type="button" size="sm" variant="outline" onClick={clear}>
          <Eraser />
          {t("maintenance.clearSignature")}
        </Button>
      </div>
    );
  },
);

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="grid gap-2">
      <Label>{label}</Label>
      {children}
    </div>
  );
}

function Review({ label, value, wide = false }: { label: string; value: string; wide?: boolean }) {
  return (
    <div className={wide ? "sm:col-span-2" : undefined}>
      <div className="text-muted-foreground text-xs">{label}</div>
      <div className="mt-1 whitespace-pre-wrap text-sm">{value}</div>
    </div>
  );
}
