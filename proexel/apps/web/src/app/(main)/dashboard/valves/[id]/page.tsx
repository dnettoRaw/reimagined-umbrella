import Link from "next/link";
import { notFound } from "next/navigation";

import { ArrowLeft, Camera, Pencil, Wrench } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { INTL_LOCALES } from "@/lib/i18n/config";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listMaintenance, listValves } from "@/lib/proexel/service";

import { CommandDialog, type CommandField } from "../../_components/command-dialog";
import { PageHeader } from "../../_components/page-header";
import { PhotoManager } from "./photo-manager";

export const dynamic = "force-dynamic";

export default async function ValveDetailPage({ params }: { readonly params: Promise<{ id: string }> }) {
  const { id } = await params;
  const [{ role }, valves, maintenance, { t, locale }] = await Promise.all([
    requirePermission("valve.read"),
    listValves({ id, page_size: 1 }),
    listMaintenance(),
    getI18n(),
  ]);
  const valve = valves.items[0];
  if (!valve) notFound();
  const timeline = maintenance.items
    .filter((record) => record.valve_id === valve.id)
    .toSorted((left, right) => right.performed_at.localeCompare(left.performed_at));

  return (
    <div>
      <PageHeader
        title={valve.tag}
        description={t("valves.detailDescription")}
        action={
          <div className="flex gap-2">
            <Button asChild variant="outline">
              <Link href="/dashboard/valves">
                <ArrowLeft />
                {t("common.previous")}
              </Link>
            </Button>
            {can("valve.update_technical_fields", role) ? (
              <CommandDialog
                trigger={
                  <Button>
                    <Pencil />
                    {t("common.edit")}
                  </Button>
                }
                title={t("valves.edit")}
                description={t("valves.editDescription")}
                endpoint="/api/proexel/valves"
                method="PATCH"
                fields={editFields(valve, t)}
              />
            ) : null}
          </div>
        }
      />

      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>{t("valves.technicalData")}</CardTitle>
            <CardDescription>{valve.zone}</CardDescription>
          </CardHeader>
          <CardContent className="grid gap-x-6 gap-y-4 sm:grid-cols-2">
            <Detail label="TAG" value={valve.tag} />
            <Detail label={t("common.health")} value={<Health health={valve.health} t={t} />} />
            <Detail label={t("common.zone")} value={valve.zone} />
            <Detail label={t("common.manufacturer")} value={valve.manufacturer} />
            <Detail label={t("valves.serial")} value={valve.serial} />
            <Detail label={t("valves.valveType")} value={valve.valve_type} />
            <Detail label="DN" value={valve.dn} />
            <Detail label={t("valves.seat")} value={valve.seat} />
            <Detail label={t("valves.actuator")} value={valve.actuator} />
            <Detail label={t("valves.kitReference")} value={valve.kit_reference} />
            <Detail label={t("valves.manufactured")} value={formatDate(valve.manufactured_at, locale)} />
            <Detail
              label={t("valves.lastMaintenance")}
              value={formatDate(valve.last_maintenance_at, locale) ?? t("common.never")}
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Camera className="size-5" />
              {t("valves.photos")}
            </CardTitle>
            <CardDescription>{valve.photos.length ? `${valve.photos.length}` : t("valves.noPhotos")}</CardDescription>
          </CardHeader>
          <CardContent>
            <PhotoManager
              valveId={valve.id}
              valveTag={valve.tag}
              photos={valve.photos}
              editable={can("valve.update_photo", role)}
            />
          </CardContent>
        </Card>
      </div>

      <Card className="mt-4">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Wrench className="size-5" />
            {t("valves.timeline")}
          </CardTitle>
          <CardDescription>{t("maintenance.records", { count: timeline.length })}</CardDescription>
        </CardHeader>
        <CardContent>
          {timeline.length ? (
            <div className="divide-y">
              {timeline.map((record) => (
                <div key={record.id} className="grid gap-2 py-4 md:grid-cols-[140px_160px_1fr_auto] md:items-start">
                  <span>{formatDate(record.performed_at, locale)}</span>
                  <span>{record.technician}</span>
                  <div>
                    <strong>{record.service}</strong>
                    {record.notes ? <p className="text-muted-foreground text-sm">{record.notes}</p> : null}
                  </div>
                  <Badge variant={record.stock_consumption_pending ? "destructive" : "outline"}>
                    {record.maintenance_type === "preventive"
                      ? t("maintenance.preventive")
                      : t("maintenance.corrective")}
                  </Badge>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-muted-foreground text-sm">{t("valves.noTimeline")}</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function Detail({ label, value }: { readonly label: string; readonly value: React.ReactNode }) {
  return (
    <div>
      <div className="text-muted-foreground text-xs">{label}</div>
      <div className="mt-1 font-medium">{value || "-"}</div>
    </div>
  );
}

function Health({ health, t }: { health: "ok" | "warning" | "critical"; t: Awaited<ReturnType<typeof getI18n>>["t"] }) {
  const label =
    health === "ok" ? t("overview.onTrack") : health === "warning" ? t("common.warning") : t("common.critical");
  return (
    <Badge variant={health === "critical" ? "destructive" : health === "warning" ? "secondary" : "outline"}>
      {label}
    </Badge>
  );
}

function formatDate(value: string | null | undefined, locale: keyof typeof INTL_LOCALES) {
  return value
    ? new Intl.DateTimeFormat(INTL_LOCALES[locale]).format(new Date(`${value.slice(0, 10)}T00:00:00`))
    : null;
}

function editFields(
  valve: Awaited<ReturnType<typeof listValves>>["items"][number],
  t: Awaited<ReturnType<typeof getI18n>>["t"],
): CommandField[] {
  return [
    { name: "id", label: "ID", type: "hidden", defaultValue: valve.id },
    { name: "tag", label: "TAG", required: true, defaultValue: valve.tag },
    { name: "zone", label: t("common.zone"), required: true, defaultValue: valve.zone },
    { name: "manufacturer", label: t("common.manufacturer"), defaultValue: valve.manufacturer ?? "" },
    { name: "serial", label: t("valves.serial"), defaultValue: valve.serial ?? "" },
    { name: "valve_type", label: t("valves.valveType"), defaultValue: valve.valve_type ?? "" },
    { name: "dn", label: "DN", defaultValue: valve.dn ?? "" },
    { name: "kit_reference", label: t("valves.kitReference"), defaultValue: valve.kit_reference ?? "" },
    { name: "seat", label: t("valves.seat"), defaultValue: valve.seat ?? "" },
    { name: "actuator", label: t("valves.actuator"), defaultValue: valve.actuator ?? "" },
    {
      name: "manufactured_at",
      label: t("valves.manufactured"),
      type: "date",
      defaultValue: valve.manufactured_at?.slice(0, 10) ?? "",
    },
  ];
}
