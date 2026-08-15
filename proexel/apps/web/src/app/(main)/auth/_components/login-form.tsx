"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { Controller, useForm } from "react-hook-form";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Field, FieldContent, FieldError, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { useI18n } from "@/lib/i18n/provider";

export function LoginForm({ next }: { readonly next?: string }) {
  const { t } = useI18n();
  const formSchema = z.object({
    email: z.string().email({ message: t("login.emailInvalid") }),
    password: z.string().min(4, { message: t("login.passwordInvalid") }),
    remember: z.boolean().optional(),
  });
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      email: "",
      password: "",
      remember: false,
    },
  });

  const onSubmit = async (data: z.infer<typeof formSchema>) => {
    const response = await fetch("/api/auth/login", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ ...data, next }),
    });
    const result = (await response.json().catch(() => ({}))) as { error?: string; next?: string };
    if (!response.ok) {
      form.setError("root", { message: result.error ?? t("login.failed") });
      return;
    }
    window.location.assign(result.next ?? "/dashboard/overview");
  };

  return (
    <form noValidate onSubmit={form.handleSubmit(onSubmit)} className="flex flex-col gap-4">
      <FieldGroup className="gap-4">
        <Controller
          control={form.control}
          name="email"
          render={({ field, fieldState }) => (
            <Field className="gap-1.5" data-invalid={fieldState.invalid}>
              <FieldLabel htmlFor="login-email">{t("common.email")}</FieldLabel>
              <Input
                {...field}
                id="login-email"
                type="email"
                placeholder="operador@proexel.local"
                autoComplete="email"
                aria-invalid={fieldState.invalid}
              />
              {fieldState.invalid && <FieldError errors={[fieldState.error]} />}
            </Field>
          )}
        />
        <Controller
          control={form.control}
          name="password"
          render={({ field, fieldState }) => (
            <Field className="gap-1.5" data-invalid={fieldState.invalid}>
              <FieldLabel htmlFor="login-password">{t("login.password")}</FieldLabel>
              <Input
                {...field}
                id="login-password"
                type="password"
                placeholder="••••••••"
                autoComplete="current-password"
                aria-invalid={fieldState.invalid}
              />
              {fieldState.invalid && <FieldError errors={[fieldState.error]} />}
            </Field>
          )}
        />
        <Controller
          control={form.control}
          name="remember"
          render={({ field, fieldState }) => (
            <Field orientation="horizontal" data-invalid={fieldState.invalid}>
              <Checkbox
                id="login-remember"
                name={field.name}
                checked={field.value}
                onCheckedChange={(checked) => field.onChange(Boolean(checked))}
                aria-invalid={fieldState.invalid}
              />
              <FieldContent>
                <FieldLabel htmlFor="login-remember" className="font-normal">
                  {t("login.remember")}
                </FieldLabel>
                {fieldState.invalid && <FieldError errors={[fieldState.error]} />}
              </FieldContent>
            </Field>
          )}
        />
      </FieldGroup>
      {form.formState.errors.root ? <FieldError errors={[form.formState.errors.root]} /> : null}
      <Button className="w-full" type="submit" disabled={form.formState.isSubmitting}>
        {form.formState.isSubmitting ? t("login.submitting") : t("login.submit")}
      </Button>
    </form>
  );
}
