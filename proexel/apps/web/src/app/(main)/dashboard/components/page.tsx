import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { listItemCategories, listMachines } from "@/lib/proexel/service";

import { PageHeader } from "../_components/page-header";
import { ComponentRegistry } from "./component-registry";

export const dynamic = "force-dynamic";

export default async function ComponentsPage() {
  const [{ role }, machines, categories, { t }] = await Promise.all([
    requirePermission("machine.read"),
    listMachines({ page: 1, page_size: 500 }),
    listItemCategories(),
    getI18n(),
  ]);
  const entries = machines.items.flatMap((machine) =>
    machine.items
      .filter((item) => item.active)
      .map((item) => ({
        item,
        machine: {
          id: machine.id,
          code: machine.code,
          name: machine.name,
          zone: machine.zone,
          location: machine.location,
        },
      })),
  );
  return (
    <div>
      <PageHeader title={t("nav.components")} description={t("components.description")} />
      <ComponentRegistry entries={entries} categories={categories.items} canManage={can("machine_item.manage", role)} />
    </div>
  );
}
