import { Plus, Wrench } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listMaintenance, listValves } from "@/lib/proexel/service";

import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function MaintenancePage() {
  const { role } = await requirePermission("maintenance.read");
  const [maintenance, valves] = await Promise.all([listMaintenance(), listValves()]);
  return (
    <div>
      <PageHeader
        title="Manutenção"
        description="Execuções registradas com atualização de condição e consumo idempotente de kit."
        action={
          can("maintenance.register", role) ? (
            <CommandDialog
              trigger={
                <Button disabled={valves.items.length === 0}>
                  <Plus />
                  Registrar manutenção
                </Button>
              }
              title="Registrar manutenção"
              description="A manutenção física será preservada mesmo quando o consumo do kit ficar pendente por falta de estoque."
              endpoint="/api/proexel/maintenance"
              fields={[
                {
                  name: "valve_id",
                  label: "Válvula",
                  type: "select",
                  required: true,
                  options: valves.items.map((valve) => ({ label: `${valve.tag} · ${valve.zone}`, value: valve.id })),
                },
                {
                  name: "performed_at",
                  label: "Data",
                  type: "date",
                  required: true,
                  defaultValue: new Date().toISOString().slice(0, 10),
                },
                { name: "technician", label: "Técnico", required: true },
                {
                  name: "maintenance_type",
                  label: "Tipo",
                  type: "select",
                  required: true,
                  options: [
                    { label: "Preventiva", value: "preventive" },
                    { label: "Corretiva", value: "corrective" },
                  ],
                },
                { name: "service", label: "Serviço executado", type: "textarea", required: true },
                { name: "notes", label: "Notas", type: "textarea" },
                { name: "signature_ref", label: "Referência da assinatura" },
                { name: "kit_changed", label: "Houve troca de kit", type: "checkbox" },
              ]}
            />
          ) : null
        }
      />
      <Card>
        <CardHeader>
          <CardTitle>Histórico de manutenção</CardTitle>
          <CardDescription>{maintenance.items.length} registro(s)</CardDescription>
        </CardHeader>
        <CardContent>
          {maintenance.items.length === 0 ? (
            <ProexelEmptyState
              icon={Wrench}
              title="Nenhuma manutenção registrada"
              description="Selecione uma válvula e registre a primeira execução."
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Data</TableHead>
                    <TableHead>TAG</TableHead>
                    <TableHead>Técnico</TableHead>
                    <TableHead>Tipo</TableHead>
                    <TableHead>Serviço</TableHead>
                    <TableHead>Kit</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {maintenance.items.map((item) => (
                    <TableRow key={item.id}>
                      <TableCell>{item.performed_at}</TableCell>
                      <TableCell className="font-medium">{item.valve_tag_snapshot}</TableCell>
                      <TableCell>{item.technician}</TableCell>
                      <TableCell>{item.maintenance_type === "preventive" ? "Preventiva" : "Corretiva"}</TableCell>
                      <TableCell className="max-w-80 whitespace-normal">{item.service}</TableCell>
                      <TableCell>
                        {!item.kit_changed ? (
                          <Badge variant="outline">Sem troca</Badge>
                        ) : item.stock_consumed ? (
                          <Badge>Consumido</Badge>
                        ) : (
                          <Badge variant="destructive">Consumo pendente</Badge>
                        )}
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
