import { NextResponse } from "next/server";

import { commandResponse } from "@/lib/proexel/http";
import { listRestockRequests } from "@/lib/proexel/service";

export async function GET() {
  return NextResponse.json(await listRestockRequests());
}

export async function POST(request: Request) {
  return commandResponse("proexel.purchasing.create_restock_request", request);
}

export async function PATCH(request: Request) {
  return commandResponse("proexel.purchasing.review_restock_request", request);
}

export async function DELETE(request: Request) {
  return commandResponse("proexel.purchasing.delete_restock_request", request);
}
