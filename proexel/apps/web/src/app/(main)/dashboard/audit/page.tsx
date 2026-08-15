import Link from "next/link";

import { History, Search } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { INTL_LOCALES } from "@/lib/i18n/config";
import type { TranslationKey } from "@/lib/i18n/messages";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { listAudit } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function AuditPage({
  searchParams,
}: {
  readonly searchParams: Promise<{
    q?: string;
    operation?: string;
    actor?: string;
    aggregate?: string;
    from?: string;
    to?: string;
    page?: string;
  }>;
}) {
  const params = await searchParams;
  const page = Math.max(1, Number.parseInt(params.page ?? "1", 10) || 1);
  const [, audit, { t, locale }] = await Promise.all([
    requirePermission("audit.read"),
    listAudit({
      search: params.q ?? "",
      operation: params.operation ?? "",
      actor: params.actor ?? "",
      aggregate: params.aggregate ?? "",
      from_ms: dateBoundary(params.from, false),
      to_ms: dateBoundary(params.to, true),
      page,
      page_size: 50,
    }),
    getI18n(),
  ]);
  const pages = Math.max(1, Math.ceil(audit.total / audit.page_size));
  const descriptions: Record<string, TranslationKey> = {
    "Item category created": "audit.categoryCreated",
    "Item category updated": "audit.categoryUpdated",
    "Maintenance guide updated": "audit.guideUpdated",
    "Machine created": "audit.machineCreated",
    "Machine updated": "audit.machineUpdated",
    "Machine item created": "audit.machineItemCreated",
    "Machine item updated": "audit.machineItemUpdated",
    "Machine items reordered": "audit.machineItemsReordered",
    "Machine item removed": "audit.machineItemRemoved",
    "Machine item physical unit replaced": "audit.machineItemReplaced",
    "Machine item status changed": "audit.itemStatusChanged",
    "Machine item status changed after inspection": "audit.itemStatusChanged",
    "Photo added": "audit.photoAdded",
    "Photo removed": "audit.photoRemoved",
    "Service order created with immutable item snapshots": "audit.orderCreated",
    "Service order started": "audit.orderStarted",
    "Service order task assigned": "audit.orderTaskAssigned",
    "Service order created": "audit.orderCreated",
    "Service order completed": "audit.orderCompleted",
    "Service order task started": "audit.orderTaskStarted",
    "Service order task completed": "audit.orderTaskCompleted",
    "Inspection started": "audit.inspectionStarted",
    "Inspection completed": "audit.inspectionCompleted",
    "Restock requested": "audit.restockRequested",
    "Restock request reviewed": "audit.restockReviewed",
    "Stock adjusted": "audit.stockAdjusted",
    "Stock item updated": "audit.stockItemUpdated",
    "Stock item created": "audit.stockItemCreated",
    "Supplier created": "audit.supplierCreated",
    "Supplier updated": "audit.supplierUpdated",
    "Service order deleted": "audit.orderDeleted",
    "Restock request deleted": "audit.restockDeleted",
    "Stock item deleted": "audit.stockDeleted",
    "Supplier deleted": "audit.supplierDeleted",
    "User created": "admin.userCreated",
    "User updated": "admin.userUpdated",
    "User credentials reset": "admin.userCredentialsReset",
  };
  const operations: Record<string, TranslationKey> = {
    "proexel.item_categories.create": "audit.categoryCreated",
    "proexel.item_categories.update": "audit.categoryUpdated",
    "proexel.machines.create": "audit.machineCreated",
    "proexel.machines.update": "audit.machineUpdated",
    "proexel.machine_items.add": "audit.machineItemCreated",
    "proexel.machine_items.update": "audit.machineItemUpdated",
    "proexel.machine_items.reorder": "audit.machineItemsReordered",
    "proexel.machine_items.remove": "audit.machineItemRemoved",
    "proexel.machine_items.replace": "audit.machineItemReplaced",
    "proexel.photos.add": "audit.photoAdded",
    "proexel.photos.delete": "audit.photoRemoved",
    "proexel.orders.create": "audit.orderCreated",
    "proexel.orders.start": "audit.orderStarted",
    "proexel.orders.assign_task": "audit.orderTaskAssigned",
    "proexel.orders.complete": "audit.orderCompleted",
    "proexel.inspections.start": "audit.inspectionStarted",
    "proexel.inspections.complete": "audit.inspectionCompleted",
    "maintenance_guide.updated": "audit.guideUpdated",
    "item.status_changed": "audit.itemStatusChanged",
    "service_order_task.started": "audit.orderTaskStarted",
    "service_order_task.completed": "audit.orderTaskCompleted",
    "proexel.purchasing.create_restock_request": "audit.restockRequested",
    "proexel.purchasing.review_restock_request": "audit.restockReviewed",
    "proexel.stock.adjust": "audit.stockAdjusted",
    "proexel.stock.upsert_item": "audit.stockItemUpdated",
    "proexel.suppliers.create": "audit.supplierCreated",
    "proexel.suppliers.update": "audit.supplierUpdated",
    "proexel.orders.delete": "audit.orderDeleted",
    "proexel.purchasing.delete_restock_request": "audit.restockDeleted",
    "proexel.stock.delete_item": "audit.stockDeleted",
    "proexel.suppliers.delete": "audit.supplierDeleted",
    "proexel.admin.users.create": "admin.userCreated",
    "proexel.admin.users.update": "admin.userUpdated",
    "proexel.admin.users.reset_credentials": "admin.userCredentialsReset",
  };
  const aggregates: Record<string, TranslationKey> = {
    item_category: "nav.categories",
    machine: "nav.machines",
    machine_item: "common.components",
    photo: "common.photo",
    service_order: "nav.orders",
    service_order_task: "nav.orders",
    inspection: "nav.execution",
    restock_request: "purchasing.requests",
    stock_item: "stock.items",
    supplier: "nav.suppliers",
    user_account: "admin.users",
  };
  return (
    <div>
      <PageHeader title={t("audit.title")} description={t("audit.description")} />
      <Card>
        <CardHeader className="gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <CardTitle>{t("audit.recent")}</CardTitle>
            <CardDescription>
              {t("audit.results", { count: audit.items.length, total: audit.total })} · {t("audit.limit")}
            </CardDescription>
          </div>
          <form className="grid w-full gap-2 sm:grid-cols-2 xl:grid-cols-[220px_180px_180px_160px_145px_145px_auto]">
            <Input name="q" defaultValue={params.q} placeholder={t("audit.search")} />
            <select
              name="actor"
              defaultValue={params.actor ?? ""}
              className="h-8 rounded-lg border bg-background px-2 text-sm"
            >
              <option value="">{t("audit.allActors")}</option>
              {audit.actors.map((actor) => (
                <option key={actor}>{actor}</option>
              ))}
            </select>
            <select
              name="aggregate"
              defaultValue={params.aggregate ?? ""}
              className="h-8 rounded-lg border bg-background px-2 text-sm"
            >
              <option value="">{t("audit.allEntities")}</option>
              {audit.aggregates.map((aggregate) => (
                <option key={aggregate} value={aggregate}>
                  {aggregates[aggregate] ? t(aggregates[aggregate]) : aggregate}
                </option>
              ))}
            </select>
            <select
              name="operation"
              defaultValue={params.operation ?? ""}
              className="h-8 rounded-lg border bg-background px-2 text-sm"
            >
              <option value="">{t("audit.allOperations")}</option>
              {audit.operations.map((operation) => (
                <option key={operation} value={operation}>
                  {operations[operation] ? t(operations[operation]) : operation}
                </option>
              ))}
            </select>
            <Input name="from" type="date" defaultValue={params.from} aria-label={t("audit.from")} />
            <Input name="to" type="date" defaultValue={params.to} aria-label={t("audit.to")} />
            <Button type="submit" size="icon" variant="outline" title={t("common.search")}>
              <Search />
            </Button>
          </form>
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
                        <div>
                          {event.description && descriptions[event.description]
                            ? t(descriptions[event.description])
                            : (event.description ?? "-")}
                        </div>
                        {event.before_json || event.after_json ? (
                          <details className="mt-1 text-xs">
                            <summary className="cursor-pointer text-muted-foreground">{t("audit.details")}</summary>
                            <div className="mt-2 grid gap-2 lg:grid-cols-2">
                              <AuditValue label={t("audit.before")} value={event.before_json} />
                              <AuditValue label={t("audit.after")} value={event.after_json} />
                            </div>
                          </details>
                        ) : null}
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
          {audit.total > 0 ? (
            <div className="mt-4 flex items-center justify-between border-t pt-4">
              <span className="text-muted-foreground text-sm">{t("common.page", { page: audit.page, pages })}</span>
              <div className="flex gap-2">
                <Button asChild={audit.page > 1} size="sm" variant="outline" disabled={audit.page <= 1}>
                  {audit.page > 1 ? (
                    <Link href={auditHref(params, audit.page - 1)}>{t("common.previous")}</Link>
                  ) : (
                    <span>{t("common.previous")}</span>
                  )}
                </Button>
                <Button asChild={audit.page < pages} size="sm" variant="outline" disabled={audit.page >= pages}>
                  {audit.page < pages ? (
                    <Link href={auditHref(params, audit.page + 1)}>{t("common.next")}</Link>
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

function AuditValue({ label, value }: { label: string; value?: string | null }) {
  return (
    <div>
      <strong>{label}</strong>
      <pre className="mt-1 max-h-48 max-w-80 overflow-auto rounded border bg-muted p-2">{value ?? "-"}</pre>
    </div>
  );
}

function dateBoundary(value: string | undefined, endOfDay: boolean) {
  if (!value) return endOfDay ? Number.MAX_SAFE_INTEGER : 0;
  const parsed = Date.parse(`${value}T${endOfDay ? "23:59:59.999" : "00:00:00.000"}Z`);
  return Number.isFinite(parsed) ? parsed : endOfDay ? Number.MAX_SAFE_INTEGER : 0;
}

function auditHref(
  params: { q?: string; operation?: string; actor?: string; aggregate?: string; from?: string; to?: string },
  page: number,
) {
  const search = new URLSearchParams();
  if (params.q) search.set("q", params.q);
  if (params.operation) search.set("operation", params.operation);
  if (params.actor) search.set("actor", params.actor);
  if (params.aggregate) search.set("aggregate", params.aggregate);
  if (params.from) search.set("from", params.from);
  if (params.to) search.set("to", params.to);
  search.set("page", String(page));
  return `/dashboard/audit?${search.toString()}`;
}
