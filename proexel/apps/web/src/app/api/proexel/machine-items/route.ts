import { commandResponse } from "@/lib/proexel/http";

export async function POST(request: Request) {
  return commandResponse("proexel.machine_items.add", request);
}

export async function PATCH(request: Request) {
  return commandResponse("proexel.machine_items.update", request);
}

export async function PUT(request: Request) {
  const body = (await request.json().catch(() => ({}))) as Record<string, unknown>;
  const capability = body.action === "reorder" ? "proexel.machine_items.reorder" : "proexel.machine_items.replace";
  return commandResponse(capability, new Request(request.url, { method: "PUT", body: JSON.stringify(body) }));
}

export async function DELETE(request: Request) {
  return commandResponse("proexel.machine_items.remove", request);
}
