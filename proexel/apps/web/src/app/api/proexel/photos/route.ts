import { NextResponse } from "next/server";

import { getI18n } from "@/lib/i18n/server";
import { deleteAttachment } from "@/lib/proexel/attachment-storage";
import { commandResponse } from "@/lib/proexel/http";
import { executeCommand, ProexelServiceError } from "@/lib/proexel/service";

export async function POST(request: Request) {
  return commandResponse("proexel.photos.add", request);
}

export async function DELETE(request: Request) {
  try {
    const data = (await request.json()) as Record<string, unknown>;
    const result = await executeCommand("proexel.photos.delete", data);
    if (typeof data.blob_ref === "string") await deleteAttachment(data.blob_ref);
    return NextResponse.json(result);
  } catch (error) {
    const { t } = await getI18n();
    const serviceError =
      error instanceof ProexelServiceError ? error : new ProexelServiceError(t("service.invalidData"), 400);
    return NextResponse.json({ accepted: false, message: serviceError.message }, { status: serviceError.status });
  }
}
