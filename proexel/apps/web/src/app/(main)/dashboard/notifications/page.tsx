import Link from "next/link";

import { Bell, CalendarClock, PackageSearch, ServerCrash, Wrench } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { INTL_LOCALES } from "@/lib/i18n/config";
import { getI18n } from "@/lib/i18n/server";
import { requireSession } from "@/lib/proexel/auth-server";
import { getRuntimeStatus, listMachines, listServiceOrders, listStock } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function NotificationsPage() {
  await requireSession();
  const [machines, stock, orders, runtime, { t, locale }] = await Promise.all([
    listMachines({ page_size: 500 }),
    listStock(),
    listServiceOrders(),
    getRuntimeStatus(),
    getI18n(),
  ]);
  const criticalItems = machines.items.flatMap((machine) =>
    machine.items
      .filter((item) => item.status === "critical" || item.status === "maintenance_required")
      .map((item) => ({ machine, item })),
  );
  const lowStock = stock.items.filter((item) => item.quantity <= item.minimum_quantity);
  const openOrders = orders.items
    .filter((order) => order.status === "pending" || order.status === "in_progress")
    .toSorted((left, right) => (left.scheduled_for ?? "9999").localeCompare(right.scheduled_for ?? "9999"));
  const hasAlerts = !runtime.healthy || criticalItems.length + lowStock.length + openOrders.length > 0;
  const today = new Date().toISOString().slice(0, 10);
  return (
    <div>
      <PageHeader title={t("nav.notifications")} description={t("notifications.description")} />
      {!hasAlerts ? (
        <Card>
          <CardContent className="pt-6">
            <ProexelEmptyState
              icon={Bell}
              title={t("notifications.none")}
              description={t("notifications.noneDescription")}
            />
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 xl:grid-cols-3">
          {!runtime.healthy ? (
            <AlertCard title={t("notifications.runtime")} icon={ServerCrash} count={1}>
              <div className="flex items-center justify-between gap-3 py-3">
                <p className="text-muted-foreground text-sm">{t("notifications.runtimeUnavailable")}</p>
                <Badge variant="destructive">{t("common.unavailable")}</Badge>
              </div>
            </AlertCard>
          ) : null}
          <AlertCard title={t("notifications.criticalItems")} icon={Wrench} count={criticalItems.length}>
            {criticalItems.map(({ machine, item }) => (
              <Link
                key={item.id}
                href={`/dashboard/machines/${encodeURIComponent(machine.id)}`}
                className="flex items-center justify-between gap-3 border-b py-3 last:border-0"
              >
                <div>
                  <strong>
                    {item.code} · {item.name}
                  </strong>
                  <p className="text-muted-foreground text-xs">
                    {machine.code} · {machine.zone}
                  </p>
                </div>
                <Badge variant="destructive">{t(`status.${item.status}`)}</Badge>
              </Link>
            ))}
          </AlertCard>
          <AlertCard title={t("notifications.lowStock")} icon={PackageSearch} count={lowStock.length}>
            {lowStock.map((item) => (
              <div key={item.id} className="flex items-center justify-between gap-3 border-b py-3 last:border-0">
                <div>
                  <strong>{item.reference}</strong>
                  <p className="text-muted-foreground text-xs">{item.location ?? "-"}</p>
                </div>
                <Badge variant="destructive">
                  {item.quantity} / {item.minimum_quantity}
                </Badge>
              </div>
            ))}
          </AlertCard>
          <AlertCard title={t("notifications.openOrders")} icon={CalendarClock} count={openOrders.length}>
            {openOrders.map((order) => {
              const overdue = Boolean(order.scheduled_for && order.scheduled_for < today);
              return (
                <Link
                  key={order.id}
                  href={`/dashboard/execution/${encodeURIComponent(order.id)}`}
                  className="flex items-center justify-between gap-3 border-b py-3 last:border-0"
                >
                  <div>
                    <strong>{order.machine_snapshot.code}</strong>
                    <p className="line-clamp-1 text-muted-foreground text-xs">{order.description}</p>
                  </div>
                  <Badge variant={overdue ? "destructive" : "outline"}>
                    {overdue
                      ? t("notifications.overdue")
                      : order.scheduled_for
                        ? new Intl.DateTimeFormat(INTL_LOCALES[locale]).format(
                            new Date(`${order.scheduled_for}T00:00:00`),
                          )
                        : t("orders.unscheduled")}
                  </Badge>
                </Link>
              );
            })}
          </AlertCard>
        </div>
      )}
    </div>
  );
}

function AlertCard({
  title,
  icon: Icon,
  count,
  children,
}: {
  title: string;
  icon: typeof Bell;
  count: number;
  children: React.ReactNode;
}) {
  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <div>
          <CardTitle>{title}</CardTitle>
          <CardDescription>{count}</CardDescription>
        </div>
        <Icon className="size-5 text-muted-foreground" />
      </CardHeader>
      <CardContent>{count ? children : <p className="text-muted-foreground text-sm">-</p>}</CardContent>
    </Card>
  );
}
