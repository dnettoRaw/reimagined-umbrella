import Link from "next/link";

import { Activity, CalendarClock, PackageSearch, Wrench } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { getI18n } from "@/lib/i18n/server";
import { getOverview } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function OverviewPage() {
  const [overview, { t }] = await Promise.all([getOverview(), getI18n()]);
  const { valves, orders, stock } = overview;
  const inDatePercent = valves.total > 0 ? Math.round((valves.ok / valves.total) * 100) : 0;

  return (
    <div>
      <PageHeader title={t("overview.title")} description={t("overview.description")} />
      <div className="mb-5 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          href="/dashboard/valves"
          title={t("overview.valves")}
          value={valves.total}
          detail={t("overview.onTime", { percent: inDatePercent })}
          icon={Activity}
        />
        <MetricCard
          href="/dashboard/valves"
          title={t("overview.critical")}
          value={valves.critical}
          detail={t("overview.warningCount", { count: valves.warning })}
          icon={Wrench}
          tone="critical"
        />
        <MetricCard
          href="/dashboard/orders"
          title={t("overview.openOrders")}
          value={orders.open}
          detail={t("overview.inProgressCount", { count: orders.in_progress })}
          icon={CalendarClock}
        />
        <MetricCard
          href="/dashboard/stock"
          title={t("overview.lowStock")}
          value={stock.low}
          detail={t("overview.stockItems", { count: stock.total })}
          icon={PackageSearch}
        />
      </div>
      <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
        <Card>
          <CardHeader>
            <CardTitle>{t("overview.plantHealth")}</CardTitle>
            <CardDescription>{t("overview.policy")}</CardDescription>
          </CardHeader>
          <CardContent>
            {valves.total === 0 ? (
              <ProexelEmptyState
                icon={Activity}
                title={t("overview.noData")}
                description={t("overview.noDataDescription")}
              />
            ) : (
              <div className="grid gap-3">
                <HealthRow
                  label={t("overview.onTrack")}
                  value={valves.ok}
                  total={valves.total}
                  className="bg-emerald-500"
                />
                <HealthRow
                  label={t("common.warning")}
                  value={valves.warning}
                  total={valves.total}
                  className="bg-amber-500"
                />
                <HealthRow
                  label={t("common.critical")}
                  value={valves.critical}
                  total={valves.total}
                  className="bg-red-500"
                />
              </div>
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
              <span className="font-medium text-sm">v{overview.schema_version}</span>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function MetricCard({
  href,
  title,
  value,
  detail,
  icon: Icon,
  tone,
}: {
  readonly href: string;
  readonly title: string;
  readonly value: number;
  readonly detail: string;
  readonly icon: typeof Activity;
  readonly tone?: "critical";
}) {
  return (
    <Link href={href} className="block rounded-lg outline-none focus-visible:ring-2">
      <Card className="h-full transition-colors hover:bg-muted/30">
        <CardHeader className="flex-row items-center justify-between space-y-0">
          <div>
            <CardDescription>{title}</CardDescription>
            <CardTitle className={tone === "critical" ? "text-destructive" : undefined}>{value}</CardTitle>
          </div>
          <Icon className="size-5 text-muted-foreground" />
        </CardHeader>
        <CardContent className="text-muted-foreground text-sm">{detail}</CardContent>
      </Card>
    </Link>
  );
}

function HealthRow({
  label,
  value,
  total,
  className,
}: {
  readonly label: string;
  readonly value: number;
  readonly total: number;
  readonly className: string;
}) {
  const width = total > 0 ? Math.round((value / total) * 100) : 0;
  return (
    <div className="grid gap-1">
      <div className="flex items-center justify-between text-sm">
        <span>{label}</span>
        <span className="text-muted-foreground">{value}</span>
      </div>
      <div className="h-2 overflow-hidden rounded-md bg-muted">
        <div className={`h-full ${className}`} style={{ width: `${width}%` }} />
      </div>
    </div>
  );
}
