import { PackageSearch, Plus, Trash2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listStock } from "@/lib/proexel/service";

import { CommandButton } from "../_components/command-button";
import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function StockPage() {
  const [{ role }, stock, { t }] = await Promise.all([requirePermission("stock.read"), listStock(), getI18n()]);
  return (
    <div>
      <PageHeader
        title={t("nav.stock")}
        description={t("stock.description")}
        action={
          can("stock.add_or_increment", role) ? (
            <CommandDialog
              trigger={
                <Button>
                  <Plus />
                  {t("stock.new")}
                </Button>
              }
              title={t("stock.create")}
              description={t("stock.createDescription")}
              endpoint="/api/proexel/stock"
              fields={[
                { name: "reference", label: t("stock.reference"), required: true },
                {
                  name: "minimum_quantity",
                  label: t("stock.minimumQuantity"),
                  type: "number",
                  required: true,
                  defaultValue: 0,
                },
                { name: "manufacturer", label: t("common.manufacturer") },
                { name: "location", label: t("stock.location") },
              ]}
            />
          ) : null
        }
      />
      <Card>
        <CardHeader>
          <CardTitle>{t("stock.items")}</CardTitle>
          <CardDescription>
            {t("stock.belowMinimum", {
              count: stock.items.filter((item) => item.quantity <= item.minimum_quantity).length,
            })}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {stock.items.length === 0 ? (
            <ProexelEmptyState
              icon={PackageSearch}
              title={t("stock.empty")}
              description={t("stock.emptyDescription")}
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("stock.reference")}</TableHead>
                    <TableHead>{t("common.manufacturer")}</TableHead>
                    <TableHead>{t("stock.location")}</TableHead>
                    <TableHead>{t("stock.balance")}</TableHead>
                    <TableHead>{t("stock.minimum")}</TableHead>
                    <TableHead>{t("common.health")}</TableHead>
                    <TableHead className="text-right">{t("common.adjust")}</TableHead>
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
                          {item.quantity <= item.minimum_quantity ? t("stock.restock") : t("orders.normal")}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-2">
                          {can("stock.adjust_quantity", role) ? (
                            <CommandDialog
                              trigger={
                                <Button size="sm" variant="outline">
                                  {t("common.adjust")}
                                </Button>
                              }
                              title={t("stock.adjustTitle", { reference: item.reference })}
                              description={t("stock.adjustDescription", { quantity: item.quantity })}
                              endpoint="/api/proexel/stock"
                              method="PATCH"
                              fields={[
                                { name: "id", label: "ID", type: "hidden", defaultValue: item.id, required: true },
                                { name: "delta", label: t("stock.variation"), type: "number", required: true },
                                { name: "reason", label: t("stock.reason"), type: "textarea", required: true },
                              ]}
                            />
                          ) : null}
                          {can("stock.delete", role) && item.quantity === 0 ? (
                            <CommandButton
                              endpoint="/api/proexel/stock"
                              data={{ id: item.id }}
                              method="DELETE"
                              variant="destructive"
                              confirmMessage={t("stock.deleteConfirm")}
                            >
                              <Trash2 />
                              <span className="sr-only">{t("common.delete")}</span>
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
