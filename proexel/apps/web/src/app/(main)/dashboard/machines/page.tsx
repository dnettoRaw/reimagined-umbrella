import Link from "next/link";

import { Eye, Factory, Plus, Search } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listMachines } from "@/lib/proexel/service";

import { CommandDialog } from "../_components/command-dialog";
import { OperationalStatusBadge } from "../_components/operational-status-badge";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function MachinesPage({
  searchParams,
}: {
  readonly searchParams: Promise<{ q?: string; zone?: string; page?: string }>;
}) {
  const params = await searchParams;
  const page = Math.max(1, Number(params.page ?? 1));
  const [{ role }, machines, { t }] = await Promise.all([
    requirePermission("machine.read"),
    listMachines({ search: params.q ?? "", zone: params.zone ?? "", page, page_size: 25 }),
    getI18n(),
  ]);
  return (
    <div>
      <PageHeader
        title={t("nav.machines")}
        description={t("machines.description")}
        action={
          can("machine.create", role) ? (
            <CommandDialog
              trigger={
                <Button>
                  <Plus />
                  {t("machines.new")}
                </Button>
              }
              title={t("machines.create")}
              description={t("machines.description")}
              endpoint="/api/proexel/machines"
              fields={machineFields(t)}
            />
          ) : null
        }
      />
      <Card>
        <CardHeader className="gap-4">
          <div>
            <CardTitle>{t("machines.registry")}</CardTitle>
            <CardDescription>{t("machines.itemCount", { count: machines.total })}</CardDescription>
          </div>
          <form className="flex flex-wrap gap-2">
            <Input className="min-w-56 flex-1" name="q" defaultValue={params.q} placeholder={t("machines.search")} />
            <select
              name="zone"
              defaultValue={params.zone ?? ""}
              className="h-9 rounded-md border bg-background px-3 text-sm"
            >
              <option value="">{t("machines.allZones")}</option>
              {machines.facets.zones.map((zone) => (
                <option key={zone} value={zone}>
                  {zone}
                </option>
              ))}
            </select>
            <Button type="submit" size="icon" variant="outline" title={t("common.search")}>
              <Search />
            </Button>
          </form>
        </CardHeader>
        <CardContent>
          {machines.items.length ? (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("common.code")}</TableHead>
                    <TableHead>{t("common.name")}</TableHead>
                    <TableHead>{t("common.zone")}</TableHead>
                    <TableHead>{t("common.components")}</TableHead>
                    <TableHead>{t("common.status")}</TableHead>
                    <TableHead className="text-right">{t("common.actions")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {machines.items.map((machine) => (
                    <TableRow key={machine.id}>
                      <TableCell className="font-medium">{machine.code}</TableCell>
                      <TableCell>{machine.name}</TableCell>
                      <TableCell>{machine.zone}</TableCell>
                      <TableCell>{machine.items.length}</TableCell>
                      <TableCell>
                        <OperationalStatusBadge status={machine.status} />
                      </TableCell>
                      <TableCell className="text-right">
                        <Button asChild size="icon-sm" variant="ghost" title={t("common.details")}>
                          <Link href={`/dashboard/machines/${encodeURIComponent(machine.id)}`}>
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
          ) : (
            <ProexelEmptyState icon={Factory} title={t("machines.none")} description={t("machines.noneDescription")} />
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function machineFields(t: Awaited<ReturnType<typeof getI18n>>["t"]) {
  return [
    { name: "code", label: t("common.code"), required: true },
    { name: "name", label: t("common.name"), required: true },
    { name: "zone", label: t("common.zone"), required: true },
    { name: "location", label: t("common.location") },
    { name: "manufacturer", label: t("common.manufacturer") },
    { name: "model", label: t("common.model") },
    { name: "serial_number", label: t("common.serialNumber") },
    { name: "description", label: t("common.description"), type: "textarea" as const },
    { name: "active", label: t("common.active"), type: "checkbox" as const, defaultValue: true },
  ];
}
