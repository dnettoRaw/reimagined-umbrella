"use client";

import { useEffect, useState } from "react";

import Link from "next/link";

import { ClipboardCheck, ExternalLink, LoaderCircle } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import { SidebarMenuButton, SidebarMenuItem } from "@/components/ui/sidebar";
import { useI18n } from "@/lib/i18n/provider";
import type { ListResult, Role, ServiceOrder } from "@/lib/proexel/types";

export function MyOrdersDialog({ role, operatorId }: { role: Role; operatorId: string }) {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const [orders, setOrders] = useState<ServiceOrder[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!open) return;
    const controller = new AbortController();
    setLoading(true);
    const query = role === "tecnico" ? `?operator_id=${encodeURIComponent(operatorId)}` : "";
    void fetch(`/api/proexel/orders${query}`, { signal: controller.signal })
      .then((response) => response.json() as Promise<ListResult<ServiceOrder>>)
      .then((result) =>
        setOrders(result.items.filter((order) => order.status === "pending" || order.status === "in_progress")),
      )
      .catch((error: unknown) => {
        if (!(error instanceof DOMException && error.name === "AbortError")) setOrders([]);
      })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, [open, operatorId, role]);

  return (
    <SidebarMenuItem>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogTrigger asChild>
          <SidebarMenuButton
            tooltip={t("nav.execution")}
            className="min-w-8 bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground"
          >
            <ClipboardCheck />
            <span>{t("nav.execution")}</span>
          </SidebarMenuButton>
        </DialogTrigger>
        <DialogContent className="sm:max-w-2xl">
          <DialogHeader>
            <DialogTitle>{t("execution.myOrdersTitle")}</DialogTitle>
            <DialogDescription>{t("execution.myOrdersDescription")}</DialogDescription>
          </DialogHeader>
          <ScrollArea className="max-h-[65vh] pr-3">
            {loading ? (
              <div className="flex min-h-32 items-center justify-center">
                <LoaderCircle className="animate-spin" />
                <span className="sr-only">{t("common.loading")}</span>
              </div>
            ) : orders.length ? (
              <div className="divide-y rounded-md border">
                {orders.map((order) => {
                  const progress = order.tasks.length ? (order.completed_tasks / order.tasks.length) * 100 : 0;
                  return (
                    <article key={order.id} className="space-y-3 p-4">
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0">
                          <h3 className="truncate font-semibold text-sm">
                            {order.machine_snapshot.code} · {order.machine_snapshot.name}
                          </h3>
                          <p className="line-clamp-2 text-muted-foreground text-sm">{order.description}</p>
                        </div>
                        <Badge variant={order.status === "in_progress" ? "default" : "secondary"}>
                          {order.status === "in_progress" ? t("common.inProgress") : t("common.pending")}
                        </Badge>
                      </div>
                      <div className="flex items-center gap-3">
                        <Progress value={progress} className="flex-1" />
                        <span className="shrink-0 text-muted-foreground text-xs">
                          {t("orders.progress", { completed: order.completed_tasks, total: order.tasks.length })}
                        </span>
                        <Button asChild size="icon-sm" title={t("execution.openOrder")}>
                          <Link
                            href={`/dashboard/execution/${encodeURIComponent(order.id)}`}
                            onClick={() => setOpen(false)}
                          >
                            <ExternalLink />
                            <span className="sr-only">{t("execution.openOrder")}</span>
                          </Link>
                        </Button>
                      </div>
                    </article>
                  );
                })}
              </div>
            ) : (
              <div className="flex min-h-32 flex-col items-center justify-center gap-2 text-center">
                <ClipboardCheck className="size-8 text-muted-foreground" />
                <p className="font-medium text-sm">{t("execution.noOrders")}</p>
                <p className="text-muted-foreground text-sm">{t("execution.noOrdersDescription")}</p>
              </div>
            )}
          </ScrollArea>
          <div className="flex justify-end border-t pt-4">
            <Button asChild variant="outline">
              <Link href="/dashboard/execution" onClick={() => setOpen(false)}>
                <ClipboardCheck />
                {t("execution.viewAllOrders")}
              </Link>
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </SidebarMenuItem>
  );
}
