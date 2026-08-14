import Link from "next/link";

import { Activity, CalendarClock, PackageSearch, Wrench } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { getOverview } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function OverviewPage() {
  const overview = await getOverview();
  const { valves, orders, stock } = overview;
  const inDatePercent = valves.total > 0 ? Math.round((valves.ok / valves.total) * 100) : 0;

  return (
    <div>
      <PageHeader title="Visão geral" description="Condição operacional consolidada da planta." />
      <div className="mb-5 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          href="/dashboard/valves"
          title="Válvulas"
          value={valves.total}
          detail={`${inDatePercent}% em dia`}
          icon={Activity}
        />
        <MetricCard
          href="/dashboard/valves"
          title="Críticas"
          value={valves.critical}
          detail={`${valves.warning} em atenção`}
          icon={Wrench}
          tone="critical"
        />
        <MetricCard
          href="/dashboard/orders"
          title="OS abertas"
          value={orders.open}
          detail={`${orders.in_progress} em andamento`}
          icon={CalendarClock}
        />
        <MetricCard
          href="/dashboard/stock"
          title="Estoque baixo"
          value={stock.low}
          detail={`${stock.total} itens cadastrados`}
          icon={PackageSearch}
        />
      </div>
      <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
        <Card>
          <CardHeader>
            <CardTitle>Saúde da planta</CardTitle>
            <CardDescription>Calculada pela policy canônica de 150/180 dias.</CardDescription>
          </CardHeader>
          <CardContent>
            {valves.total === 0 ? (
              <ProexelEmptyState
                icon={Activity}
                title="Sem dados operacionais"
                description="Cadastre ou importe válvulas para iniciar o acompanhamento."
              />
            ) : (
              <div className="grid gap-3">
                <HealthRow label="Em dia" value={valves.ok} total={valves.total} className="bg-emerald-500" />
                <HealthRow label="Atenção" value={valves.warning} total={valves.total} className="bg-amber-500" />
                <HealthRow label="Crítica" value={valves.critical} total={valves.total} className="bg-red-500" />
              </div>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Serviço local</CardTitle>
            <CardDescription>Estado da integração com o AppCore.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground text-sm">Conexão</span>
              <Badge variant={overview.source === "appcore" ? "default" : "secondary"}>
                {overview.source === "appcore" ? "Conectado" : "Indisponível"}
              </Badge>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground text-sm">Schema</span>
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
