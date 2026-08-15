import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { listItemCategories } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";
import { CategoryEditor } from "./category-editor";

export const dynamic = "force-dynamic";

export default async function CategoriesPage() {
  await requirePermission("item_category.manage");
  const [categories, { t }] = await Promise.all([listItemCategories(), getI18n()]);
  return (
    <div>
      <PageHeader title={t("nav.categories")} description={t("categories.description")} />
      <CategoryEditor categories={categories.items} />
    </div>
  );
}
