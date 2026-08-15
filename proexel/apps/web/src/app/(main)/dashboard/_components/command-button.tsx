"use client";

import { useState } from "react";

import { useRouter } from "next/navigation";

import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { useI18n } from "@/lib/i18n/provider";

export function CommandButton({
  endpoint,
  data,
  children,
  variant = "outline",
  method = "PATCH",
  confirmMessage,
}: {
  readonly endpoint: string;
  readonly data: Record<string, unknown>;
  readonly children: React.ReactNode;
  readonly variant?: "default" | "outline" | "destructive" | "secondary" | "ghost";
  readonly method?: "PATCH" | "DELETE";
  readonly confirmMessage?: string;
}) {
  const router = useRouter();
  const { t } = useI18n();
  const [pending, setPending] = useState(false);

  async function run() {
    if (confirmMessage && !window.confirm(confirmMessage)) return;
    setPending(true);
    try {
      const response = await fetch(endpoint, {
        method,
        headers: { "content-type": "application/json" },
        body: JSON.stringify(data),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string };
      if (!response.ok || !result.accepted) throw new Error(result.message ?? t("command.rejected"));
      toast.success(t("command.statusUpdated"));
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t("command.updateFailed"));
    } finally {
      setPending(false);
    }
  }

  return (
    <Button type="button" size="sm" variant={variant} disabled={pending} onClick={run}>
      {children}
    </Button>
  );
}
