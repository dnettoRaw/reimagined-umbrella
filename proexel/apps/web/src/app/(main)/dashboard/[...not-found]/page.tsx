"use client";

import { useI18n } from "@/lib/i18n/provider";

export default function DashboardNotFound() {
  const { t } = useI18n();
  return (
    <div className="flex h-full flex-col items-center justify-center space-y-2 text-center">
      <h1 className="font-semibold text-2xl">{t("notFound.title")}</h1>
      <p className="text-muted-foreground">{t("notFound.dashboardDescription")}</p>
    </div>
  );
}
