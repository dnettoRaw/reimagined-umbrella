import { CircleCheck, CircleX, Server } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { getRuntimeStatus } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";

export const dynamic = "force-dynamic";

export default async function SettingsPage() {
  const runtime = await getRuntimeStatus();
  const StatusIcon = runtime.healthy ? CircleCheck : CircleX;
  return (
    <div>
      <PageHeader title="Configurações" description="Estado da instalação local e limites operacionais." />
      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Server className="size-5" />
              <CardTitle>Runtime AppCore</CardTitle>
            </div>
            <CardDescription>Conectividade do serviço local configurado no servidor web.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground text-sm">Estado</span>
              <Badge variant={runtime.healthy ? "outline" : "destructive"} className="gap-1">
                <StatusIcon />
                {runtime.healthy ? "Saudável" : runtime.configured ? "Indisponível" : "Não configurado"}
              </Badge>
            </div>
            <div className="flex items-center justify-between gap-4">
              <span className="text-muted-foreground text-sm">Endpoint</span>
              <code className="truncate text-xs">{runtime.url ?? "-"}</code>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Políticas operacionais</CardTitle>
            <CardDescription>Valores consolidados no domínio.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            <div className="flex justify-between">
              <span>Manutenção em atenção</span>
              <strong>&gt; 150 dias</strong>
            </div>
            <div className="flex justify-between">
              <span>Manutenção crítica</span>
              <strong>&gt; 180 dias</strong>
            </div>
            <div className="flex justify-between">
              <span>Estoque negativo</span>
              <strong>Bloqueado</strong>
            </div>
            <div className="flex justify-between">
              <span>Schema local</span>
              <strong>v1</strong>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
