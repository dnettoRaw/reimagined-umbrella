import { Check, Plus, ShoppingCart, Trash2, X } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { getI18n } from "@/lib/i18n/server";
import { requireSession } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listRestockRequests, listStock } from "@/lib/proexel/service";

import { CommandButton } from "../_components/command-button";
import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";
import { PurchasePlan } from "./purchase-plan";

export const dynamic = "force-dynamic";

export default async function PurchasingPage() {
  const [{ role }, requests, stock, { t }] = await Promise.all([
    requireSession(),
    listRestockRequests(),
    listStock(),
    getI18n(),
  ]);
  return (
    <div>
      <PageHeader
        title={t("purchasing.title")}
        description={t("purchasing.description")}
        action={
          can("restock.create_suggestion", role) ? (
            <CommandDialog
              trigger={
                <Button>
                  <Plus />
                  {t("purchasing.request")}
                </Button>
              }
              title={t("purchasing.request")}
              description={t("purchasing.requestDescription")}
              endpoint="/api/proexel/purchasing"
              fields={[
                { name: "reference", label: t("stock.reference"), required: true },
                { name: "reason", label: t("stock.reason"), type: "textarea", required: true },
              ]}
            />
          ) : null
        }
      />
      <Card>
        <CardHeader>
          <CardTitle>{t("purchasing.requests")}</CardTitle>
          <CardDescription>
            {t("purchasing.pendingCount", { count: requests.items.filter((item) => item.status === "pending").length })}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {requests.items.length === 0 ? (
            <ProexelEmptyState
              icon={ShoppingCart}
              title={t("purchasing.none")}
              description={t("purchasing.noneDescription")}
            />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("stock.reference")}</TableHead>
                    <TableHead>{t("stock.reason")}</TableHead>
                    <TableHead>{t("purchasing.requester")}</TableHead>
                    <TableHead>{t("common.status")}</TableHead>
                    <TableHead>{t("common.reviewer")}</TableHead>
                    <TableHead className="text-right">{t("purchasing.decision")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {requests.items.map((item) => (
                    <TableRow key={item.id}>
                      <TableCell className="font-medium">{item.reference}</TableCell>
                      <TableCell className="max-w-80 whitespace-normal">{item.reason}</TableCell>
                      <TableCell>{item.requested_by}</TableCell>
                      <TableCell>
                        <RequestStatus
                          status={item.status}
                          label={
                            item.status === "pending"
                              ? t("common.pending")
                              : item.status === "approved"
                                ? t("purchasing.approved")
                                : t("purchasing.rejected")
                          }
                        />
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
                                {t("common.approve")}
                              </CommandButton>
                              <CommandButton
                                endpoint="/api/proexel/purchasing"
                                data={{ id: item.id, status: "rejected" }}
                                variant="ghost"
                              >
                                <X />
                                {t("common.reject")}
                              </CommandButton>
                            </>
                          ) : null}
                          {can("restock.delete", role) && item.status !== "approved" ? (
                            <CommandButton
                              endpoint="/api/proexel/purchasing"
                              data={{ id: item.id }}
                              method="DELETE"
                              variant="destructive"
                              confirmMessage={t("purchasing.deleteConfirm")}
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
      {can("stock.read", role) ? <PurchasePlan stock={stock.items} requests={requests.items} /> : null}
    </div>
  );
}

function RequestStatus({
  status,
  label,
}: {
  readonly status: "pending" | "approved" | "rejected";
  readonly label: string;
}) {
  return (
    <Badge variant={status === "approved" ? "default" : status === "rejected" ? "destructive" : "secondary"}>
      {label}
    </Badge>
  );
}
