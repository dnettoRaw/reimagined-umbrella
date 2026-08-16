"use client";

import { useMemo, useState } from "react";

import Link from "next/link";
import { useRouter } from "next/navigation";

import { Eye, Pencil, Search, Wrench } from "lucide-react";
import { toast } from "sonner";

import { Badge } from "@/components/ui/badge";
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
import { Switch } from "@/components/ui/switch";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { useI18n } from "@/lib/i18n/provider";
import type { ItemCategory, Machine, MachineItem, OperationalStatus } from "@/lib/proexel/types";

import { OperationalStatusBadge } from "../_components/operational-status-badge";
import { ProexelEmptyState } from "../_components/proexel-empty-state";
import { MaintenanceGuideEditor } from "./maintenance-guide-editor";

type ComponentEntry = { item: MachineItem; machine: Pick<Machine, "id" | "code" | "name" | "zone" | "location"> };

export function ComponentRegistry({
  entries,
  categories,
  canManage,
}: {
  entries: ComponentEntry[];
  categories: ItemCategory[];
  canManage: boolean;
}) {
  const { t } = useI18n();
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<ComponentEntry | null>(null);
  const filtered = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    return entries.filter(
      ({ item, machine }) =>
        !normalized ||
        `${item.code} ${item.name} ${machine.code} ${machine.name} ${machine.zone}`.toLowerCase().includes(normalized),
    );
  }, [entries, query]);

  return (
    <>
      <div className="mb-4 flex items-center gap-2">
        <Search className="size-4 text-muted-foreground" />
        <Input
          className="max-w-xl"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder={t("components.search")}
        />
        <Badge variant="secondary">{filtered.length}</Badge>
      </div>
      {filtered.length ? (
        <div className="overflow-x-auto rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("common.component")}</TableHead>
                <TableHead>{t("common.machine")}</TableHead>
                <TableHead>{t("common.category")}</TableHead>
                <TableHead>{t("components.machineLocation")}</TableHead>
                <TableHead>{t("components.tutorial")}</TableHead>
                <TableHead>{t("common.status")}</TableHead>
                <TableHead className="text-right">{t("common.actions")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filtered.map((entry) => (
                <TableRow key={entry.item.id}>
                  <TableCell>
                    <p className="font-medium">{entry.item.code}</p>
                    <p className="text-muted-foreground text-xs">{entry.item.name}</p>
                  </TableCell>
                  <TableCell>
                    {entry.machine.code} · {entry.machine.name}
                  </TableCell>
                  <TableCell>{entry.item.category?.name ?? entry.item.category_id}</TableCell>
                  <TableCell>
                    <p>{entry.machine.location || "-"}</p>
                    <p className="text-muted-foreground text-xs">{entry.machine.zone}</p>
                  </TableCell>
                  <TableCell>
                    {entry.item.maintenance_guide_override ? (
                      <Badge>{t("components.specificGuide")}</Badge>
                    ) : (
                      <Badge variant="outline">{t("components.inheritedGuide")}</Badge>
                    )}
                  </TableCell>
                  <TableCell>
                    <OperationalStatusBadge status={entry.item.status} />
                  </TableCell>
                  <TableCell className="text-right">
                    <Button asChild size="icon-sm" variant="ghost" title={t("common.details")}>
                      <Link href={`/dashboard/machines/${encodeURIComponent(entry.machine.id)}`}>
                        <Eye />
                        <span className="sr-only">{t("common.details")}</span>
                      </Link>
                    </Button>
                    {canManage ? (
                      <Button
                        size="icon-sm"
                        variant="ghost"
                        title={t("machines.editItem")}
                        onClick={() => setSelected(entry)}
                      >
                        <Pencil />
                        <span className="sr-only">{t("machines.editItem")}</span>
                      </Button>
                    ) : null}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      ) : (
        <ProexelEmptyState icon={Wrench} title={t("components.none")} description={t("components.noneDescription")} />
      )}
      {selected ? (
        <ComponentEditor
          key={selected.item.id}
          entry={selected}
          categories={categories}
          close={() => setSelected(null)}
        />
      ) : null}
    </>
  );
}

function ComponentEditor({
  entry,
  categories,
  close,
}: {
  entry: ComponentEntry;
  categories: ItemCategory[];
  close: () => void;
}) {
  const { t } = useI18n();
  const router = useRouter();
  const [pending, setPending] = useState(false);
  const [draft, setDraft] = useState(() => structuredClone(entry.item));
  const category = categories.find((item) => item.id === draft.category_id) ?? entry.item.category;
  const inherited = !draft.maintenance_guide_override;

  async function save(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPending(true);
    try {
      const response = await fetch("/api/proexel/machine-items", {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          id: draft.id,
          category_id: draft.category_id,
          name: draft.name,
          code: draft.code,
          complexity_level: draft.complexity_level,
          status: draft.status,
          custom_field_values: draft.custom_field_values,
          maintenance_guide_override: draft.maintenance_guide_override,
          replacement_specification: draft.replacement_specification,
          notes: draft.notes || null,
        }),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string };
      if (!response.ok || !result.accepted) throw new Error(result.message ?? t("command.rejected"));
      toast.success(t("command.success"));
      close();
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t("command.failed"));
    } finally {
      setPending(false);
    }
  }

  return (
    <Dialog open onOpenChange={(open) => !open && close()}>
      <DialogContent className="max-h-[94vh] overflow-y-auto sm:max-w-4xl">
        <DialogHeader>
          <DialogTitle>{t("components.editTitle", { code: entry.item.code })}</DialogTitle>
          <DialogDescription>
            {entry.machine.code} · {entry.machine.name}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={save}>
          <Tabs defaultValue="details">
            <TabsList>
              <TabsTrigger value="details">{t("common.details")}</TabsTrigger>
              <TabsTrigger value="tutorial">{t("components.tutorial")}</TabsTrigger>
            </TabsList>
            <TabsContent value="details" className="grid gap-3 pt-3 sm:grid-cols-2">
              <Field label={t("common.code")}>
                <Input
                  value={draft.code}
                  required
                  onChange={(event) => setDraft({ ...draft, code: event.target.value })}
                />
              </Field>
              <Field label={t("common.name")}>
                <Input
                  value={draft.name}
                  required
                  onChange={(event) => setDraft({ ...draft, name: event.target.value })}
                />
              </Field>
              <Field label={t("common.category")}>
                <select
                  className="h-9 w-full rounded-md border bg-background px-3 text-sm"
                  value={draft.category_id}
                  onChange={(event) => setDraft({ ...draft, category_id: event.target.value })}
                >
                  {categories
                    .filter((item) => item.active || item.id === draft.category_id)
                    .map((item) => (
                      <option key={item.id} value={item.id}>
                        {item.name}
                      </option>
                    ))}
                </select>
              </Field>
              <Field label={t("common.complexity")}>
                <select
                  className="h-9 w-full rounded-md border bg-background px-3 text-sm"
                  value={draft.complexity_level}
                  onChange={(event) =>
                    setDraft({ ...draft, complexity_level: Number(event.target.value) as 1 | 2 | 3 | 4 | 5 })
                  }
                >
                  {[1, 2, 3, 4, 5].map((level) => (
                    <option key={level} value={level}>
                      {level}
                    </option>
                  ))}
                </select>
              </Field>
              <Field label={t("common.status")}>
                <select
                  className="h-9 w-full rounded-md border bg-background px-3 text-sm"
                  value={draft.status}
                  onChange={(event) => setDraft({ ...draft, status: event.target.value as OperationalStatus })}
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
              <Field label={t("common.notes")} wide>
                <Textarea
                  value={draft.notes ?? ""}
                  onChange={(event) => setDraft({ ...draft, notes: event.target.value })}
                />
              </Field>
            </TabsContent>
            <TabsContent value="tutorial" className="space-y-4 pt-3">
              <div className="flex items-center justify-between gap-4 rounded-md border p-3">
                <div>
                  <Label htmlFor="inherit-guide">{t("components.useCategoryGuide")}</Label>
                  <p className="text-muted-foreground text-sm">{category?.name ?? draft.category_id}</p>
                </div>
                <Switch
                  id="inherit-guide"
                  checked={inherited}
                  onCheckedChange={(checked) =>
                    setDraft({
                      ...draft,
                      maintenance_guide_override: checked
                        ? null
                        : structuredClone(category?.maintenance_guide ?? { version: 1, steps: [] }),
                    })
                  }
                />
              </div>
              {!inherited && draft.maintenance_guide_override ? (
                <MaintenanceGuideEditor
                  steps={draft.maintenance_guide_override.steps}
                  onChange={(steps) =>
                    setDraft({
                      ...draft,
                      maintenance_guide_override: {
                        version: draft.maintenance_guide_override?.version ?? 1,
                        steps,
                      },
                    })
                  }
                />
              ) : null}
            </TabsContent>
          </Tabs>
          <DialogFooter className="mt-5 border-t pt-4">
            <Button type="button" variant="outline" onClick={close}>
              {t("common.cancel")}
            </Button>
            <Button type="submit" disabled={pending}>
              {pending ? t("common.saving") : t("common.save")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
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
