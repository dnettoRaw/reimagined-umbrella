"use client";

import { useMemo, useState } from "react";

import { Download, Mail } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { useI18n } from "@/lib/i18n/provider";
import type { RestockRequest, StockItem } from "@/lib/proexel/types";

export function PurchasePlan({ stock, requests }: { stock: StockItem[]; requests: RestockRequest[] }) {
  const { t } = useI18n();
  const suggested = useMemo(() => buildDemand(stock, requests), [stock, requests]);
  const [quantities, setQuantities] = useState<Record<string, number>>(() =>
    Object.fromEntries(suggested.map((row) => [row.reference, row.quantity])),
  );
  const rows = suggested.map((row) => ({ ...row, quantity: quantities[row.reference] ?? row.quantity }));

  function csv() {
    const content = [
      [t("stock.reference"), t("purchasing.quantity"), t("stock.balance"), t("stock.minimum")],
      ...rows.map((row) => [row.reference, row.quantity, row.balance, row.minimum]),
    ]
      .map((row) => row.map(csvCell).join(","))
      .join("\n");
    const url = URL.createObjectURL(new Blob([content], { type: "text/csv;charset=utf-8" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = `proexel-purchase-${new Date().toISOString().slice(0, 10)}.csv`;
    link.click();
    URL.revokeObjectURL(url);
  }

  function email() {
    const body = rows.map((row) => `${row.reference}: ${row.quantity}`).join("\n");
    window.location.href = `mailto:?subject=${encodeURIComponent(t("purchasing.plan"))}&body=${encodeURIComponent(body)}`;
  }

  return (
    <Card className="mt-4">
      <CardHeader className="gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <CardTitle>{t("purchasing.plan")}</CardTitle>
          <CardDescription>{t("purchasing.planDescription")}</CardDescription>
        </div>
        {rows.length ? (
          <div className="flex gap-2">
            <Button type="button" variant="outline" onClick={csv}>
              <Download />
              {t("purchasing.exportCsv")}
            </Button>
            <Button type="button" variant="outline" onClick={email}>
              <Mail />
              {t("purchasing.createEmail")}
            </Button>
          </div>
        ) : null}
      </CardHeader>
      <CardContent>
        {rows.length ? (
          <div className="divide-y">
            {rows.map((row) => (
              <div key={row.reference} className="grid gap-2 py-3 sm:grid-cols-[1fr_120px_120px_160px] sm:items-center">
                <strong>{row.reference}</strong>
                <span className="text-muted-foreground text-sm">
                  {t("stock.balance")}: {row.balance}
                </span>
                <span className="text-muted-foreground text-sm">
                  {t("stock.minimum")}: {row.minimum}
                </span>
                <Input
                  type="number"
                  min={1}
                  aria-label={`${t("purchasing.quantity")} ${row.reference}`}
                  value={row.quantity}
                  onChange={(event) =>
                    setQuantities((current) => ({
                      ...current,
                      [row.reference]: Math.max(1, Number(event.target.value) || 1),
                    }))
                  }
                />
              </div>
            ))}
          </div>
        ) : (
          <p className="text-muted-foreground text-sm">{t("purchasing.noDemand")}</p>
        )}
      </CardContent>
    </Card>
  );
}

function buildDemand(stock: StockItem[], requests: RestockRequest[]) {
  const demand = new Map<string, { reference: string; quantity: number; balance: number; minimum: number }>();
  for (const item of stock) {
    if (item.quantity <= item.minimum_quantity) {
      demand.set(item.reference, {
        reference: item.reference,
        quantity: Math.max(1, item.minimum_quantity - item.quantity + 1),
        balance: item.quantity,
        minimum: item.minimum_quantity,
      });
    }
  }
  for (const request of requests) {
    if (request.status !== "approved") continue;
    const current = demand.get(request.reference);
    if (current) current.quantity += 1;
    else demand.set(request.reference, { reference: request.reference, quantity: 1, balance: 0, minimum: 0 });
  }
  return [...demand.values()].toSorted((left, right) => left.reference.localeCompare(right.reference));
}

function csvCell(value: string | number) {
  const text = String(value);
  return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
}
