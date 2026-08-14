"use client";

import { useState } from "react";

import { useRouter } from "next/navigation";

import { toast } from "sonner";

import { Button } from "@/components/ui/button";

export function CommandButton({
  endpoint,
  data,
  children,
  variant = "outline",
}: {
  readonly endpoint: string;
  readonly data: Record<string, unknown>;
  readonly children: React.ReactNode;
  readonly variant?: "default" | "outline" | "destructive" | "secondary" | "ghost";
}) {
  const router = useRouter();
  const [pending, setPending] = useState(false);

  async function run() {
    setPending(true);
    try {
      const response = await fetch(endpoint, {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(data),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string };
      if (!response.ok || !result.accepted) throw new Error(result.message ?? "Operação rejeitada.");
      toast.success("Status atualizado");
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Não foi possível atualizar.");
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
