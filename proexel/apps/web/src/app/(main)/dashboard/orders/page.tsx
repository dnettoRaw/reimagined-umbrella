import { CalendarClock, Plus } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listServiceOrders, listValves } from "@/lib/proexel/service";

import { CommandButton } from "../_components/command-button";
import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function OrdersPage() {
  const { role } = await requirePermission("order.read");
  const [orders, valves] = await Promise.all([listServiceOrders(), listValves()]);
  return (
    <div>
      <PageHeader
        title="Ordens de serviço"
        description="Fila de trabalho e programação operacional."
        action={
          can("order.create", role) ? (
            <CommandDialog
              trigger={
                <Button>
                  <Plus />
                  Nova OS
                </Button>
              }
              title="Criar ordem de serviço"
              description="A ordem inicia no estado pendente."
              endpoint="/api/proexel/orders"
              fields={[
                { name: "zone", label: "Zona", required: true },
                {
                  name: "valve_id",
                  label: "Válvula (opcional)",
                  type: "select",
                  options: valves.items.map((valve) => ({ label: valve.tag, value: valve.id })),
                },
                {
                  name: "priority",
                  label: "Prioridade",
                  type: "select",
                  required: true,
                  options: [
                    { label: "Baixa", value: "low" },
                    { label: "Normal", value: "normal" },
                    { label: "Alta", value: "high" },
                    { label: "Urgente", value: "urgent" },
                  ],
                  defaultValue: "normal",
                },
                { name: "technician", label: "Técnico responsável" },
                { name: "scheduled_for", label: "Data programada", type: "date" },
                { name: "description", label: "Descrição", type: "textarea", required: true },
              ]}
            />
          ) : null
        }
      />
      <Card>
        <CardHeader>
          <CardTitle>Agenda operacional</CardTitle>
          <CardDescription>{orders.items.length} ordem(ns)</CardDescription>
        </CardHeader>
        <CardContent>
          {orders.items.length === 0 ? (
            <ProexelEmptyState
              icon={CalendarClock}
              title="Nenhuma ordem aberta"
              description="Crie uma OS para planejar o próximo trabalho."
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Zona / TAG</TableHead>
                    <TableHead>Descrição</TableHead>
                    <TableHead>Prioridade</TableHead>
                    <TableHead>Programada</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead className="text-right">Ações</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {orders.items.map((order) => (
                    <TableRow key={order.id}>
                      <TableCell>
                        <div className="font-medium">{order.zone}</div>
                        <div className="text-muted-foreground text-xs">{order.valve_tag_snapshot ?? "Sem válvula"}</div>
                      </TableCell>
                      <TableCell className="max-w-80 whitespace-normal">{order.description}</TableCell>
                      <TableCell>{order.priority}</TableCell>
                      <TableCell>{order.scheduled_for ?? "-"}</TableCell>
                      <TableCell>
                        <OrderStatus status={order.status} />
                      </TableCell>
                      <TableCell>
                        <div className="flex justify-end gap-2">
                          {can("order.change_status", role) && order.status === "pending" ? (
                            <CommandButton
                              endpoint="/api/proexel/orders"
                              data={{ id: order.id, status: "in_progress" }}
                            >
                              Iniciar
                            </CommandButton>
                          ) : null}
                          {can("order.change_status", role) && order.status !== "completed" ? (
                            <CommandButton endpoint="/api/proexel/orders" data={{ id: order.id, status: "completed" }}>
                              Concluir
                            </CommandButton>
                          ) : null}
                        </div>
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

function OrderStatus({ status }: { readonly status: "pending" | "in_progress" | "completed" }) {
  const labels = { pending: "Pendente", in_progress: "Em andamento", completed: "Concluída" };
  return (
    <Badge variant={status === "completed" ? "outline" : status === "in_progress" ? "default" : "secondary"}>
      {labels[status]}
    </Badge>
  );
}
