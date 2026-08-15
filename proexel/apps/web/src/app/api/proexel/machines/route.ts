import { NextResponse } from "next/server";

import { commandResponse } from "@/lib/proexel/http";
import { listMachines } from "@/lib/proexel/service";

export async function GET(request: Request) {
  const params = new URL(request.url).searchParams;
  return NextResponse.json(
    await listMachines({
      id: params.get("id") ?? "",
      search: params.get("search") ?? "",
      zone: params.get("zone") ?? "",
      status: params.get("status") ?? "",
      include_removed: params.get("include_removed") === "true",
      page: Number(params.get("page") ?? 1),
      page_size: Number(params.get("page_size") ?? 25),
    }),
  );
}

export async function POST(request: Request) {
  return commandResponse("proexel.machines.create", request);
}

export async function PATCH(request: Request) {
  return commandResponse("proexel.machines.update", request);
}
