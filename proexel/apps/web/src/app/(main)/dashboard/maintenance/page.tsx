import { Plus, Wrench } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { INTL_LOCALES } from "@/lib/i18n/config";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listMaintenance, listValves } from "@/lib/proexel/service";

import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function MaintenancePage() {
  const [{ role }, maintenance, valves, { t, locale }] = await Promise.all([
    requirePermission("maintenance.read"),
    listMaintenance(),
    listValves(),
    getI18n(),
  ]);
  return (
    <div>
      <PageHeader
        title={t("nav.maintenance")}
        description={t("maintenance.description")}
        action={
          can("maintenance.register", role) ? (
            <CommandDialog
              trigger={
                <Button disabled={valves.items.length === 0}>
                  <Plus />
                  {t("maintenance.register")}
                </Button>
              }
              title={t("maintenance.register")}
              description={t("maintenance.registerDescription")}
              endpoint="/api/proexel/maintenance"
              fields={[
                {
                  name: "valve_id",
                  label: t("maintenance.valve"),
                  type: "select",
                  required: true,
                  options: valves.items.map((valve) => ({ label: `${valve.tag} · ${valve.zone}`, value: valve.id })),
                },
                {
                  name: "performed_at",
                  label: t("common.date"),
                  type: "date",
                  required: true,
                  defaultValue: new Date().toISOString().slice(0, 10),
                },
                { name: "technician", label: t("common.technician"), required: true },
                {
                  name: "maintenance_type",
                  label: t("common.type"),
                  type: "select",
                  required: true,
                  options: [
                    { label: t("maintenance.preventive"), value: "preventive" },
                    { label: t("maintenance.corrective"), value: "corrective" },
                  ],
                },
                { name: "service", label: t("maintenance.service"), type: "textarea", required: true },
                { name: "notes", label: t("common.notes"), type: "textarea" },
                { name: "signature_ref", label: t("maintenance.signature") },
                { name: "kit_changed", label: t("maintenance.kitChanged"), type: "checkbox" },
              ]}
            />
          ) : null
        }
      />
      <Card>
        <CardHeader>
          <CardTitle>{t("maintenance.history")}</CardTitle>
          <CardDescription>{t("maintenance.records", { count: maintenance.items.length })}</CardDescription>
        </CardHeader>
        <CardContent>
          {maintenance.items.length === 0 ? (
            <ProexelEmptyState
              icon={Wrench}
              title={t("maintenance.none")}
              description={t("maintenance.noneDescription")}
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("common.date")}</TableHead>
                    <TableHead>TAG</TableHead>
                    <TableHead>{t("common.technician")}</TableHead>
                    <TableHead>{t("common.type")}</TableHead>
                    <TableHead>{t("maintenance.service")}</TableHead>
                    <TableHead>{t("valves.kit")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {maintenance.items.map((item) => (
                    <TableRow key={item.id}>
                      <TableCell>
                        {new Intl.DateTimeFormat(INTL_LOCALES[locale]).format(
                          new Date(`${item.performed_at}T00:00:00`),
                        )}
                      </TableCell>
                      <TableCell className="font-medium">{item.valve_tag_snapshot}</TableCell>
                      <TableCell>{item.technician}</TableCell>
                      <TableCell>
                        {item.maintenance_type === "preventive"
                          ? t("maintenance.preventive")
                          : t("maintenance.corrective")}
                      </TableCell>
                      <TableCell className="max-w-80 whitespace-normal">{item.service}</TableCell>
                      <TableCell>
                        {!item.kit_changed ? (
                          <Badge variant="outline">{t("maintenance.noChange")}</Badge>
                        ) : item.stock_consumed ? (
                          <Badge>{t("maintenance.consumed")}</Badge>
                        ) : (
                          <Badge variant="destructive">{t("maintenance.pendingConsumption")}</Badge>
                        )}
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
