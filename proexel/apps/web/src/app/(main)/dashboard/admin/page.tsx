import { Check, Minus, ShieldCheck } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { requirePermission } from "@/lib/proexel/auth-server";

import { PageHeader } from "../_components/page-header";

const rows = [
  ["Consultar válvulas", true, true, false, true],
  ["Cadastrar válvulas", true, true, false, false],
  ["Alterar dados técnicos", true, false, false, false],
  ["Registrar manutenção", true, true, false, true],
  ["Gerenciar ordens", true, true, false, false],
  ["Ajustar estoque", true, true, true, false],
  ["Revisar reposição", true, true, false, false],
  ["Gerenciar fornecedores", true, false, false, false],
  ["Consultar auditoria", true, true, false, false],
] as const;
const roleNames = ["admin", "chefe", "compras", "tecnico"] as const;

export default async function AdminPage() {
  await requirePermission("admin.manage");
  return (
    <div>
      <PageHeader title="Administração" description="Matriz de autorização aplicada pela camada de aplicação." />
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <ShieldCheck className="size-5" />
            <CardTitle>Permissões por papel</CardTitle>
          </div>
          <CardDescription>
            Os controles da interface refletem esta matriz, mas o backend permanece como autoridade.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Permissão</TableHead>
                  <TableHead>Admin</TableHead>
                  <TableHead>Chefe</TableHead>
                  <TableHead>Compras</TableHead>
                  <TableHead>Técnico</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map(([name, ...roles]) => (
                  <TableRow key={name}>
                    <TableCell className="font-medium">{name}</TableCell>
                    {roles.map((allowed, index) => (
                      <TableCell key={`${name}-${roleNames[index]}`}>
                        <Badge variant={allowed ? "outline" : "secondary"} className="gap-1">
                          {allowed ? <Check /> : <Minus />}
                          {allowed ? "Permitido" : "Negado"}
                        </Badge>
                      </TableCell>
                    ))}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
