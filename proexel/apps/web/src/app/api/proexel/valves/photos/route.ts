import { commandResponse } from "@/lib/proexel/http";

export async function POST(request: Request) {
  return commandResponse("proexel.valves.add_photo", request);
}

export async function DELETE(request: Request) {
  return commandResponse("proexel.valves.delete_photo", request);
}
