import { NextResponse } from "next/server";

import { getReports } from "@/lib/proexel/service";

export async function GET() {
  return NextResponse.json(await getReports());
}
