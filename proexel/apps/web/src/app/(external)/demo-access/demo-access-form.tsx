"use client";

import { useState } from "react";

import { ClipboardCheck, LockKeyhole, ShieldCheck, ShoppingCart, Wrench } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Field, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { useI18n } from "@/lib/i18n/provider";
import type { DemoProfile } from "@/lib/proexel/demo-access";
import type { Role } from "@/lib/proexel/types";

function safeDestination(value: string | undefined) {
  return value?.startsWith("/") && !value.startsWith("//") ? value : "/dashboard/overview";
}

const ROLE_ICONS = {
  admin: ShieldCheck,
  chefe: ClipboardCheck,
  compras: ShoppingCart,
  tecnico: Wrench,
} satisfies Record<Role, typeof ShieldCheck>;

export function DemoAccessForm({
  next,
  initialProfiles,
}: {
  readonly next?: string;
  readonly initialProfiles: DemoProfile[];
}) {
  const { t } = useI18n();
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [profiles, setProfiles] = useState(initialProfiles);

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError("");
    setSubmitting(true);

    try {
      const response = await fetch("/api/demo-access", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ password }),
      });
      const result = (await response.json().catch(() => ({}))) as { profiles?: DemoProfile[] };
      if (!response.ok || !result.profiles) {
        setError(t("demoAccess.invalid"));
        return;
      }
      setProfiles(result.profiles);
    } catch {
      setError(t("demoAccess.failed"));
    } finally {
      setSubmitting(false);
    }
  }

  async function selectProfile(profileId: string) {
    setError("");
    setSubmitting(true);
    try {
      const response = await fetch("/api/demo-access", {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ profileId }),
      });
      if (!response.ok) {
        setError(t("demoAccess.profileFailed"));
        return;
      }
      window.location.assign(safeDestination(next));
    } catch {
      setError(t("demoAccess.profileFailed"));
    } finally {
      setSubmitting(false);
    }
  }

  if (profiles.length > 0) {
    return (
      <div className="flex flex-col gap-4">
        <div>
          <h3 className="font-heading font-semibold text-lg">{t("demoAccess.profileTitle")}</h3>
          <p className="mt-1 text-muted-foreground text-sm">{t("demoAccess.profileDescription")}</p>
        </div>
        <div className="grid gap-2">
          {profiles.map((profile) => {
            const Icon = ROLE_ICONS[profile.role];
            return (
              <button
                key={profile.id}
                type="button"
                className="flex min-h-20 w-full items-center gap-3 rounded-lg border bg-background p-3 text-left transition-colors hover:border-primary hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50"
                onClick={() => selectProfile(profile.id)}
                disabled={submitting}
              >
                <span className="flex size-10 shrink-0 items-center justify-center rounded-md bg-muted text-foreground">
                  <Icon className="size-5" />
                </span>
                <span className="min-w-0 flex-1">
                  <span className="block truncate font-medium text-sm">{profile.name}</span>
                  <span className="block text-muted-foreground text-xs">
                    {t(`role.${profile.role}`)} · {t("demoAccess.repairLevel", { level: profile.maximumRepairLevel })}
                  </span>
                </span>
                <span className="text-primary text-xs">{t("demoAccess.selectProfile")}</span>
              </button>
            );
          })}
        </div>
        {error ? <FieldError>{error}</FieldError> : null}
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-8">
      <div>
        <h2 className="font-heading font-semibold text-2xl">{t("demoAccess.title")}</h2>
        <p className="mt-2 text-muted-foreground text-sm">{t("demoAccess.description")}</p>
      </div>
      <form className="flex flex-col gap-4" onSubmit={submit}>
        <Field data-invalid={Boolean(error)}>
          <FieldLabel htmlFor="demo-password">{t("demoAccess.password")}</FieldLabel>
          <div className="relative">
            <LockKeyhole className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              id="demo-password"
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              className="pl-9"
              autoComplete="current-password"
              autoFocus
              required
              aria-invalid={Boolean(error)}
            />
          </div>
          {error ? <FieldError>{error}</FieldError> : null}
        </Field>
        <Button className="w-full" type="submit" disabled={submitting || password.length === 0}>
          {submitting ? t("demoAccess.submitting") : t("demoAccess.submit")}
        </Button>
      </form>
    </div>
  );
}
