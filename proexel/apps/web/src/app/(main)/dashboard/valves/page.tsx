import Link from "next/link";

import { Activity, Eye, Plus, Search, SlidersHorizontal } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { INTL_LOCALES } from "@/lib/i18n/config";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listValves } from "@/lib/proexel/service";

import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

type ValveSearch = {
  q?: string;
  zone?: string;
  health?: string;
  valve_type?: string;
  sort?: string;
  direction?: string;
  page?: string;
};

export default async function ValvesPage({ searchParams }: { readonly searchParams: Promise<ValveSearch> }) {
  const params = await searchParams;
  const page = Math.max(1, Number.parseInt(params.page ?? "1", 10) || 1);
  const [{ role }, valves, { t, locale }] = await Promise.all([
    requirePermission("valve.read"),
    listValves({
      search: params.q ?? "",
      zone: params.zone ?? "",
      health: params.health ?? "",
      valve_type: params.valve_type ?? "",
      sort: params.sort ?? "tag",
      direction: params.direction ?? "asc",
      page,
      page_size: 25,
    }),
    getI18n(),
  ]);
  const pages = Math.max(1, Math.ceil(valves.total / valves.page_size));

  return (
    <div>
      <PageHeader
        title={t("nav.valves")}
        description={t("valves.description")}
        action={
          can("valve.create", role) ? (
            <CommandDialog
              trigger={
                <Button>
                  <Plus />
                  {t("valves.new")}
                </Button>
              }
              title={t("valves.create")}
              description={t("valves.createDescription")}
              endpoint="/api/proexel/valves"
              fields={valveFields(t)}
            />
          ) : null
        }
      />

      <Card>
        <CardHeader className="gap-4">
          <div className="flex flex-wrap items-end justify-between gap-3">
            <div>
              <CardTitle>{t("valves.registry")}</CardTitle>
              <CardDescription>
                {t("valves.showing", { count: valves.items.length, total: valves.total })} ·{" "}
                {t("valves.results", {
                  count: valves.total,
                  source: valves.source === "appcore" ? "AppCore" : t("common.unavailable"),
                })}
              </CardDescription>
            </div>
            <div className="flex items-center gap-2 text-muted-foreground text-sm">
              <SlidersHorizontal className="size-4" />
              {t("valves.filters")}
            </div>
          </div>
          <form className="grid gap-2 md:grid-cols-2 xl:grid-cols-[minmax(220px,2fr)_repeat(5,minmax(130px,1fr))_auto]">
            <div className="flex gap-2">
              <Input
                name="q"
                defaultValue={params.q}
                placeholder={t("valves.searchPlaceholder")}
                aria-label={t("valves.searchAria")}
              />
              <Button type="submit" size="icon" variant="outline" title={t("common.search")}>
                <Search />
              </Button>
            </div>
            <FilterSelect
              name="zone"
              defaultValue={params.zone}
              label={t("valves.allZones")}
              values={valves.facets.zones}
            />
            <select
              name="health"
              defaultValue={params.health ?? ""}
              className="h-8 rounded-lg border bg-background px-2 text-sm"
            >
              <option value="">{t("valves.allHealth")}</option>
              <option value="ok">{t("overview.onTrack")}</option>
              <option value="warning">{t("common.warning")}</option>
              <option value="critical">{t("common.critical")}</option>
            </select>
            <FilterSelect
              name="valve_type"
              defaultValue={params.valve_type}
              label={t("valves.allTypes")}
              values={valves.facets.valve_types}
            />
            <select
              name="sort"
              defaultValue={params.sort ?? "tag"}
              className="h-8 rounded-lg border bg-background px-2 text-sm"
            >
              <option value="tag">{t("valves.sortTag")}</option>
              <option value="zone">{t("valves.sortZone")}</option>
              <option value="health">{t("valves.sortHealth")}</option>
              <option value="last_maintenance">{t("valves.sortMaintenance")}</option>
            </select>
            <select
              name="direction"
              defaultValue={params.direction ?? "asc"}
              className="h-8 rounded-lg border bg-background px-2 text-sm"
            >
              <option value="asc">{t("valves.ascending")}</option>
              <option value="desc">{t("valves.descending")}</option>
            </select>
            <Button type="submit" variant="secondary">
              {t("valves.applyFilters")}
            </Button>
          </form>
        </CardHeader>
        <CardContent>
          {valves.items.length === 0 ? (
            <ProexelEmptyState
              icon={Activity}
              title={params.q ? t("valves.noneFound") : t("valves.none")}
              description={params.q ? t("valves.reviewSearch") : t("valves.createFirst")}
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>TAG</TableHead>
                    <TableHead>{t("common.zone")}</TableHead>
                    <TableHead>{t("common.manufacturer")}</TableHead>
                    <TableHead>{t("valves.typeDn")}</TableHead>
                    <TableHead>{t("valves.kit")}</TableHead>
                    <TableHead>{t("valves.lastMaintenance")}</TableHead>
                    <TableHead>{t("common.health")}</TableHead>
                    <TableHead className="text-right">{t("common.actions")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {valves.items.map((valve) => (
                    <TableRow key={valve.id}>
                      <TableCell className="font-medium">{valve.tag}</TableCell>
                      <TableCell>{valve.zone}</TableCell>
                      <TableCell>{valve.manufacturer ?? "-"}</TableCell>
                      <TableCell>{[valve.valve_type, valve.dn].filter(Boolean).join(" / ") || "-"}</TableCell>
                      <TableCell>{valve.kit_reference ?? "-"}</TableCell>
                      <TableCell>
                        {valve.last_maintenance_at
                          ? new Intl.DateTimeFormat(INTL_LOCALES[locale]).format(
                              new Date(`${valve.last_maintenance_at}T00:00:00`),
                            )
                          : t("common.never")}
                      </TableCell>
                      <TableCell>
                        <HealthBadge health={valve.health} label={healthLabel(valve.health, t)} />
                      </TableCell>
                      <TableCell className="text-right">
                        <Button asChild size="icon-sm" variant="ghost" title={t("common.details")}>
                          <Link href={`/dashboard/valves/${encodeURIComponent(valve.id)}`}>
                            <Eye />
                            <span className="sr-only">{t("common.details")}</span>
                          </Link>
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          )}
          {valves.total > 0 ? (
            <div className="mt-4 flex items-center justify-between border-t pt-4">
              <span className="text-muted-foreground text-sm">{t("valves.page", { page: valves.page, pages })}</span>
              <div className="flex gap-2">
                <Button asChild={valves.page > 1} size="sm" variant="outline" disabled={valves.page <= 1}>
                  {valves.page > 1 ? (
                    <Link href={pageHref(params, valves.page - 1)}>{t("common.previous")}</Link>
                  ) : (
                    <span>{t("common.previous")}</span>
                  )}
                </Button>
                <Button asChild={valves.page < pages} size="sm" variant="outline" disabled={valves.page >= pages}>
                  {valves.page < pages ? (
                    <Link href={pageHref(params, valves.page + 1)}>{t("common.next")}</Link>
                  ) : (
                    <span>{t("common.next")}</span>
                  )}
                </Button>
              </div>
            </div>
          ) : null}
        </CardContent>
      </Card>
    </div>
  );
}

function FilterSelect({
  name,
  defaultValue,
  label,
  values,
}: {
  name: string;
  defaultValue?: string;
  label: string;
  values: string[];
}) {
  return (
    <select name={name} defaultValue={defaultValue ?? ""} className="h-8 rounded-lg border bg-background px-2 text-sm">
      <option value="">{label}</option>
      {values.map((value) => (
        <option key={value} value={value}>
          {value}
        </option>
      ))}
    </select>
  );
}

function pageHref(params: ValveSearch, page: number) {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) if (value && key !== "page") search.set(key, value);
  search.set("page", String(page));
  return `/dashboard/valves?${search.toString()}`;
}

function valveFields(t: Awaited<ReturnType<typeof getI18n>>["t"]) {
  return [
    { name: "tag", label: t("valves.tag"), required: true, placeholder: t("valves.tagPlaceholder") },
    { name: "zone", label: t("common.zone"), required: true, placeholder: t("valves.zonePlaceholder") },
    { name: "manufacturer", label: t("common.manufacturer") },
    { name: "serial", label: t("valves.serial") },
    { name: "valve_type", label: t("valves.valveType") },
    { name: "dn", label: "DN" },
    { name: "kit_reference", label: t("valves.kitReference") },
    { name: "seat", label: t("valves.seat") },
    { name: "actuator", label: t("valves.actuator") },
    { name: "manufactured_at", label: t("valves.manufactured"), type: "date" as const },
  ];
}

function healthLabel(health: "ok" | "warning" | "critical", t: Awaited<ReturnType<typeof getI18n>>["t"]) {
  return health === "ok" ? t("overview.onTrack") : health === "warning" ? t("common.warning") : t("common.critical");
}

function HealthBadge({ health, label }: { readonly health: "ok" | "warning" | "critical"; readonly label: string }) {
  return (
    <Badge variant={health === "critical" ? "destructive" : health === "warning" ? "secondary" : "outline"}>
      {label}
    </Badge>
  );
}
