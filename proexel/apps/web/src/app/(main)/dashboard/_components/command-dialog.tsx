"use client";

import { useState } from "react";

import { useRouter } from "next/navigation";

import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";

export interface CommandField {
  name: string;
  label: string;
  type?: "text" | "number" | "date" | "email" | "url" | "textarea" | "select" | "checkbox" | "hidden";
  required?: boolean;
  placeholder?: string;
  options?: Array<{ label: string; value: string }>;
  defaultValue?: string | number | boolean;
}

export function CommandDialog({
  trigger,
  title,
  description,
  endpoint,
  method = "POST",
  fields,
}: {
  readonly trigger: React.ReactNode;
  readonly title: string;
  readonly description: string;
  readonly endpoint: string;
  readonly method?: "POST" | "PATCH";
  readonly fields: CommandField[];
}) {
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [pending, setPending] = useState(false);

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPending(true);
    const form = new FormData(event.currentTarget);
    const data = Object.fromEntries(
      fields.map((field) => {
        if (field.type === "checkbox") return [field.name, form.get(field.name) === "on"];
        const value = form.get(field.name)?.toString() ?? "";
        if (field.type === "number") return [field.name, Number(value)];
        return [field.name, value || null];
      }),
    );
    try {
      const response = await fetch(endpoint, {
        method,
        headers: { "content-type": "application/json" },
        body: JSON.stringify(data),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string };
      if (!response.ok || !result.accepted) throw new Error(result.message ?? "Operação rejeitada.");
      toast.success("Operação concluída");
      setOpen(false);
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Não foi possível concluir a operação.");
    } finally {
      setPending(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <span className="contents">{trigger}</span>
      </DialogTrigger>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <form onSubmit={submit} className="grid gap-4">
          <div className="grid gap-4 sm:grid-cols-2">
            {fields.map((field) =>
              field.type === "hidden" ? (
                <input key={field.name} type="hidden" name={field.name} value={String(field.defaultValue ?? "")} />
              ) : (
                <div key={field.name} className={field.type === "textarea" ? "grid gap-2 sm:col-span-2" : "grid gap-2"}>
                  {field.type === "checkbox" ? (
                    <label className="flex min-h-9 items-center gap-2 font-medium text-sm">
                      <input name={field.name} type="checkbox" defaultChecked={Boolean(field.defaultValue)} />
                      {field.label}
                    </label>
                  ) : (
                    <>
                      <Label htmlFor={field.name}>{field.label}</Label>
                      {field.type === "textarea" ? (
                        <Textarea
                          id={field.name}
                          name={field.name}
                          required={field.required}
                          placeholder={field.placeholder}
                        />
                      ) : field.type === "select" ? (
                        <select
                          id={field.name}
                          name={field.name}
                          required={field.required}
                          defaultValue={String(field.defaultValue ?? "")}
                          className="h-9 rounded-md border bg-background px-3 text-sm"
                        >
                          <option value="">Selecione</option>
                          {field.options?.map((option) => (
                            <option key={option.value} value={option.value}>
                              {option.label}
                            </option>
                          ))}
                        </select>
                      ) : (
                        <Input
                          id={field.name}
                          name={field.name}
                          type={field.type ?? "text"}
                          required={field.required}
                          placeholder={field.placeholder}
                          defaultValue={typeof field.defaultValue === "boolean" ? undefined : field.defaultValue}
                        />
                      )}
                    </>
                  )}
                </div>
              ),
            )}
          </div>
          <DialogFooter>
            <Button type="submit" disabled={pending}>
              {pending ? "Salvando..." : "Confirmar"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
