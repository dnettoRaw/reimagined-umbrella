import { NextResponse } from "next/server";

import { commandResponse } from "@/lib/proexel/http";
import { listItemCategories } from "@/lib/proexel/service";

export async function GET(request: Request) {
  const search = new URL(request.url).searchParams.get("search") ?? "";
  return NextResponse.json(await listItemCategories({ search }));
}

export async function POST(request: Request) {
  return commandResponse("proexel.item_categories.create", request);
}

export async function PATCH(request: Request) {
  return commandResponse("proexel.item_categories.update", request);
}
