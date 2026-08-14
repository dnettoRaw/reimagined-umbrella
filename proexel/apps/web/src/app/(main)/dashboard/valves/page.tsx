import { Activity, Plus, Search } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listValves } from "@/lib/proexel/service";

import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function ValvesPage({ searchParams }: { readonly searchParams: Promise<{ q?: string }> }) {
  const { q = "" } = await searchParams;
  const { role } = await requirePermission("valve.read");
  const valves = await listValves(q ? { search: q } : {});

  return (
    <div>
      <PageHeader
        title="Válvulas"
        description="Cadastro técnico e condição de manutenção da planta."
        action={
          can("valve.create", role) ? (
            <CommandDialog
              trigger={
                <Button>
                  <Plus />
                  Nova válvula
                </Button>
              }
              title="Cadastrar válvula"
              description="A TAG será normalizada e a referência de kit criará um item de estoque quando necessário."
              endpoint="/api/proexel/valves"
              fields={[
                { name: "tag", label: "TAG", required: true, placeholder: "FV 10.2" },
                { name: "zone", label: "Zona", required: true, placeholder: "Zona A" },
                { name: "manufacturer", label: "Fabricante" },
                { name: "serial", label: "Número de série" },
                { name: "valve_type", label: "Tipo" },
                { name: "dn", label: "DN" },
                { name: "kit_reference", label: "Referência do kit" },
                { name: "seat", label: "Assento" },
                { name: "actuator", label: "Atuador" },
                { name: "manufactured_at", label: "Fabricação", type: "date" },
              ]}
            />
          ) : null
        }
      />

      <Card>
        <CardHeader className="gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <CardTitle>Registro técnico</CardTitle>
            <CardDescription>
              {valves.items.length} resultado(s) · origem {valves.source}
            </CardDescription>
          </div>
          <form className="flex w-full gap-2 sm:w-80">
            <Input name="q" defaultValue={q} placeholder="Buscar TAG ou zona" aria-label="Buscar válvulas" />
            <Button type="submit" size="icon" variant="outline" title="Buscar">
              <Search />
            </Button>
          </form>
        </CardHeader>
        <CardContent>
          {valves.items.length === 0 ? (
            <ProexelEmptyState
              icon={Activity}
              title={q ? "Nenhuma válvula encontrada" : "Nenhuma válvula cadastrada"}
              description={q ? "Revise o termo de busca." : "Cadastre a primeira válvula para iniciar a operação."}
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>TAG</TableHead>
                    <TableHead>Zona</TableHead>
                    <TableHead>Fabricante</TableHead>
                    <TableHead>Tipo / DN</TableHead>
                    <TableHead>Kit</TableHead>
                    <TableHead>Última manutenção</TableHead>
                    <TableHead>Condição</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {valves.items.map((valve) => (
                    <TableRow key={valve.id}>
                      <TableCell className="font-medium">{valve.tag}</TableCell>
                      <TableCell>{valve.zone}</TableCell>
                      <TableCell>{valve.manufacturer ?? "-"}</TableCell>
                      <TableCell>{[valve.valve_type, valve.dn].filter(Boolean).join(" / ") || "-"}</TableCell>
                      <TableCell>{valve.kit_reference ?? "-"}</TableCell>
                      <TableCell>{valve.last_maintenance_at ?? "Nunca"}</TableCell>
                      <TableCell>
                        <HealthBadge health={valve.health} />
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

function HealthBadge({ health }: { readonly health: "ok" | "warning" | "critical" }) {
  const label = health === "ok" ? "Em dia" : health === "warning" ? "Atenção" : "Crítica";
  return (
    <Badge variant={health === "critical" ? "destructive" : health === "warning" ? "secondary" : "outline"}>
      {label}
    </Badge>
  );
}
