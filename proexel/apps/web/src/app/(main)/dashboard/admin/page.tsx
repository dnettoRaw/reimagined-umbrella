import { Check, Minus, ShieldCheck } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { TranslationKey } from "@/lib/i18n/messages";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";

import { PageHeader } from "../_components/page-header";

const rows: readonly [TranslationKey, boolean, boolean, boolean, boolean][] = [
  ["admin.readValves", true, true, false, true],
  ["admin.createValves", true, true, false, false],
  ["admin.editTechnical", true, false, false, false],
  ["admin.registerMaintenance", true, true, false, true],
  ["admin.manageOrders", true, true, false, false],
  ["admin.adjustStock", true, true, true, false],
  ["admin.reviewRestock", true, true, false, false],
  ["admin.manageSuppliers", true, false, false, false],
  ["admin.readAudit", true, true, false, false],
];
const roleNames = ["admin", "chefe", "compras", "tecnico"] as const;

export default async function AdminPage() {
  const [, { t }] = await Promise.all([requirePermission("admin.manage"), getI18n()]);
  return (
    <div>
      <PageHeader title={t("nav.admin")} description={t("admin.description")} />
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <ShieldCheck className="size-5" />
            <CardTitle>{t("admin.permissionsByRole")}</CardTitle>
          </div>
          <CardDescription>{t("admin.backendAuthority")}</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t("common.permission")}</TableHead>
                  {roleNames.map((role) => (
                    <TableHead key={role}>{t(`role.${role}`)}</TableHead>
                  ))}
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map(([name, ...roles]) => (
                  <TableRow key={name}>
                    <TableCell className="font-medium">{t(name)}</TableCell>
                    {roles.map((allowed, index) => (
                      <TableCell key={`${name}-${roleNames[index]}`}>
                        <Badge variant={allowed ? "outline" : "secondary"} className="gap-1">
                          {allowed ? <Check /> : <Minus />}
                          {allowed ? t("admin.allowed") : t("admin.denied")}
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
