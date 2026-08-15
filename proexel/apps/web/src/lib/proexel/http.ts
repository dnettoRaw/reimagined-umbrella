import { NextResponse } from "next/server";

import { getI18n } from "@/lib/i18n/server";

import { executeCommand, ProexelServiceError } from "./service";

export async function commandResponse(capability: string, request: Request) {
  try {
    const data = (await request.json()) as Record<string, unknown>;
    return NextResponse.json(await executeCommand(capability, data));
  } catch (error) {
    const { t } = await getI18n();
    const serviceError =
      error instanceof ProexelServiceError ? error : new ProexelServiceError(t("service.invalidData"), 400);
    return NextResponse.json({ accepted: false, message: serviceError.message }, { status: serviceError.status });
  }
}
