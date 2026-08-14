import { BarChart3 } from "lucide-react";

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { requirePermission } from "@/lib/proexel/auth-server";
import { getReports } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";

export const dynamic = "force-dynamic";

export default async function ReportsPage() {
  await requirePermission("report.read");
  const report = await getReports();
  const byZone = report.by_zone.toSorted((a, b) => b.critical - a.critical);
  return (
    <div>
      <PageHeader title="Relatórios" description="Resumo geral, criticidade por zona e manutenções recentes." />
      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Resumo geral</CardTitle>
            <CardDescription>Indicadores canônicos atuais.</CardDescription>
          </CardHeader>
          <CardContent className="grid grid-cols-2 gap-4">
            <ReportMetric label="Válvulas" value={report.overview.valves.total} />
            <ReportMetric label="Críticas" value={report.overview.valves.critical} />
            <ReportMetric label="OS abertas" value={report.overview.orders.open} />
            <ReportMetric label="Estoque baixo" value={report.overview.stock.low} />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Criticidade por zona</CardTitle>
            <CardDescription>Zonas ordenadas por válvulas críticas.</CardDescription>
          </CardHeader>
          <CardContent>
            {byZone.length === 0 ? (
              <div className="flex min-h-32 items-center justify-center text-muted-foreground">
                <BarChart3 className="mr-2 size-5" />
                Sem dados
              </div>
            ) : (
              <div className="space-y-3">
                {byZone.map((row) => (
                  <div key={row.zone} className="flex items-center justify-between border-b pb-2 text-sm">
                    <span>{row.zone}</span>
                    <span>
                      <strong>{row.critical}</strong> críticas / {row.total}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>Manutenções recentes</CardTitle>
            <CardDescription>Últimos registros disponíveis.</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="divide-y">
              {report.recent_maintenance.slice(0, 10).map((item) => (
                <div key={item.id} className="grid gap-1 py-3 sm:grid-cols-[140px_140px_1fr]">
                  <span>{item.performed_at}</span>
                  <strong>{item.valve_tag_snapshot}</strong>
                  <span>{item.service}</span>
                </div>
              ))}
              {report.recent_maintenance.length === 0 ? (
                <p className="text-muted-foreground text-sm">Nenhuma manutenção registrada.</p>
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
