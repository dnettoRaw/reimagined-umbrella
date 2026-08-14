import { History } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { requirePermission } from "@/lib/proexel/auth-server";
import { listAudit } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function AuditPage() {
  await requirePermission("audit.read");
  const audit = await listAudit();
  return (
    <div>
      <PageHeader title="Histórico e auditoria" description="Trilha confiável das operações de escrita do domínio." />
      <Card>
        <CardHeader>
          <CardTitle>Eventos recentes</CardTitle>
          <CardDescription>Até 250 eventos, do mais recente para o mais antigo.</CardDescription>
        </CardHeader>
        <CardContent>
          {audit.items.length === 0 ? (
            <ProexelEmptyState
              icon={History}
              title="Sem eventos"
              description="A trilha será criada automaticamente ao executar operações."
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Data</TableHead>
                    <TableHead>Ator</TableHead>
                    <TableHead>Operação</TableHead>
                    <TableHead>Entidade</TableHead>
                    <TableHead>Descrição</TableHead>
                    <TableHead>Resultado</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {audit.items.map((event) => (
                    <TableRow key={event.id}>
                      <TableCell>{new Date(event.created_at_ms).toLocaleString("pt-BR")}</TableCell>
                      <TableCell>
                        <div className="font-medium">{event.actor}</div>
                        <div className="text-muted-foreground text-xs">{event.role}</div>
                      </TableCell>
                      <TableCell className="font-mono text-xs">{event.operation}</TableCell>
                      <TableCell>{event.aggregate}</TableCell>
                      <TableCell>{event.description ?? "-"}</TableCell>
                      <TableCell>
                        <Badge variant={event.result === "success" ? "outline" : "destructive"}>{event.result}</Badge>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
