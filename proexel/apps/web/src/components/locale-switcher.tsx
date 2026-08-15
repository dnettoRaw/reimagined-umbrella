"use client";

import { Languages } from "lucide-react";

import { NativeSelect, NativeSelectOption } from "@/components/ui/native-select";
import { setClientCookie } from "@/lib/cookie.client";
import { LOCALE_COOKIE, LOCALE_LABELS, LOCALES, type Locale } from "@/lib/i18n/config";
import { useI18n } from "@/lib/i18n/provider";

export function LocaleSwitcher({ compact = false }: { readonly compact?: boolean }) {
  const { locale, t } = useI18n();

  function changeLocale(nextLocale: Locale) {
    setClientCookie(LOCALE_COOKIE, nextLocale, 365);
    window.location.reload();
  }

  return (
    <div className="relative flex items-center">
      <Languages className="pointer-events-none absolute left-2.5 z-10 size-4 text-muted-foreground" />
      <NativeSelect
        value={locale}
        onChange={(event) => changeLocale(event.target.value as Locale)}
        aria-label={t("language.change")}
        className={compact ? "w-24 [&_select]:pl-8" : "w-36 [&_select]:pl-8"}
      >
        {LOCALES.map((value) => (
          <NativeSelectOption key={value} value={value}>
            {compact ? value.toUpperCase() : LOCALE_LABELS[value]}
          </NativeSelectOption>
        ))}
      </NativeSelect>
    </div>
  );
}
