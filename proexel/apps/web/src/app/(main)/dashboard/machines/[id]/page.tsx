import Image from "next/image";
import { notFound } from "next/navigation";

import { History, Pencil } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listInspections, listItemCategories, listMachines } from "@/lib/proexel/service";
import type { ItemInspection, MachineItem } from "@/lib/proexel/types";

import { CommandDialog } from "../../_components/command-dialog";
import { OperationalStatusBadge } from "../../_components/operational-status-badge";
import { PageHeader } from "../../_components/page-header";
import { AssetPhotoManager } from "./asset-photo-manager";
import { MachineItemManager } from "./machine-item-manager";

export const dynamic = "force-dynamic";

export default async function MachineDetailPage({ params }: { readonly params: Promise<{ id: string }> }) {
  const { id } = await params;
  const [{ role }, machines, categories, inspections, { t, locale }] = await Promise.all([
    requirePermission("machine.read"),
    listMachines({ id, include_removed: true, page_size: 1 }),
    listItemCategories(),
    listInspections({ machine_id: id }),
    getI18n(),
  ]);
  const machine = machines.items[0];
  if (!machine) notFound();
  const canManage = can("machine_item.manage", role);
  const activeItems = machine.items
    .filter((item) => item.active)
    .toSorted((left, right) => left.position - right.position);
  const removedItems = machine.items
    .filter((item) => !item.active)
    .toSorted((left, right) => (right.removed_at_ms ?? 0) - (left.removed_at_ms ?? 0));
  return (
    <div>
      <PageHeader
        title={`${machine.code} · ${machine.name}`}
        description={t("machines.detailDescription")}
        action={
          can("machine.update", role) ? (
            <CommandDialog
              trigger={
                <Button variant="outline">
                  <Pencil />
                  {t("machines.edit")}
                </Button>
              }
              title={t("machines.edit")}
              description={t("machines.description")}
              endpoint="/api/proexel/machines"
              method="PATCH"
              fields={machineFields(machine, t)}
            />
          ) : null
        }
      />
      <div className="mb-4 grid gap-4 lg:grid-cols-[minmax(0,1fr)_340px]">
        <Card>
          <CardHeader>
            <div className="flex flex-wrap items-center gap-2">
              <CardTitle className="text-lg">{machine.name}</CardTitle>
              <OperationalStatusBadge status={machine.status} />
            </div>
            <CardDescription>{machine.description || "-"}</CardDescription>
          </CardHeader>
          <CardContent className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            <Datum label={t("common.code")} value={machine.code} />
            <Datum label={t("common.zone")} value={machine.zone} />
            <Datum label={t("common.location")} value={machine.location} />
            <Datum label={t("common.manufacturer")} value={machine.manufacturer} />
            <Datum label={t("common.model")} value={machine.model} />
            <Datum label={t("common.serialNumber")} value={machine.serial_number} />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle className="text-base">{t("common.photo")}</CardTitle>
          </CardHeader>
          <CardContent>
            <AssetPhotoManager
              ownerType="machine"
              ownerId={machine.id}
              photos={machine.photos}
              kind="machine-photos"
              canEdit={can("photo.manage_reference", role)}
              defaultPurpose="general"
            />
          </CardContent>
        </Card>
      </div>
      <Card>
        <CardHeader>
          <CardTitle>{t("common.components")}</CardTitle>
          <CardDescription>{t("machines.itemCount", { count: activeItems.length })}</CardDescription>
        </CardHeader>
        <CardContent>
          <MachineItemManager
            machine={{ ...machine, items: activeItems }}
            categories={categories.items}
            canManage={canManage}
          />
          <div className="mt-4 space-y-3">
            {activeItems.map((item) => (
              <Component
                key={item.id}
                item={item}
                inspections={inspections.items.filter((inspection) => inspection.machine_item_id === item.id)}
                canEditPhotos={can("photo.manage_reference", role)}
                locale={locale}
                t={t}
              />
            ))}
          </div>
        </CardContent>
      </Card>
      {removedItems.length ? (
        <section className="mt-6">
          <h2 className="mb-3 font-semibold text-lg">{t("machines.removedComponents")}</h2>
          <div className="space-y-3">
            {removedItems.map((item) => (
              <Component
                key={item.id}
                item={item}
                inspections={inspections.items.filter((inspection) => inspection.machine_item_id === item.id)}
                canEditPhotos={false}
                locale={locale}
                t={t}
              />
            ))}
          </div>
        </section>
      ) : null}
    </div>
  );
}

