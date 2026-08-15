import { NextResponse } from "next/server";

import { commandResponse } from "@/lib/proexel/http";
import { listServiceOrders } from "@/lib/proexel/service";

export async function GET(request: Request) {
  const params = new URL(request.url).searchParams;
  return NextResponse.json(
    await listServiceOrders({
      id: params.get("id") ?? "",
      machine_id: params.get("machine_id") ?? "",
      status: params.get("status") ?? "",
      operator_id: params.get("operator_id") ?? "",
    }),
  );
}

export async function POST(request: Request) {
  return commandResponse("proexel.orders.create", request);
}

export async function PATCH(request: Request) {
  const body = (await request.json().catch(() => ({}))) as Record<string, unknown>;
  const capability = body.action === "assign" ? "proexel.orders.assign_task" : "proexel.orders.start";
  return commandResponse(capability, new Request(request.url, { method: "PATCH", body: JSON.stringify(body) }));
}

export async function PUT(request: Request) {
  return commandResponse("proexel.orders.complete", request);
}

export async function DELETE(request: Request) {
  return commandResponse("proexel.orders.delete", request);
}
