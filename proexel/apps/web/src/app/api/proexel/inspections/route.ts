import { NextResponse } from "next/server";

import { commandResponse } from "@/lib/proexel/http";
import { listInspections } from "@/lib/proexel/service";

export async function GET(request: Request) {
  const params = new URL(request.url).searchParams;
  return NextResponse.json(
    await listInspections({
      id: params.get("id") ?? "",
      service_order_id: params.get("service_order_id") ?? "",
      machine_id: params.get("machine_id") ?? "",
      machine_item_id: params.get("machine_item_id") ?? "",
      operator_id: params.get("operator_id") ?? "",
    }),
  );
}

export async function POST(request: Request) {
  return commandResponse("proexel.inspections.start", request);
}

export async function PATCH(request: Request) {
  return commandResponse("proexel.inspections.complete", request);
}
