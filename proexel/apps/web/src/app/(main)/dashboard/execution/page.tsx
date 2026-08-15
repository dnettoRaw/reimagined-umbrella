import Link from "next/link";

import { ClipboardCheck, ExternalLink } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { listServiceOrders } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function ExecutionPage() {
  const session = await requirePermission("inspection.execute");
  const [orders, { t }] = await Promise.all([
    listServiceOrders(session.role === "tecnico" ? { operator_id: session.sub } : {}),
    getI18n(),
  ]);
  const active = orders.items.filter((order) => order.status === "pending" || order.status === "in_progress");
  return (
    <div>
      <PageHeader title={t("nav.execution")} description={t("execution.description")} />
      {active.length ? (
        <div className="grid gap-3 lg:grid-cols-2">
          {active.map((order) => (
            <Card key={order.id}>
              <CardHeader>
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <CardTitle className="text-base">
                      {order.machine_snapshot.code} · {order.machine_snapshot.name}
                    </CardTitle>
                    <CardDescription>{order.description}</CardDescription>
                  </div>
                  <Badge variant={order.status === "in_progress" ? "default" : "secondary"}>
                    {order.status === "in_progress" ? t("common.inProgress") : t("common.pending")}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent className="flex items-end justify-between gap-3">
                <div className="text-sm">
                  <p>{t("orders.progress", { completed: order.completed_tasks, total: order.tasks.length })}</p>
                  <p className="text-muted-foreground">
                    {t("orders.maxLevel", { level: order.maximum_complexity_level })}
                  </p>
                </div>
                <Button asChild>
                  <Link href={`/dashboard/execution/${encodeURIComponent(order.id)}`}>
                    <ExternalLink />
                    {t("execution.openOrder")}
                  </Link>
                </Button>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <Card>
          <CardContent>
            <ProexelEmptyState
              icon={ClipboardCheck}
              title={t("execution.noOrders")}
              description={t("execution.noOrdersDescription")}
            />
          </CardContent>
        </Card>
      )}
    </div>
  );
}
