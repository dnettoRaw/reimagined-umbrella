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
      const critical =
        (report.overview.machine_items.by_status.critical ?? 0) +
        (report.overview.machine_items.by_status.maintenance_required ?? 0);
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
        head: [[t("nav.machines"), t("common.components"), t("overview.critical"), t("overview.lowStock")]],
        body: [
          [report.overview.machines.total, report.overview.machine_items.total, critical, report.overview.stock.low],
        ],
        theme: "grid",
        styles: { fontSize: 9 },
      });
      autoTable(doc, {
        startY: lastTableY(doc) + 8,
        head: [[t("common.zone"), t("nav.machines"), t("common.components"), t("common.critical")]],
        body: report.by_zone.map((row) => [row.zone, row.machines, row.items, row.critical_items]),
        theme: "striped",
        styles: { fontSize: 8 },
        headStyles: { fillColor: [31, 41, 55] },
      });
      autoTable(doc, {
        startY: lastTableY(doc) + 8,
        head: [[t("orders.machine"), t("common.component"), t("common.category"), t("common.status")]],
        body: report.critical_items.map(({ item, machine, category }) => [
          machine?.code ?? "-",
          item.code,
          category?.name ?? "-",
          t(`status.${item.status}`),
        ]),
        theme: "striped",
        styles: { fontSize: 8 },
        headStyles: { fillColor: [153, 27, 27] },
      });
      autoTable(doc, {
        startY: lastTableY(doc) + 8,
        head: [[t("common.date"), t("common.technician"), t("common.category"), t("common.result")]],
        body: report.recent_inspections.map((inspection) => [
          new Intl.DateTimeFormat(intl).format(new Date(inspection.completed_at_ms ?? inspection.started_at_ms)),
          inspection.operator_name,
          inspection.category_snapshot.name,
          t(`status.${inspection.status_after ?? inspection.status_before}`),
        ]),
        theme: "grid",
        styles: { fontSize: 7 },
        headStyles: { fillColor: [31, 41, 55] },
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
