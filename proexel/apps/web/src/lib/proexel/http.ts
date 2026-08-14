import { NextResponse } from "next/server";

import { executeCommand, ProexelServiceError } from "./service";

export async function commandResponse(capability: string, request: Request) {
  try {
    const data = (await request.json()) as Record<string, unknown>;
    return NextResponse.json(await executeCommand(capability, data));
  } catch (error) {
    const serviceError =
      error instanceof ProexelServiceError ? error : new ProexelServiceError("Dados inválidos.", 400);
    return NextResponse.json({ accepted: false, message: serviceError.message }, { status: serviceError.status });
  }
}
