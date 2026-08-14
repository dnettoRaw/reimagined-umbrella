import { Check, Plus, ShoppingCart, X } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { requireSession } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listRestockRequests } from "@/lib/proexel/service";

import { CommandButton } from "../_components/command-button";
import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function PurchasingPage() {
  const { role } = await requireSession();
  const requests = await listRestockRequests();
  return (
    <div>
      <PageHeader
        title="Compras e reposição"
        description="Solicitações de reposição e decisão registrada por revisor."
        action={
          can("restock.create_suggestion", role) ? (
            <CommandDialog
              trigger={
                <Button>
                  <Plus />
                  Solicitar reposição
                </Button>
              }
              title="Solicitar reposição"
              description="A criação exige a permissão operacional de técnico."
              endpoint="/api/proexel/purchasing"
              fields={[
                { name: "reference", label: "Referência", required: true },
                { name: "reason", label: "Motivo", type: "textarea", required: true },
              ]}
            />
          ) : null
        }
      />
      <Card>
        <CardHeader>
          <CardTitle>Solicitações</CardTitle>
          <CardDescription>
            {requests.items.filter((item) => item.status === "pending").length} pendente(s)
          </CardDescription>
        </CardHeader>
        <CardContent>
          {requests.items.length === 0 ? (
            <ProexelEmptyState
              icon={ShoppingCart}
              title="Nenhuma solicitação"
              description="As sugestões de reposição aparecerão nesta fila."
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Referência</TableHead>
                    <TableHead>Motivo</TableHead>
                    <TableHead>Solicitante</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Revisor</TableHead>
                    <TableHead className="text-right">Decisão</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {requests.items.map((item) => (
                    <TableRow key={item.id}>
                      <TableCell className="font-medium">{item.reference}</TableCell>
                      <TableCell className="max-w-80 whitespace-normal">{item.reason}</TableCell>
                      <TableCell>{item.requested_by}</TableCell>
                      <TableCell>
                        <RequestStatus status={item.status} />
                      </TableCell>
                      <TableCell>{item.reviewed_by ?? "-"}</TableCell>
                      <TableCell>
                        <div className="flex justify-end gap-2">
                          {can("restock.approve_reject", role) && item.status === "pending" ? (
                            <>
                              <CommandButton
                                endpoint="/api/proexel/purchasing"
                                data={{ id: item.id, status: "approved" }}
                              >
                                <Check />
                                Aprovar
                              </CommandButton>
                              <CommandButton
                                endpoint="/api/proexel/purchasing"
                                data={{ id: item.id, status: "rejected" }}
                                variant="ghost"
                              >
                                <X />
                                Rejeitar
                              </CommandButton>
                            </>
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

function RequestStatus({ status }: { readonly status: "pending" | "approved" | "rejected" }) {
  const labels = { pending: "Pendente", approved: "Aprovada", rejected: "Rejeitada" };
  return (
    <Badge variant={status === "approved" ? "default" : status === "rejected" ? "destructive" : "secondary"}>
      {labels[status]}
    </Badge>
  );
}
