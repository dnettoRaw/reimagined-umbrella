import { cookies } from "next/headers";

import { DEFAULT_LOCALE, isLocale, LOCALE_COOKIE } from "./config";
import { type TranslationKey, type TranslationValues, translate } from "./messages";

export async function getLocale() {
  const cookieStore = await cookies();
  const value = cookieStore.get(LOCALE_COOKIE)?.value;
  return isLocale(value) ? value : DEFAULT_LOCALE;
}

export async function getI18n() {
  const locale = await getLocale();
  return {
    locale,
    t: (key: TranslationKey, values?: TranslationValues) => translate(locale, key, values),
  };
}
