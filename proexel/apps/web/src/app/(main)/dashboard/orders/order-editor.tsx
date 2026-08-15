"use client";

import { useMemo, useState } from "react";

import { useRouter } from "next/navigation";

import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
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
import type { Machine, OperatorSummary } from "@/lib/proexel/types";

export function OrderEditor({
  machines,
  operators,
  trigger,
}: {
  machines: Machine[];
  operators: OperatorSummary[];
  trigger: React.ReactNode;
}) {
  const { t } = useI18n();
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [machineId, setMachineId] = useState(machines[0]?.id ?? "");
  const [allItems, setAllItems] = useState(true);
  const [itemIds, setItemIds] = useState<string[]>([]);
  const machine = useMemo(() => machines.find((entry) => entry.id === machineId), [machines, machineId]);
  function selectMachine(id: string) {
    setMachineId(id);
    setItemIds([]);
    setAllItems(true);
  }
  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    setPending(true);
    try {
      const response = await fetch("/api/proexel/orders", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          machine_id: machineId,
          all_items: allItems,
          item_ids: allItems ? [] : itemIds,
          description: form.get("description"),
          priority: form.get("priority"),
          scheduled_for: form.get("scheduled_for") || null,
          assigned_operator_id: form.get("assigned_operator_id") || null,
        }),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string };
      if (!response.ok || !result.accepted) throw new Error(result.message ?? t("command.rejected"));
      toast.success(t("command.success"));
      setOpen(false);
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t("command.failed"));
    } finally {
      setPending(false);
    }
  }
  const maxLevel =
    (allItems ? machine?.items : machine?.items.filter((item) => itemIds.includes(item.id)))?.reduce(
      (max, item) => Math.max(max, item.complexity_level),
      1,
    ) ?? 1;
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <span className="contents">{trigger}</span>
      </DialogTrigger>
      <DialogContent className="max-h-[92vh] overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t("orders.create")}</DialogTitle>
          <DialogDescription>{t("orders.createDescription")}</DialogDescription>
        </DialogHeader>
        <form className="space-y-4" onSubmit={submit}>
          <Field label={t("orders.machine")}>
            <select
              className="h-9 rounded-md border bg-background px-3 text-sm"
              value={machineId}
              onChange={(event) => selectMachine(event.target.value)}
              required
            >
              {machines.map((entry) => (
                <option key={entry.id} value={entry.id}>
                  {entry.code} · {entry.name}
                </option>
              ))}
            </select>
          </Field>
          <div className="space-y-2">
            <Label>{t("orders.selectItems")}</Label>
            <div className="flex items-center gap-2 rounded-md border p-3 text-sm">
              <Checkbox id="all-items" checked={allItems} onCheckedChange={(value) => setAllItems(value === true)} />
              <Label htmlFor="all-items">{t("orders.allItems")}</Label>
            </div>
            {!allItems ? (
              <div className="max-h-52 divide-y overflow-y-auto rounded-md border">
                {machine?.items.map((item) => (
                  <div key={item.id} className="flex items-center gap-3 p-3 text-sm">
                    <Checkbox
                      id={`order-item-${item.id}`}
                      checked={itemIds.includes(item.id)}
                      onCheckedChange={(checked) =>
                        setItemIds((current) =>
                          checked ? [...current, item.id] : current.filter((id) => id !== item.id),
                        )
                      }
                    />
                    <Label htmlFor={`order-item-${item.id}`} className="flex-1">
                      {item.code} · {item.name}
                    </Label>
                    <span>{item.complexity_level}/5</span>
                  </div>
                ))}
              </div>
            ) : null}
            <p className="text-muted-foreground text-xs">{t("orders.maxLevel", { level: maxLevel })}</p>
          </div>
          <Field label={t("common.description")}>
            <Textarea name="description" required />
          </Field>
          <div className="grid gap-3 sm:grid-cols-3">
            <Field label={t("common.priority")}>
              <select
                name="priority"
                defaultValue="normal"
                className="h-9 rounded-md border bg-background px-3 text-sm"
              >
                {(["low", "normal", "high", "urgent"] as const).map((value) => (
                  <option key={value} value={value}>
                    {t(`orders.${value}`)}
                  </option>
                ))}
              </select>
            </Field>
            <Field label={t("orders.scheduledDate")}>
              <Input name="scheduled_for" type="date" />
            </Field>
            <Field label={t("orders.operator")}>
              <select name="assigned_operator_id" className="h-9 rounded-md border bg-background px-3 text-sm">
                <option value="">{t("common.select")}</option>
                {operators
                  .filter((operator) => operator.maximum_repair_level >= maxLevel)
                  .map((operator) => (
                    <option key={operator.id} value={operator.id}>
                      {operator.name} ({operator.maximum_repair_level}/5)
                    </option>
                  ))}
              </select>
            </Field>
          </div>
          <DialogFooter>
            <Button type="submit" disabled={pending || !machineId || (!allItems && itemIds.length === 0)}>
              {pending ? t("common.saving") : t("common.confirm")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
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
