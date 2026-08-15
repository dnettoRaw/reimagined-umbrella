import Link from "next/link";

import { CalendarClock, Factory, PackageSearch, Wrench } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { getI18n } from "@/lib/i18n/server";
import { getOverview } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function OverviewPage() {
  const [overview, { t }] = await Promise.all([getOverview(), getI18n()]);
  const criticalItems =
    (overview.machine_items.by_status.critical ?? 0) + (overview.machine_items.by_status.maintenance_required ?? 0);
  const attentionItems = overview.machine_items.by_status.attention ?? 0;
  const openOrders = overview.orders.pending + overview.orders.in_progress;
  return (
    <div>
      <PageHeader title={t("overview.title")} description={t("overview.description")} />
      <div className="mb-5 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <Metric
          href="/dashboard/machines"
          title={t("nav.machines")}
          value={overview.machines.total}
          detail={t("overview.componentCount", { count: overview.machine_items.total })}
          icon={Factory}
        />
        <Metric
          href="/dashboard/machines"
          title={t("overview.critical")}
          value={criticalItems}
          detail={t("overview.warningCount", { count: attentionItems })}
          icon={Wrench}
          critical
        />
        <Metric
          href="/dashboard/orders"
          title={t("overview.openOrders")}
          value={openOrders}
          detail={t("overview.inProgressCount", { count: overview.orders.in_progress })}
          icon={CalendarClock}
        />
        <Metric
          href="/dashboard/stock"
          title={t("overview.lowStock")}
          value={overview.stock.low}
          detail={t("overview.stockItems", { count: overview.stock.total })}
          icon={PackageSearch}
        />
      </div>
      <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
        <Card>
          <CardHeader>
            <CardTitle>{t("overview.plantHealth")}</CardTitle>
            <CardDescription>{t("overview.deterministicPolicy")}</CardDescription>
          </CardHeader>
          <CardContent>
            {overview.machine_items.total ? (
              <div className="grid gap-3">
                {(
                  [
                    "ok",
                    "attention",
                    "critical",
                    "maintenance_required",
                    "under_maintenance",
                    "unknown",
                    "disabled",
                  ] as const
                ).map((status) => (
                  <HealthRow
                    key={status}
                    label={t(`status.${status}`)}
                    value={overview.machine_items.by_status[status] ?? 0}
                    total={overview.machine_items.total}
                    status={status}
                  />
                ))}
              </div>
            ) : (
              <ProexelEmptyState
                icon={Factory}
                title={t("overview.noData")}
                description={t("overview.noMachineDataDescription")}
              />
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>{t("overview.localService")}</CardTitle>
            <CardDescription>{t("overview.integration")}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground text-sm">{t("overview.connection")}</span>
              <Badge variant={overview.source === "appcore" ? "default" : "secondary"}>
                {overview.source === "appcore" ? t("common.connected") : t("common.unavailable")}
              </Badge>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground text-sm">{t("overview.schema")}</span>
              <strong className="text-sm">v{overview.schema_version}</strong>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function Metric({
  href,
  title,
  value,
  detail,
  icon: Icon,
  critical,
}: {
  href: string;
  title: string;
  value: number;
  detail: string;
  icon: typeof Factory;
  critical?: boolean;
}) {
  return (
    <Link href={href} className="block rounded-lg outline-none focus-visible:ring-2">
      <Card className="h-full transition-colors hover:bg-muted/30">
        <CardHeader className="flex-row items-center justify-between">
          <div>
            <CardDescription>{title}</CardDescription>
            <CardTitle className={critical ? "text-destructive" : undefined}>{value}</CardTitle>
          </div>
          <Icon className="size-5 text-muted-foreground" />
        </CardHeader>
        <CardContent className="text-muted-foreground text-sm">{detail}</CardContent>
      </Card>
    </Link>
  );
}

function HealthRow({ label, value, total, status }: { label: string; value: number; total: number; status: string }) {
  const color =
    status === "critical" || status === "maintenance_required"
      ? "bg-red-500"
      : status === "attention"
        ? "bg-amber-500"
        : status === "ok"
          ? "bg-emerald-500"
          : "bg-zinc-400";
  return (
    <div className="grid gap-1">
      <div className="flex justify-between text-sm">
        <span>{label}</span>
        <span className="text-muted-foreground">{value}</span>
      </div>
      <div className="h-2 overflow-hidden rounded-md bg-muted">
        <div className={`h-full ${color}`} style={{ width: `${total ? (value / total) * 100 : 0}%` }} />
      </div>
    </div>
  );
}
