import { CalendarClock, Plus, Trash2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { INTL_LOCALES } from "@/lib/i18n/config";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listServiceOrders, listValves } from "@/lib/proexel/service";

import { CommandButton } from "../_components/command-button";
import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function OrdersPage() {
  const [{ role }, orders, valves, { t, locale }] = await Promise.all([
    requirePermission("order.read"),
    listServiceOrders(),
    listValves({ page_size: 500 }),
    getI18n(),
  ]);
  return (
    <div>
      <PageHeader
        title={t("nav.orders")}
        description={t("orders.description")}
        action={
          can("order.create", role) ? (
            <CommandDialog
              trigger={
                <Button>
                  <Plus />
                  {t("orders.new")}
                </Button>
              }
              title={t("orders.create")}
              description={t("orders.createDescription")}
              endpoint="/api/proexel/orders"
              fields={[
                { name: "zone", label: t("common.zone"), required: true },
                {
                  name: "valve_id",
                  label: t("orders.optionalValve"),
                  type: "select",
                  options: valves.items.map((valve) => ({ label: valve.tag, value: valve.id })),
                },
                {
                  name: "priority",
                  label: t("common.priority"),
                  type: "select",
                  required: true,
                  options: [
                    { label: t("orders.low"), value: "low" },
                    { label: t("orders.normal"), value: "normal" },
                    { label: t("orders.high"), value: "high" },
                    { label: t("orders.urgent"), value: "urgent" },
                  ],
                  defaultValue: "normal",
                },
                { name: "technician", label: t("orders.responsible") },
                { name: "scheduled_for", label: t("orders.scheduledDate"), type: "date" },
                { name: "description", label: t("common.description"), type: "textarea", required: true },
              ]}
            />
          ) : null
        }
      />
      <Card>
        <CardHeader>
          <CardTitle>{t("orders.agenda")}</CardTitle>
          <CardDescription>{t("orders.count", { count: orders.items.length })}</CardDescription>
        </CardHeader>
        <CardContent>
          {orders.items.length === 0 ? (
            <ProexelEmptyState
              icon={CalendarClock}
              title={t("orders.none")}
              description={t("orders.noneDescription")}
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("orders.zoneTag")}</TableHead>
                    <TableHead>{t("common.description")}</TableHead>
                    <TableHead>{t("common.priority")}</TableHead>
                    <TableHead>{t("orders.scheduled")}</TableHead>
                    <TableHead>{t("common.status")}</TableHead>
                    <TableHead className="text-right">{t("common.actions")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {orders.items.map((order) => (
                    <TableRow key={order.id}>
                      <TableCell>
                        <div className="font-medium">{order.zone}</div>
                        <div className="text-muted-foreground text-xs">
                          {order.valve_tag_snapshot ?? t("orders.noValve")}
                        </div>
                      </TableCell>
                      <TableCell className="max-w-80 whitespace-normal">{order.description}</TableCell>
                      <TableCell>
                        {t(
                          `orders.${order.priority}` as
                            | "orders.low"
                            | "orders.normal"
                            | "orders.high"
                            | "orders.urgent",
                        )}
                      </TableCell>
                      <TableCell>
                        {order.scheduled_for
                          ? new Intl.DateTimeFormat(INTL_LOCALES[locale]).format(
                              new Date(`${order.scheduled_for}T00:00:00`),
                            )
                          : "-"}
                      </TableCell>
                      <TableCell>
                        <OrderStatus
                          status={order.status}
                          label={
                            order.status === "pending"
                              ? t("common.pending")
                              : order.status === "in_progress"
                                ? t("common.inProgress")
                                : t("common.completed")
                          }
                        />
                      </TableCell>
                      <TableCell>
                        <div className="flex justify-end gap-2">
                          {can("order.change_status", role) && order.status === "pending" ? (
                            <CommandButton
                              endpoint="/api/proexel/orders"
                              data={{ id: order.id, status: "in_progress" }}
                            >
                              {t("orders.start")}
                            </CommandButton>
                          ) : null}
                          {can("order.change_status", role) && order.status !== "completed" ? (
                            <CommandButton endpoint="/api/proexel/orders" data={{ id: order.id, status: "completed" }}>
                              {t("orders.finish")}
                            </CommandButton>
                          ) : null}
                          {can("order.delete", role) && order.status !== "completed" ? (
                            <CommandButton
                              endpoint="/api/proexel/orders"
                              data={{ id: order.id }}
                              method="DELETE"
                              variant="destructive"
                              confirmMessage={t("orders.deleteConfirm")}
                            >
                              <Trash2 />
                              <span className="sr-only">{t("common.delete")}</span>
                            </CommandButton>
                          ) : null}
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          )}
        </CardContent>
      </Card>
      <Card className="mt-4">
        <CardHeader>
          <CardTitle>{t("orders.scheduleView")}</CardTitle>
          <CardDescription>{t("orders.scheduledDate")}</CardDescription>
        </CardHeader>
        <CardContent className="divide-y">
          {orders.items
            .filter((order) => order.status !== "completed")
            .toSorted((left, right) => (left.scheduled_for ?? "9999").localeCompare(right.scheduled_for ?? "9999"))
            .map((order) => {
              const critical = valves.items.filter(
                (valve) => valve.zone === order.zone && valve.health === "critical",
              ).length;
              return (
                <div key={order.id} className="grid gap-2 py-3 sm:grid-cols-[140px_1fr_auto] sm:items-center">
                  <span>
                    {order.scheduled_for
                      ? new Intl.DateTimeFormat(INTL_LOCALES[locale]).format(
                          new Date(`${order.scheduled_for}T00:00:00`),
                        )
                      : t("orders.unscheduled")}
                  </span>
                  <div>
                    <strong>{order.zone}</strong>
                    <p className="text-muted-foreground text-sm">{order.description}</p>
                  </div>
                  <Badge variant={critical ? "destructive" : "outline"}>
                    {critical
                      ? t("reports.criticalCount", {
                          critical,
                          total: valves.items.filter((valve) => valve.zone === order.zone).length,
                        })
                      : t("overview.onTrack")}
                  </Badge>
                </div>
              );
            })}
        </CardContent>
      </Card>
    </div>
  );
}

function OrderStatus({
  status,
  label,
}: {
  readonly status: "pending" | "in_progress" | "completed";
  readonly label: string;
}) {
  return (
    <Badge variant={status === "completed" ? "outline" : status === "in_progress" ? "default" : "secondary"}>
      {label}
    </Badge>
  );
}
