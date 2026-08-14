import { NextResponse } from "next/server";

import { commandResponse } from "@/lib/proexel/http";
import { listSuppliers } from "@/lib/proexel/service";

export async function GET() {
  return NextResponse.json(await listSuppliers());
}

export async function POST(request: Request) {
  return commandResponse("proexel.suppliers.create", request);
}

export async function PATCH(request: Request) {
  return commandResponse("proexel.suppliers.update", request);
}
