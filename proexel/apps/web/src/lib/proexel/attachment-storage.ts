import { unlink } from "node:fs/promises";
import path from "node:path";

export function resolveAttachment(ref: string) {
  const root = path.resolve(
    /* turbopackIgnore: true */
    process.env.PROEXEL_ATTACHMENTS_DIR ?? path.join(process.cwd(), "../service/target/runtime/attachments"),
  );
  const target = path.resolve(root, ref);
  if (!target.startsWith(`${root}${path.sep}`)) throw new Error("invalid_attachment_path");
  return target;
}

export async function deleteAttachment(ref: string) {
  await unlink(resolveAttachment(ref)).catch(() => undefined);
}
