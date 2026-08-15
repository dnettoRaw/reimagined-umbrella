import { History } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { INTL_LOCALES } from "@/lib/i18n/config";
import type { TranslationKey } from "@/lib/i18n/messages";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { listAudit } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function AuditPage() {
  const [, audit, { t, locale }] = await Promise.all([requirePermission("audit.read"), listAudit(), getI18n()]);
  const descriptions: Record<string, TranslationKey> = {
    "Valve created": "audit.valveCreated",
    "Valve updated": "audit.valveUpdated",
    "Maintenance registered": "audit.maintenanceRegistered",
    "Service order created": "audit.orderCreated",
    "Service order status changed": "audit.orderStatusChanged",
    "Restock requested": "audit.restockRequested",
    "Restock request reviewed": "audit.restockReviewed",
    "Stock adjusted": "audit.stockAdjusted",
    "Stock item updated": "audit.stockItemUpdated",
    "Stock item created": "audit.stockItemCreated",
    "Supplier created": "audit.supplierCreated",
    "Supplier updated": "audit.supplierUpdated",
  };
  const operations: Record<string, TranslationKey> = {
    "proexel.valves.create": "audit.valveCreated",
    "proexel.valves.update": "audit.valveUpdated",
    "proexel.maintenance.register": "audit.maintenanceRegistered",
    "proexel.orders.create": "audit.orderCreated",
    "proexel.orders.change_status": "audit.orderStatusChanged",
    "proexel.purchasing.create_restock_request": "audit.restockRequested",
    "proexel.purchasing.review_restock_request": "audit.restockReviewed",
    "proexel.stock.adjust": "audit.stockAdjusted",
    "proexel.stock.upsert_item": "audit.stockItemUpdated",
    "proexel.suppliers.create": "audit.supplierCreated",
    "proexel.suppliers.update": "audit.supplierUpdated",
  };
  const aggregates: Record<string, TranslationKey> = {
    Valve: "nav.valves",
    Maintenance: "nav.maintenance",
    ServiceOrder: "nav.orders",
    RestockRequest: "purchasing.requests",
    StockItem: "stock.items",
    Supplier: "nav.suppliers",
  };
  return (
    <div>
      <PageHeader title={t("audit.title")} description={t("audit.description")} />
      <Card>
        <CardHeader>
          <CardTitle>{t("audit.recent")}</CardTitle>
          <CardDescription>{t("audit.limit")}</CardDescription>
        </CardHeader>
        <CardContent>
          {audit.items.length === 0 ? (
            <ProexelEmptyState icon={History} title={t("audit.none")} description={t("audit.noneDescription")} />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("common.date")}</TableHead>
                    <TableHead>{t("audit.actor")}</TableHead>
                    <TableHead>{t("common.operation")}</TableHead>
                    <TableHead>{t("common.entity")}</TableHead>
                    <TableHead>{t("common.description")}</TableHead>
                    <TableHead>{t("common.result")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {audit.items.map((event) => (
                    <TableRow key={event.id}>
                      <TableCell>
                        {new Intl.DateTimeFormat(INTL_LOCALES[locale], {
                          dateStyle: "short",
                          timeStyle: "short",
                        }).format(new Date(event.created_at_ms))}
                      </TableCell>
                      <TableCell>
                        <div className="font-medium">{event.actor}</div>
                        <div className="text-muted-foreground text-xs">{t(`role.${event.role}`)}</div>
                      </TableCell>
                      <TableCell>
                        {operations[event.operation] ? t(operations[event.operation]) : event.operation}
                      </TableCell>
                      <TableCell>
                        {aggregates[event.aggregate] ? t(aggregates[event.aggregate]) : event.aggregate}
                      </TableCell>
                      <TableCell>
                        {event.description && descriptions[event.description]
                          ? t(descriptions[event.description])
                          : (event.description ?? "-")}
                      </TableCell>
                      <TableCell>
                        <Badge variant={event.result === "success" ? "outline" : "destructive"}>
                          {event.result === "success" ? t("common.success") : event.result}
                        </Badge>
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
