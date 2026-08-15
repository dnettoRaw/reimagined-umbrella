"use client";

import { createContext, useContext, useMemo } from "react";

import { INTL_LOCALES, type Locale } from "./config";
import { type TranslationKey, type TranslationValues, translate } from "./messages";

type I18nContextValue = {
  locale: Locale;
  t: (key: TranslationKey, values?: TranslationValues) => string;
  formatDate: (value: string | number | Date, options?: Intl.DateTimeFormatOptions) => string;
  formatNumber: (value: number, options?: Intl.NumberFormatOptions) => string;
};

const I18nContext = createContext<I18nContextValue | null>(null);

export function I18nProvider({ locale, children }: { readonly locale: Locale; readonly children: React.ReactNode }) {
  const value = useMemo<I18nContextValue>(() => {
    const intlLocale = INTL_LOCALES[locale];
    return {
      locale,
      t: (key, values) => translate(locale, key, values),
      formatDate: (input, options) => {
        const value = typeof input === "string" && /^\d{4}-\d{2}-\d{2}$/.test(input) ? `${input}T00:00:00` : input;
        return new Intl.DateTimeFormat(intlLocale, options).format(new Date(value));
      },
      formatNumber: (input, options) => new Intl.NumberFormat(intlLocale, options).format(input),
    };
  }, [locale]);
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const context = useContext(I18nContext);
  if (!context) throw new Error("useI18n must be used within I18nProvider");
  return context;
}
