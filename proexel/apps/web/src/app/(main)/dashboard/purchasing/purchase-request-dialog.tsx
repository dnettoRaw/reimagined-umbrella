"use client";

import { useState } from "react";

import { useRouter } from "next/navigation";

import { Plus } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "@/components/ui/command";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { useI18n } from "@/lib/i18n/provider";

export interface PurchaseComponentOption {
  id: string;
  code: string;
  name: string;
  machine: string;
  reference: string;
}

export function PurchaseRequestDialog({ components }: { readonly components: PurchaseComponentOption[] }) {
  const router = useRouter();
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [selectedId, setSelectedId] = useState("");
  const selected = components.find((component) => component.id === selectedId);

  function changeOpen(next: boolean) {
    setOpen(next);
    if (!next) setSelectedId("");
  }

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selected) return;
    setPending(true);
    const form = new FormData(event.currentTarget);

    try {
      const response = await fetch("/api/proexel/purchasing", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ reference: selected.reference, reason: form.get("reason")?.toString() ?? "" }),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string };
      if (!response.ok || !result.accepted) throw new Error(result.message ?? t("command.rejected"));
      toast.success(t("command.success"));
      changeOpen(false);
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t("command.failed"));
    } finally {
      setPending(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={changeOpen}>
      <DialogTrigger asChild>
        <Button>
          <Plus />
          {t("purchasing.request")}
        </Button>
      </DialogTrigger>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{t("purchasing.request")}</DialogTitle>
          <DialogDescription>{t("purchasing.requestDescription")}</DialogDescription>
        </DialogHeader>
        <form onSubmit={submit} className="grid gap-4">
          <div className="grid gap-2">
            <Label>{t("purchasing.component")}</Label>
            <Command className="h-auto rounded-lg border">
              <CommandInput placeholder={t("purchasing.componentSearch")} />
              <CommandList className="max-h-64">
                <CommandEmpty>{t("purchasing.noComponents")}</CommandEmpty>
                <CommandGroup>
                  {components.map((component) => (
                    <CommandItem
                      key={component.id}
                      value={`${component.id} ${component.code} ${component.name} ${component.machine} ${component.reference}`}
                      data-checked={component.id === selectedId}
                      onSelect={() => setSelectedId(component.id)}
                      className="items-start py-2"
                    >
                      <span className="min-w-0 flex-1">
                        <span className="block font-medium">
                          {component.code} · {component.name}
                        </span>
                        <span className="block truncate text-muted-foreground text-xs">{component.machine}</span>
                        <span className="block truncate text-muted-foreground text-xs">
                          {t("stock.reference")}: {component.reference}
                        </span>
                      </span>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </CommandList>
            </Command>
            {selected ? (
              <p className="text-muted-foreground text-xs">
                {t("purchasing.selectedReference")}: <strong className="text-foreground">{selected.reference}</strong>
              </p>
            ) : null}
          </div>
          <div className="grid gap-2">
            <Label htmlFor="purchase-reason">{t("stock.reason")}</Label>
            <Textarea id="purchase-reason" name="reason" required />
          </div>
          <DialogFooter>
            <Button type="submit" disabled={pending || !selected}>
              {pending ? t("common.saving") : t("common.confirm")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
