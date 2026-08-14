import { Building2, Plus } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listSuppliers } from "@/lib/proexel/service";

import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function SuppliersPage() {
  const { role } = await requirePermission("supplier.read");
  const suppliers = await listSuppliers();
  return (
    <div>
      <PageHeader
        title="Fornecedores"
        description="Contatos comerciais e referências de fornecimento."
        action={
          can("supplier.create_update_delete", role) ? (
            <CommandDialog
              trigger={
                <Button>
                  <Plus />
                  Novo fornecedor
                </Button>
              }
              title="Cadastrar fornecedor"
              description="Nome e contato são obrigatórios."
              endpoint="/api/proexel/suppliers"
              fields={[
                { name: "name", label: "Nome", required: true },
                { name: "contact", label: "Contato", required: true },
                { name: "email", label: "Email", type: "email" },
                { name: "website", label: "Website", type: "url" },
                { name: "notes", label: "Notas", type: "textarea" },
              ]}
            />
          ) : null
        }
      />
      <Card>
        <CardHeader>
          <CardTitle>Cadastro de fornecedores</CardTitle>
          <CardDescription>{suppliers.items.length} fornecedor(es)</CardDescription>
        </CardHeader>
        <CardContent>
          {suppliers.items.length === 0 ? (
            <ProexelEmptyState
              icon={Building2}
              title="Nenhum fornecedor"
              description="Cadastre contatos para apoiar o processo de compras."
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Nome</TableHead>
                    <TableHead>Contato</TableHead>
                    <TableHead>Email</TableHead>
                    <TableHead>Website</TableHead>
                    <TableHead>Notas</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {suppliers.items.map((supplier) => (
                    <TableRow key={supplier.id}>
                      <TableCell className="font-medium">{supplier.name}</TableCell>
                      <TableCell>{supplier.contact}</TableCell>
                      <TableCell>{supplier.email ?? "-"}</TableCell>
                      <TableCell>
                        {supplier.website ? (
                          <a
                            className="underline underline-offset-4"
                            href={supplier.website}
                            target="_blank"
                            rel="noreferrer"
                          >
                            Abrir
                          </a>
                        ) : (
                          "-"
                        )}
                      </TableCell>
                      <TableCell className="max-w-80 whitespace-normal">{supplier.notes ?? "-"}</TableCell>
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
