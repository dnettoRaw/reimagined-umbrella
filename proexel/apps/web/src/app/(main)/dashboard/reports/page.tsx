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
  const byZone = report.by_zone.toSorted((left, right) => right.critical_items - left.critical_items);
  const critical =
    (report.overview.machine_items.by_status.critical ?? 0) +
    (report.overview.machine_items.by_status.maintenance_required ?? 0);
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
            <Metric label={t("nav.machines")} value={report.overview.machines.total} />
            <Metric label={t("common.components")} value={report.overview.machine_items.total} />
            <Metric label={t("overview.critical")} value={critical} />
            <Metric label={t("overview.lowStock")} value={report.overview.stock.low} />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>{t("reports.byZone")}</CardTitle>
            <CardDescription>{t("reports.zoneComposition")}</CardDescription>
          </CardHeader>
          <CardContent>
            {byZone.length ? (
              <div className="space-y-3">
                {byZone.map((row) => (
                  <div key={row.zone} className="grid grid-cols-[1fr_auto_auto] gap-4 border-b pb-2 text-sm">
                    <span>{row.zone}</span>
                    <span>
                      {row.items} {t("common.components")}
                    </span>
                    <strong className={row.critical_items ? "text-destructive" : ""}>
                      {row.critical_items} {t("common.critical")}
                    </strong>
                  </div>
                ))}
              </div>
            ) : (
              <div className="flex min-h-32 items-center justify-center text-muted-foreground">
                <BarChart3 className="mr-2 size-5" />
                {t("common.noData")}
              </div>
            )}
          </CardContent>
        </Card>
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>{t("reports.recentInspections")}</CardTitle>
            <CardDescription>{t("reports.latest")}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="divide-y">
              {report.recent_inspections.slice(0, 10).map((inspection) => (
                <div key={inspection.id} className="grid gap-1 py-3 sm:grid-cols-[180px_180px_1fr_auto]">
                  <span>
                    {new Intl.DateTimeFormat(INTL_LOCALES[locale], { dateStyle: "short", timeStyle: "short" }).format(
                      new Date(inspection.completed_at_ms ?? inspection.started_at_ms),
                    )}
                  </span>
                  <strong>{inspection.operator_name}</strong>
                  <span>{inspection.maintenance_action ?? inspection.notes ?? "-"}</span>
                  <span>{t(`status.${inspection.status_after ?? inspection.status_before}`)}</span>
                </div>
              ))}
              {report.recent_inspections.length === 0 ? (
                <p className="text-muted-foreground text-sm">{t("reports.noInspections")}</p>
              ) : null}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <div>
      <div className="text-muted-foreground text-sm">{label}</div>
      <div className="font-heading font-semibold text-2xl">{value}</div>
    </div>
  );
}
