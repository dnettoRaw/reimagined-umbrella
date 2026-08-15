"use client";

import { useState } from "react";

import { jsPDF } from "jspdf";
import autoTable from "jspdf-autotable";
import { Download, Loader2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { INTL_LOCALES } from "@/lib/i18n/config";
import { useI18n } from "@/lib/i18n/provider";
import type { ReportResult } from "@/lib/proexel/types";

export function ReportExport({ report }: { readonly report: ReportResult }) {
  const { t, locale } = useI18n();
  const [pending, setPending] = useState(false);

  function exportPdf() {
    setPending(true);
    try {
      const intl = INTL_LOCALES[locale];
      const doc = new jsPDF({ unit: "mm", format: "a4" });
      const generated = new Intl.DateTimeFormat(intl, { dateStyle: "medium", timeStyle: "short" }).format(
        new Date(report.generated_at_ms),
      );
      doc.setProperties({
        title: `PROEXEL - ${t("nav.reports")}`,
        subject: t("reports.description"),
        creator: "PROEXEL",
      });
      doc.setFontSize(18);
      doc.text(`PROEXEL - ${t("nav.reports")}`, 14, 18);
      doc.setFontSize(9);
      doc.setTextColor(90);
      doc.text(t("reports.generatedAt", { date: generated }), 14, 24);
      autoTable(doc, {
        startY: 30,
        head: [[t("overview.valves"), t("overview.critical"), t("overview.openOrders"), t("overview.lowStock")]],
        body: [
          [
            report.overview.valves.total,
            report.overview.valves.critical,
            report.overview.orders.open,
            report.overview.stock.low,
          ],
        ],
        theme: "grid",
        styles: { fontSize: 9 },
      });
      autoTable(doc, {
        startY: lastTableY(doc) + 8,
        head: [[t("common.zone"), t("overview.valves"), t("common.critical"), t("common.warning")]],
        body: report.by_zone.map((row) => [row.zone, row.total, row.critical, row.warning]),
        theme: "striped",
        styles: { fontSize: 8 },
        headStyles: { fillColor: [31, 41, 55] },
      });
      autoTable(doc, {
        startY: lastTableY(doc) + 8,
        head: [["TAG", t("common.zone"), t("valves.lastMaintenance"), t("common.health")]],
        body: report.critical_valves.map((valve) => [
          valve.tag,
          valve.zone,
          valve.last_maintenance_at
            ? new Intl.DateTimeFormat(intl).format(new Date(`${valve.last_maintenance_at}T00:00:00`))
            : t("common.never"),
          t("common.critical"),
        ]),
        theme: "striped",
        styles: { fontSize: 8 },
        headStyles: { fillColor: [153, 27, 27] },
      });
      autoTable(doc, {
        startY: lastTableY(doc) + 8,
        head: [[t("common.date"), "TAG", t("common.technician"), t("common.type"), t("maintenance.service")]],
        body: report.recent_maintenance.map((item) => [
          new Intl.DateTimeFormat(intl).format(new Date(`${item.performed_at}T00:00:00`)),
          item.valve_tag_snapshot,
          item.technician,
          item.maintenance_type === "preventive" ? t("maintenance.preventive") : t("maintenance.corrective"),
          item.service,
        ]),
        theme: "grid",
        styles: { fontSize: 7, cellWidth: "wrap", overflow: "linebreak" },
        headStyles: { fillColor: [31, 41, 55] },
        columnStyles: { 4: { cellWidth: 70 } },
        margin: { bottom: 14 },
      });
      const pages = doc.getNumberOfPages();
      for (let page = 1; page <= pages; page += 1) {
        doc.setPage(page);
        doc.setFontSize(8);
        doc.setTextColor(100);
        doc.text(t("reports.pageNumber", { page, pages }), 196, 290, { align: "right" });
      }
      doc.save(`proexel-report-${new Date().toISOString().slice(0, 10)}-${locale}.pdf`);
    } finally {
      setPending(false);
    }
  }

  return (
    <Button type="button" onClick={exportPdf} disabled={pending || report.source !== "appcore"}>
      {pending ? <Loader2 className="animate-spin" /> : <Download />}
      {pending ? t("reports.exporting") : t("reports.exportPdf")}
    </Button>
  );
}

function lastTableY(doc: jsPDF) {
  return (doc as jsPDF & { lastAutoTable?: { finalY: number } }).lastAutoTable?.finalY ?? 40;
}
