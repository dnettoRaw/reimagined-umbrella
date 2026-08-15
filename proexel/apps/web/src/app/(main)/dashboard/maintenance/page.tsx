import { ExternalLink, Wrench } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { INTL_LOCALES } from "@/lib/i18n/config";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listMaintenance, listStock, listValves } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";
import { MaintenanceWizard } from "./maintenance-wizard";

export const dynamic = "force-dynamic";

export default async function MaintenancePage() {
  const [session, maintenance, valves, stock, { t, locale }] = await Promise.all([
    requirePermission("maintenance.read"),
    listMaintenance(),
    listValves({ page_size: 500 }),
    listStock(),
    getI18n(),
  ]);
  return (
    <div>
      <PageHeader
        title={t("nav.maintenance")}
        description={t("maintenance.description")}
        action={
          can("maintenance.register", session.role) ? (
            <MaintenanceWizard valves={valves.items} stock={stock.items} technician={session.name} />
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
                    <TableHead>{t("maintenance.stepSignature")}</TableHead>
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
                      <TableCell>
                        {item.signature_ref ? (
                          <Button asChild size="icon-sm" variant="ghost" title={t("maintenance.viewSignature")}>
                            <a
                              href={`/api/proexel/attachments?ref=${encodeURIComponent(item.signature_ref)}`}
                              target="_blank"
                              rel="noreferrer"
                            >
                              <ExternalLink />
                              <span className="sr-only">{t("maintenance.viewSignature")}</span>
                            </a>
                          </Button>
                        ) : (
                          "-"
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
