import { NextResponse } from "next/server";

import { commandResponse } from "@/lib/proexel/http";
import { listMaintenance } from "@/lib/proexel/service";

export async function GET() {
  return NextResponse.json(await listMaintenance());
}

export async function POST(request: Request) {
  return commandResponse("proexel.maintenance.register", request);
}
