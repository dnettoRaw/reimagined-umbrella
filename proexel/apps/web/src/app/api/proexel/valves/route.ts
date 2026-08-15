import { NextResponse } from "next/server";

import { commandResponse } from "@/lib/proexel/http";
import { listValves } from "@/lib/proexel/service";

export async function GET() {
  const result = await listValves();
  return NextResponse.json(result);
}

export async function POST(request: Request) {
  return commandResponse("proexel.valves.create", request);
}

export async function PATCH(request: Request) {
  return commandResponse("proexel.valves.update", request);
}
