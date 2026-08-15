import Link from "next/link";

import { Lock } from "lucide-react";

import { getI18n } from "@/lib/i18n/server";

export default async function UnauthorizedPage() {
  const { t } = await getI18n();
  return (
    <div className="flex min-h-dvh flex-col items-center justify-center bg-background px-4 py-12 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-md text-center">
        <Lock className="mx-auto size-12 text-primary" />
        <h1 className="mt-4 font-bold text-3xl sm:text-4xl">{t("unauthorized.title")}</h1>
        <p className="mt-4 text-muted-foreground">{t("unauthorized.description")}</p>
        <div className="mt-6">
          <Link
            href="/dashboard/overview"
            className="inline-flex items-center rounded-md bg-primary px-4 py-2 font-medium text-primary-foreground text-sm shadow-xs transition-colors hover:bg-primary/90 focus:outline-hidden focus:ring-2 focus:ring-primary focus:ring-offset-2"
            prefetch={false}
          >
            {t("unauthorized.back")}
          </Link>
        </div>
      </div>
    </div>
  );
}
