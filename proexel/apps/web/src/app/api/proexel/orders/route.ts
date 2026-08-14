import { NextResponse } from "next/server";

import { commandResponse } from "@/lib/proexel/http";
import { listServiceOrders } from "@/lib/proexel/service";

export async function GET() {
  return NextResponse.json(await listServiceOrders());
}

export async function POST(request: Request) {
  return commandResponse("proexel.orders.create", request);
}

export async function PATCH(request: Request) {
  return commandResponse("proexel.orders.change_status", request);
}