function Component({
  item,
  inspections,
  canEditPhotos,
  locale,
  t,
}: {
  item: MachineItem;
  inspections: ItemInspection[];
  canEditPhotos: boolean;
  locale: "pt" | "en" | "es" | "fr";
  t: Awaited<ReturnType<typeof getI18n>>["t"];
}) {
  const category = item.category;
  return (
    <section className="rounded-md border p-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 className="font-semibold">
            {item.code} · {item.name}
          </h2>
          <p className="text-muted-foreground text-sm">{category?.name ?? item.category_id}</p>
        </div>
        <div className="flex gap-2">
          <Badge variant="secondary">{item.complexity_level}/5</Badge>
          <OperationalStatusBadge status={item.status} />
        </div>
      </div>
      <Separator className="my-4" />
      <Tabs defaultValue="details">
        <TabsList>
          <TabsTrigger value="details">{t("common.details")}</TabsTrigger>
          <TabsTrigger value="photos">{t("common.photo")}</TabsTrigger>
          <TabsTrigger value="history">
            <History />
            {t("common.history")}
          </TabsTrigger>
        </TabsList>
        <TabsContent value="details" className="grid gap-3 md:grid-cols-3">
          <Datum label={t("common.location")} value={item.location_description} />
          <Datum label={t("common.manufacturer")} value={item.installed_component?.manufacturer} />
          <Datum label={t("common.model")} value={item.installed_component?.model} />
          <Datum label={t("common.partNumber")} value={item.installed_component?.part_number} />
          <Datum label={t("common.serialNumber")} value={item.installed_component?.serial_number} />
          <Datum label={t("machines.replacementSpec")} value={item.replacement_specification.part_number} />
          {Object.entries(item.custom_field_values).map(([key, value]) => (
            <Datum
              key={key}
              label={category?.custom_field_definitions.find((field) => field.key === key)?.label ?? key}
              value={String(value)}
            />
          ))}
        </TabsContent>
        <TabsContent value="photos">
          <AssetPhotoManager
            ownerType="machine_item"
            ownerId={item.id}
            photos={item.photos}
            kind="item-photos"
            canEdit={canEditPhotos}
          />
        </TabsContent>
        <TabsContent value="history">
          <Timeline item={item} inspections={inspections} canEditPhotos={canEditPhotos} locale={locale} t={t} />
        </TabsContent>
      </Tabs>
    </section>
  );
}

function Timeline({
  item,
  inspections,
  canEditPhotos,
  locale,
  t,
}: {
  item: MachineItem;
  inspections: ItemInspection[];
  canEditPhotos: boolean;
  locale: string;
  t: Awaited<ReturnType<typeof getI18n>>["t"];
}) {
  const events = [
    ...inspections.map((inspection) => ({
      id: inspection.id,
      at: inspection.completed_at_ms ?? inspection.started_at_ms,
      title: inspection.operator_name,
      detail:
        inspection.findings.map((finding) => finding.description).join(" · ") ||
        inspection.maintenance_action ||
        inspection.notes ||
        inspection.status,
      photos: inspection.photos,
      replacementId: null,
    })),
    ...item.replacement_history.map((replacement) => ({
      id: replacement.id,
      at: replacement.replaced_at_ms,
      title: replacement.replaced_by,
      detail: `${replacement.previous?.serial_number ?? "-"} -> ${replacement.current.serial_number ?? "-"} - ${replacement.reason}`,
      photos: replacement.photos,
      replacementId: replacement.id,
    })),
  ].toSorted((left, right) => right.at - left.at);
  return events.length ? (
    <div className="divide-y">
      {events.map((event) => (
        <div key={event.id} className="grid gap-2 py-3 sm:grid-cols-[160px_180px_1fr]">
          <time className="text-muted-foreground text-sm">
            {new Intl.DateTimeFormat(locale, { dateStyle: "medium", timeStyle: "short" }).format(new Date(event.at))}
          </time>
          <strong className="text-sm">{event.title}</strong>
          <div>
            <span className="text-sm">{event.detail}</span>
            {event.replacementId ? (
              <div className="mt-2">
                <AssetPhotoManager
                  ownerType="replacement"
                  ownerId={event.replacementId}
                  photos={event.photos}
                  kind="replacement-photos"
                  canEdit={canEditPhotos}
                  defaultPurpose="after"
                />
              </div>
            ) : event.photos.length ? (
              <div className="mt-2 grid grid-cols-2 gap-2 sm:grid-cols-4">
                {event.photos.map((photo) => (
                  <Image
                    key={photo.id}
                    unoptimized
                    src={`/api/proexel/attachments?ref=${encodeURIComponent(photo.blob_ref)}`}
                    alt={photo.description ?? t("common.photo")}
                    width={320}
                    height={220}
                    className="aspect-[3/2] w-full rounded-md border object-cover"
                  />
                ))}
              </div>
            ) : null}
          </div>
        </div>
      ))}
    </div>
  ) : (
    <p className="py-6 text-center text-muted-foreground text-sm">{t("machines.noHistory")}</p>
  );
}

function Datum({ label, value }: { label: string; value?: string | null }) {
  return (
    <div>
      <div className="text-muted-foreground text-xs">{label}</div>
      <div className="mt-1 text-sm">{value ?? "-"}</div>
    </div>
  );
}

function machineFields(
  machine: NonNullable<Awaited<ReturnType<typeof listMachines>>["items"][number]>,
  t: Awaited<ReturnType<typeof getI18n>>["t"],
) {
  return [
    { name: "id", label: "ID", type: "hidden" as const, defaultValue: machine.id },
    { name: "code", label: t("common.code"), required: true, defaultValue: machine.code },
    { name: "name", label: t("common.name"), required: true, defaultValue: machine.name },
    { name: "zone", label: t("common.zone"), required: true, defaultValue: machine.zone },
    { name: "location", label: t("common.location"), defaultValue: machine.location ?? "" },
    { name: "manufacturer", label: t("common.manufacturer"), defaultValue: machine.manufacturer ?? "" },
    { name: "model", label: t("common.model"), defaultValue: machine.model ?? "" },
    { name: "serial_number", label: t("common.serialNumber"), defaultValue: machine.serial_number ?? "" },
    {
      name: "description",
      label: t("common.description"),
      type: "textarea" as const,
      defaultValue: machine.description ?? "",
    },
    { name: "active", label: t("common.active"), type: "checkbox" as const, defaultValue: machine.active },
  ];
}
