import { PackageSearch, Plus } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listStock } from "@/lib/proexel/service";

import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function StockPage() {
  const { role } = await requirePermission("stock.read");
  const stock = await listStock();
  return (
    <div>
      <PageHeader
        title="Estoque"
        description="Saldos, mínimos e rastreabilidade de ajustes."
        action={
          can("stock.add_or_increment", role) ? (
            <CommandDialog
              trigger={
                <Button>
                  <Plus />
                  Novo item
                </Button>
              }
              title="Cadastrar item"
              description="A referência é única e normalizada."
              endpoint="/api/proexel/stock"
              fields={[
                { name: "reference", label: "Referência", required: true },
                {
                  name: "minimum_quantity",
                  label: "Quantidade mínima",
                  type: "number",
                  required: true,
                  defaultValue: 0,
                },
                { name: "manufacturer", label: "Fabricante" },
                { name: "location", label: "Localização" },
              ]}
            />
          ) : null
        }
      />
      <Card>
        <CardHeader>
          <CardTitle>Itens de estoque</CardTitle>
          <CardDescription>
            {stock.items.filter((item) => item.quantity <= item.minimum_quantity).length} abaixo ou no mínimo
          </CardDescription>
        </CardHeader>
        <CardContent>
          {stock.items.length === 0 ? (
            <ProexelEmptyState
              icon={PackageSearch}
              title="Estoque vazio"
              description="Itens de kit criados por válvulas também aparecem aqui."
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Referência</TableHead>
                    <TableHead>Fabricante</TableHead>
                    <TableHead>Localização</TableHead>
                    <TableHead>Saldo</TableHead>
                    <TableHead>Mínimo</TableHead>
                    <TableHead>Condição</TableHead>
                    <TableHead className="text-right">Ajustar</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {stock.items.map((item) => (
                    <TableRow key={item.id}>
                      <TableCell className="font-medium">{item.reference}</TableCell>
                      <TableCell>{item.manufacturer ?? "-"}</TableCell>
                      <TableCell>{item.location ?? "-"}</TableCell>
                      <TableCell>{item.quantity}</TableCell>
                      <TableCell>{item.minimum_quantity}</TableCell>
                      <TableCell>
                        <Badge variant={item.quantity <= item.minimum_quantity ? "destructive" : "outline"}>
                          {item.quantity <= item.minimum_quantity ? "Repor" : "Normal"}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-right">
                        {can("stock.adjust_quantity", role) ? (
                          <CommandDialog
                            trigger={
                              <Button size="sm" variant="outline">
                                Ajustar
                              </Button>
                            }
                            title={`Ajustar ${item.reference}`}
                            description={`Saldo atual: ${item.quantity}. Use valor negativo para saída.`}
                            endpoint="/api/proexel/stock"
                            method="PATCH"
                            fields={[
                              { name: "id", label: "ID", type: "hidden", defaultValue: item.id, required: true },
                              { name: "delta", label: "Variação", type: "number", required: true },
                              { name: "reason", label: "Motivo", type: "textarea", required: true },
                            ]}
                          />
                        ) : null}
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
