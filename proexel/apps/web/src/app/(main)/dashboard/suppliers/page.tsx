import { Building2, Pencil, Plus, Trash2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listSuppliers } from "@/lib/proexel/service";

import { CommandButton } from "../_components/command-button";
import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

export const dynamic = "force-dynamic";

export default async function SuppliersPage() {
  const [{ role }, suppliers, { t }] = await Promise.all([
    requirePermission("supplier.read"),
    listSuppliers(),
    getI18n(),
  ]);
  return (
    <div>
      <PageHeader
        title={t("nav.suppliers")}
        description={t("suppliers.description")}
        action={
          can("supplier.create_update_delete", role) ? (
            <CommandDialog
              trigger={
                <Button>
                  <Plus />
                  {t("suppliers.new")}
                </Button>
              }
              title={t("suppliers.create")}
              description={t("suppliers.createDescription")}
              endpoint="/api/proexel/suppliers"
              fields={[
                { name: "name", label: t("common.name"), required: true },
                { name: "contact", label: t("common.contact"), required: true },
                { name: "email", label: t("common.email"), type: "email" },
                { name: "website", label: t("common.website"), type: "url" },
                { name: "notes", label: t("common.notes"), type: "textarea" },
              ]}
            />
          ) : null
        }
      />
      <Card>
        <CardHeader>
          <CardTitle>{t("suppliers.registry")}</CardTitle>
          <CardDescription>{t("suppliers.count", { count: suppliers.items.length })}</CardDescription>
        </CardHeader>
        <CardContent>
          {suppliers.items.length === 0 ? (
            <ProexelEmptyState
              icon={Building2}
              title={t("suppliers.none")}
              description={t("suppliers.noneDescription")}
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("common.name")}</TableHead>
                    <TableHead>{t("common.contact")}</TableHead>
                    <TableHead>{t("common.email")}</TableHead>
                    <TableHead>{t("common.website")}</TableHead>
                    <TableHead>{t("common.notes")}</TableHead>
                    <TableHead className="text-right">{t("common.actions")}</TableHead>
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
                            {t("common.open")}
                          </a>
                        ) : (
                          "-"
                        )}
                      </TableCell>
                      <TableCell className="max-w-80 whitespace-normal">{supplier.notes ?? "-"}</TableCell>
                      <TableCell>
                        <div className="flex justify-end gap-2">
                          {can("supplier.create_update_delete", role) ? (
                            <>
                              <CommandDialog
                                trigger={
                                  <Button size="icon-sm" variant="ghost" title={t("common.edit")}>
                                    <Pencil />
                                    <span className="sr-only">{t("common.edit")}</span>
                                  </Button>
                                }
                                title={t("suppliers.edit")}
                                description={t("suppliers.editDescription")}
                                endpoint="/api/proexel/suppliers"
                                method="PATCH"
                                fields={[
                                  { name: "id", label: "ID", type: "hidden", defaultValue: supplier.id },
                                  {
                                    name: "name",
                                    label: t("common.name"),
                                    required: true,
                                    defaultValue: supplier.name,
                                  },
                                  {
                                    name: "contact",
                                    label: t("common.contact"),
                                    required: true,
                                    defaultValue: supplier.contact,
                                  },
                                  {
                                    name: "email",
                                    label: t("common.email"),
                                    type: "email",
                                    defaultValue: supplier.email ?? "",
                                  },
                                  {
                                    name: "website",
                                    label: t("common.website"),
                                    type: "url",
                                    defaultValue: supplier.website ?? "",
                                  },
                                  {
                                    name: "notes",
                                    label: t("common.notes"),
                                    type: "textarea",
                                    defaultValue: supplier.notes ?? "",
                                  },
                                ]}
                              />
                              <CommandButton
                                endpoint="/api/proexel/suppliers"
                                data={{ id: supplier.id }}
                                method="DELETE"
                                variant="destructive"
                                confirmMessage={t("suppliers.deleteConfirm")}
                              >
                                <Trash2 />
                                <span className="sr-only">{t("common.delete")}</span>
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
