import { NextResponse } from "next/server";

import { listAudit } from "@/lib/proexel/service";

export async function GET() {
  return NextResponse.json(await listAudit());
}
