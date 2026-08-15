import Link from "next/link";

import { ClipboardList, ExternalLink, Plus, Trash2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listMachines, listOperators, listServiceOrders } from "@/lib/proexel/service";
import type { OperatorSummary, ServiceOrderStatus } from "@/lib/proexel/types";

import { CommandButton } from "../_components/command-button";
import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";
import { OrderEditor } from "./order-editor";

export const dynamic = "force-dynamic";

export default async function OrdersPage() {
  const session = await requirePermission("order.read");
  const [orders, machines, operators, { t }] = await Promise.all([
    listServiceOrders(),
    listMachines({ page_size: 500 }),
    can("operator.read", session.role)
      ? listOperators()
      : Promise.resolve({ items: [], schema_version: 2, source: "unavailable" as const }),
    getI18n(),
  ]);
  return (
    <div>
      <PageHeader
        title={t("nav.orders")}
        description={t("orders.description")}
        action={
          can("order.create", session.role) ? (
            <OrderEditor
              machines={machines.items}
              operators={operators.items}
              trigger={
                <Button>
                  <Plus />
                  {t("orders.new")}
                </Button>
              }
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
          {orders.items.length ? (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("orders.machine")}</TableHead>
                    <TableHead>{t("common.description")}</TableHead>
                    <TableHead>{t("common.components")}</TableHead>
                    <TableHead>{t("common.priority")}</TableHead>
                    <TableHead>{t("common.status")}</TableHead>
                    <TableHead className="text-right">{t("common.actions")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {orders.items.map((order) => (
                    <TableRow key={order.id}>
                      <TableCell>
                        <strong>{order.machine_snapshot.code}</strong>
                        <div className="text-muted-foreground text-xs">
                          {order.machine_snapshot.name} · {order.machine_snapshot.zone}
                        </div>
                      </TableCell>
                      <TableCell className="max-w-72 whitespace-normal">{order.description}</TableCell>
                      <TableCell>
                        <div>
                          {t("orders.progress", { completed: order.completed_tasks, total: order.tasks.length })}
                        </div>
                        <div className="text-muted-foreground text-xs">
                          {t("orders.maxLevel", { level: order.maximum_complexity_level })}
                        </div>
                      </TableCell>
                      <TableCell>{t(`orders.${order.priority}`)}</TableCell>
                      <TableCell>
                        <OrderStatus status={order.status} label={statusLabel(order.status, t)} />
                      </TableCell>
                      <TableCell>
                        <div className="flex justify-end gap-1">
                          {can("order.manage", session.role) &&
                          order.status !== "completed" &&
                          order.status !== "cancelled"
                            ? order.tasks
                                .filter((task) => task.status === "pending")
                                .map((task) => (
                                  <Assign
                                    key={task.id}
                                    orderId={order.id}
                                    taskId={task.id}
                                    itemName={task.item_snapshot.code}
                                    level={task.complexity_snapshot}
                                    operators={operators.items}
                                    t={t}
                                  />
                                ))
                            : null}
                          <Button asChild size="icon-sm" variant="ghost" title={t("execution.openOrder")}>
                            <Link href={`/dashboard/execution/${encodeURIComponent(order.id)}`}>
                              <ExternalLink />
                              <span className="sr-only">{t("execution.openOrder")}</span>
                            </Link>
                          </Button>
                          {can("order.delete", session.role) && order.status === "pending" ? (
                            <CommandButton
                              endpoint="/api/proexel/orders"
                              method="DELETE"
                              data={{ id: order.id }}
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
          ) : (
            <ProexelEmptyState
              icon={ClipboardList}
              title={t("orders.none")}
              description={t("orders.noneDescription")}
            />
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function Assign({
  orderId,
  taskId,
  itemName,
  level,
  operators,
  t,
}: {
  orderId: string;
  taskId: string;
  itemName: string;
  level: number;
  operators: OperatorSummary[];
  t: Awaited<ReturnType<typeof getI18n>>["t"];
}) {
  const eligible = operators.filter((operator) => operator.maximum_repair_level >= level);
  return (
    <CommandDialog
      trigger={
        <Button size="sm" variant="ghost" title={t("orders.assign")}>
          <Plus />
          {itemName}
        </Button>
      }
      title={t("orders.assign")}
      description={t("orders.maxLevel", { level })}
      endpoint="/api/proexel/orders"
      method="PATCH"
      fields={[
        { name: "action", label: t("common.operation"), type: "hidden", defaultValue: "assign" },
        { name: "order_id", label: t("nav.orders"), type: "hidden", defaultValue: orderId },
        { name: "task_id", label: t("common.component"), type: "hidden", defaultValue: taskId },
        {
          name: "operator_id",
          label: t("orders.operator"),
          type: "select",
          required: true,
          options: eligible.map((operator) => ({
            value: operator.id,
            label: `${operator.name} (${operator.maximum_repair_level}/5)`,
          })),
        },
      ]}
    />
  );
}

function OrderStatus({ status, label }: { status: ServiceOrderStatus; label: string }) {
  return (
    <Badge
      variant={
        status === "completed"
          ? "outline"
          : status === "cancelled"
            ? "destructive"
            : status === "in_progress"
              ? "default"
              : "secondary"
      }
    >
      {label}
    </Badge>
  );
}

function statusLabel(status: ServiceOrderStatus, t: Awaited<ReturnType<typeof getI18n>>["t"]) {
  if (status === "pending") return t("common.pending");
  if (status === "in_progress") return t("common.inProgress");
  if (status === "completed") return t("common.completed");
  return t("orders.cancelled");
}
