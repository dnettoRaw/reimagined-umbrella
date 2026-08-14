import { NextResponse } from "next/server";

import { executeCommand, listValves, ProexelServiceError } from "@/lib/proexel/service";

export async function GET() {
  const result = await listValves();
  return NextResponse.json(result);
}

export async function POST(request: Request) {
  try {
    const data = (await request.json()) as Record<string, unknown>;
    return NextResponse.json(await executeCommand("proexel.valves.create", data));
  } catch (error) {
    const serviceError =
      error instanceof ProexelServiceError ? error : new ProexelServiceError("Dados inválidos.", 400);
    return NextResponse.json({ accepted: false, message: serviceError.message }, { status: serviceError.status });
  }
}
