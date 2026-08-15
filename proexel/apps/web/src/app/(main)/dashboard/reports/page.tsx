import { BarChart3 } from "lucide-react";

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { INTL_LOCALES } from "@/lib/i18n/config";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { getReports } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";
import { ReportExport } from "./report-export";

export const dynamic = "force-dynamic";

export default async function ReportsPage() {
  const [, report, { t, locale }] = await Promise.all([requirePermission("report.read"), getReports(), getI18n()]);
  const byZone = report.by_zone.toSorted((a, b) => b.critical - a.critical);
  return (
    <div>
      <PageHeader
        title={t("nav.reports")}
        description={t("reports.description")}
        action={<ReportExport report={report} />}
      />
      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>{t("reports.summary")}</CardTitle>
            <CardDescription>{t("reports.currentIndicators")}</CardDescription>
          </CardHeader>
          <CardContent className="grid grid-cols-2 gap-4">
            <ReportMetric label={t("overview.valves")} value={report.overview.valves.total} />
            <ReportMetric label={t("overview.critical")} value={report.overview.valves.critical} />
            <ReportMetric label={t("overview.openOrders")} value={report.overview.orders.open} />
            <ReportMetric label={t("overview.lowStock")} value={report.overview.stock.low} />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>{t("reports.byZone")}</CardTitle>
            <CardDescription>{t("reports.byZoneDescription")}</CardDescription>
          </CardHeader>
          <CardContent>
            {byZone.length === 0 ? (
              <div className="flex min-h-32 items-center justify-center text-muted-foreground">
                <BarChart3 className="mr-2 size-5" />
                {t("common.noData")}
              </div>
            ) : (
              <div className="space-y-3">
                {byZone.map((row) => (
                  <div key={row.zone} className="flex items-center justify-between border-b pb-2 text-sm">
                    <span>{row.zone}</span>
                    <span>{t("reports.criticalCount", { critical: row.critical, total: row.total })}</span>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>{t("reports.recentMaintenance")}</CardTitle>
            <CardDescription>{t("reports.latest")}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="divide-y">
              {report.recent_maintenance.slice(0, 10).map((item) => (
                <div key={item.id} className="grid gap-1 py-3 sm:grid-cols-[140px_140px_1fr]">
                  <span>
                    {new Intl.DateTimeFormat(INTL_LOCALES[locale]).format(new Date(`${item.performed_at}T00:00:00`))}
                  </span>
                  <strong>{item.valve_tag_snapshot}</strong>
                  <span>{item.service}</span>
                </div>
              ))}
              {report.recent_maintenance.length === 0 ? (
                <p className="text-muted-foreground text-sm">{t("reports.noMaintenance")}</p>
              ) : null}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function ReportMetric({ label, value }: { readonly label: string; readonly value: number }) {
  return (
    <div>
      <div className="text-muted-foreground text-sm">{label}</div>
      <div className="font-heading font-semibold text-2xl">{value}</div>
    </div>
  );
}
