"use client";

import { Badge } from "@/components/ui/badge";
import { useI18n } from "@/lib/i18n/provider";
import type { OperationalStatus } from "@/lib/proexel/types";

export function OperationalStatusBadge({ status }: { readonly status: OperationalStatus }) {
  const { t } = useI18n();
  return (
    <Badge
      variant={
        status === "critical" || status === "maintenance_required"
          ? "destructive"
          : status === "attention" || status === "under_maintenance"
            ? "secondary"
            : "outline"
      }
    >
      {t(`status.${status}`)}
    </Badge>
  );
}
