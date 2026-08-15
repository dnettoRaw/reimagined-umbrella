import { Activity, Plus, Search } from "lucide-react";

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

export default async function ValvesPage({ searchParams }: { readonly searchParams: Promise<{ q?: string }> }) {
  const { q = "" } = await searchParams;
  const [{ role }, valves, { t, locale }] = await Promise.all([
    requirePermission("valve.read"),
    listValves(q ? { search: q } : {}),
    getI18n(),
  ]);

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
              fields={[
                { name: "tag", label: t("valves.tag"), required: true, placeholder: t("valves.tagPlaceholder") },
                { name: "zone", label: t("common.zone"), required: true, placeholder: t("valves.zonePlaceholder") },
                { name: "manufacturer", label: t("common.manufacturer") },
                { name: "serial", label: t("valves.serial") },
                { name: "valve_type", label: t("valves.valveType") },
                { name: "dn", label: "DN" },
                { name: "kit_reference", label: t("valves.kitReference") },
                { name: "seat", label: t("valves.seat") },
                { name: "actuator", label: t("valves.actuator") },
                { name: "manufactured_at", label: t("valves.manufactured"), type: "date" },
              ]}
            />
          ) : null
        }
      />

      <Card>
        <CardHeader className="gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <CardTitle>{t("valves.registry")}</CardTitle>
            <CardDescription>
              {t("valves.results", {
                count: valves.items.length,
                source: valves.source === "appcore" ? "AppCore" : t("common.unavailable"),
              })}
            </CardDescription>
          </div>
          <form className="flex w-full gap-2 sm:w-80">
            <Input
              name="q"
              defaultValue={q}
              placeholder={t("valves.searchPlaceholder")}
              aria-label={t("valves.searchAria")}
            />
            <Button type="submit" size="icon" variant="outline" title={t("common.search")}>
              <Search />
            </Button>
          </form>
        </CardHeader>
        <CardContent>
          {valves.items.length === 0 ? (
            <ProexelEmptyState
              icon={Activity}
              title={q ? t("valves.noneFound") : t("valves.none")}
              description={q ? t("valves.reviewSearch") : t("valves.createFirst")}
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
                        <HealthBadge
                          health={valve.health}
                          label={
                            valve.health === "ok"
                              ? t("overview.onTrack")
                              : valve.health === "warning"
                                ? t("common.warning")
                                : t("common.critical")
                          }
                        />
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function HealthBadge({ health, label }: { readonly health: "ok" | "warning" | "critical"; readonly label: string }) {
  return (
    <Badge variant={health === "critical" ? "destructive" : health === "warning" ? "secondary" : "outline"}>
      {label}
    </Badge>
  );
}
