import { NextResponse } from "next/server";

import { commandResponse } from "@/lib/proexel/http";
import { listStock } from "@/lib/proexel/service";

export async function GET() {
  return NextResponse.json(await listStock());
}

export async function POST(request: Request) {
  return commandResponse("proexel.stock.upsert_item", request);
}

export async function PATCH(request: Request) {
  return commandResponse("proexel.stock.adjust", request);
}
