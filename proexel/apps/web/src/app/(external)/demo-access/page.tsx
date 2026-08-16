import { cookies } from "next/headers";

import { Factory } from "lucide-react";

import { LocaleSwitcher } from "@/components/locale-switcher";
import { getI18n } from "@/lib/i18n/server";
import { createDemoAccessToken, DEMO_ACCESS_COOKIE, DEMO_PROFILES, getDemoPassword } from "@/lib/proexel/demo-access";

import { DemoAccessForm } from "./demo-access-form";

export default async function DemoAccessPage({ searchParams }: { readonly searchParams: Promise<{ next?: string }> }) {
  const [{ next }, { t }, cookieStore, expectedToken] = await Promise.all([
    searchParams,
    getI18n(),
    cookies(),
    createDemoAccessToken(getDemoPassword()),
  ]);
  const hasAccess = cookieStore.get(DEMO_ACCESS_COOKIE)?.value === expectedToken;

  return (
    <main className="grid min-h-dvh bg-background lg:grid-cols-[minmax(320px,0.75fr)_1.25fr]">
      <section className="flex flex-col justify-between border-b bg-foreground p-8 text-background lg:border-r lg:border-b-0 lg:p-12">
        <div className="flex items-center gap-3">
          <Factory className="size-7" />
          <strong className="text-xl">PROEXEL</strong>
        </div>
        <div className="mt-16 max-w-md lg:mt-0">
          <h1 className="font-heading font-semibold text-3xl sm:text-4xl">{t("demoAccess.hero")}</h1>
          <p className="mt-4 text-background/70">{t("demoAccess.heroDescription")}</p>
        </div>
        <p className="mt-12 text-background/60 text-xs">{t("demoAccess.restricted")}</p>
      </section>
      <section className="flex items-center justify-center p-6 sm:p-10">
        <div className="w-full max-w-sm">
          <div className="mb-8 flex justify-end">
            <LocaleSwitcher />
          </div>
          <DemoAccessForm next={next} initialProfiles={hasAccess ? DEMO_PROFILES : []} />
        </div>
      </section>
    </main>
  );
}
