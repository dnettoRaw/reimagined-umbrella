import { NextResponse } from "next/server";

import { getOverview } from "@/lib/proexel/service";

export async function GET() {
  return NextResponse.json(await getOverview());
}
