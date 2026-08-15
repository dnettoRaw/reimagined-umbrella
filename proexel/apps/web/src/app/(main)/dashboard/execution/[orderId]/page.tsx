import { notFound } from "next/navigation";

import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { listInspections, listMachines, listServiceOrders } from "@/lib/proexel/service";

import { PageHeader } from "../../_components/page-header";
import { InspectionRunner } from "./inspection-runner";

export const dynamic = "force-dynamic";

export default async function ExecutionDetailPage({ params }: { readonly params: Promise<{ orderId: string }> }) {
  const { orderId } = await params;
  const session = await requirePermission("inspection.execute");
  const [orders, inspections, { t }] = await Promise.all([
    listServiceOrders({ id: orderId }),
    listInspections({ service_order_id: orderId }),
    getI18n(),
  ]);
  const order = orders.items[0];
  if (!order) notFound();
  const machines = await listMachines({ id: order.machine_id, page_size: 1 });
  const machine = machines.items[0];
  return (
    <div>
      <PageHeader
        title={`${order.machine_snapshot.code} · ${order.machine_snapshot.name}`}
        description={t("execution.description")}
      />
      <InspectionRunner
        order={order}
        inspections={inspections.items}
        currentItems={machine?.items ?? []}
        session={{ id: session.sub, role: session.role, maximumRepairLevel: session.maximum_repair_level }}
      />
    </div>
  );
}
