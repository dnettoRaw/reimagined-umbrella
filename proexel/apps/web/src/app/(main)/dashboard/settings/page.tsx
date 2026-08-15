import { CircleCheck, CircleX, Server } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { getI18n } from "@/lib/i18n/server";
import { getRuntimeStatus } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";

export const dynamic = "force-dynamic";

export default async function SettingsPage() {
  const [runtime, { t }] = await Promise.all([getRuntimeStatus(), getI18n()]);
  const StatusIcon = runtime.healthy ? CircleCheck : CircleX;
  return (
    <div>
      <PageHeader title={t("nav.settings")} description={t("settings.description")} />
      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Server className="size-5" />
              <CardTitle>{t("settings.runtime")}</CardTitle>
            </div>
            <CardDescription>{t("settings.connectivity")}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground text-sm">{t("common.status")}</span>
              <Badge variant={runtime.healthy ? "outline" : "destructive"} className="gap-1">
                <StatusIcon />
                {runtime.healthy
                  ? t("settings.healthy")
                  : runtime.configured
                    ? t("common.unavailable")
                    : t("common.notConfigured")}
              </Badge>
            </div>
            <div className="flex items-center justify-between gap-4">
              <span className="text-muted-foreground text-sm">{t("settings.endpoint")}</span>
              <code className="truncate text-xs">{runtime.url ?? "-"}</code>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>{t("settings.policies")}</CardTitle>
            <CardDescription>{t("settings.policyDescription")}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            <div className="flex justify-between">
              <span>{t("settings.warningMaintenance")}</span>
              <strong>{t("settings.days150")}</strong>
            </div>
            <div className="flex justify-between">
              <span>{t("settings.criticalMaintenance")}</span>
              <strong>{t("settings.days180")}</strong>
            </div>
            <div className="flex justify-between">
              <span>{t("settings.negativeStock")}</span>
              <strong>{t("common.blocked")}</strong>
            </div>
            <div className="flex justify-between">
              <span>{t("settings.localSchema")}</span>
              <strong>v1</strong>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
