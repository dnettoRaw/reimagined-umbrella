"use client";

import { useRef, useState } from "react";

import Image from "next/image";
import { useRouter } from "next/navigation";

import { Camera, Loader2, Trash2, Upload } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { useI18n } from "@/lib/i18n/provider";
import type { ValvePhoto } from "@/lib/proexel/types";

export function PhotoManager({
  valveId,
  valveTag,
  photos,
  editable,
}: {
  valveId: string;
  valveTag: string;
  photos: ValvePhoto[];
  editable: boolean;
}) {
  const { t } = useI18n();
  const router = useRouter();
  const input = useRef<HTMLInputElement>(null);
  const [pending, setPending] = useState(false);

  async function upload(file: File | undefined) {
    if (!file || !["image/png", "image/jpeg", "image/webp"].includes(file.type) || file.size > 5 * 1024 * 1024) {
      if (file) toast.error(t("valves.invalidPhoto"));
      return;
    }
    setPending(true);
    let blobRef: string | undefined;
    try {
      const form = new FormData();
      form.set("kind", "valve-photos");
      form.set("file", file);
      const uploaded = await fetch("/api/proexel/attachments", { method: "POST", body: form });
      const uploadBody = (await uploaded.json()) as { ref?: string };
      if (!uploaded.ok || !uploadBody.ref) throw new Error(t("valves.invalidPhoto"));
      blobRef = uploadBody.ref;
      const associated = await fetch("/api/proexel/valves/photos", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ valve_id: valveId, blob_ref: blobRef }),
      });
      const result = (await associated.json()) as { accepted?: boolean; message?: string };
      if (!associated.ok || !result.accepted) throw new Error(result.message ?? t("command.rejected"));
      toast.success(t("valves.photoUploaded"));
      router.refresh();
    } catch (error) {
      if (blobRef) {
        await fetch("/api/proexel/attachments", {
          method: "DELETE",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ ref: blobRef }),
        });
      }
      toast.error(error instanceof Error ? error.message : t("command.failed"));
    } finally {
      setPending(false);
      if (input.current) input.current.value = "";
    }
  }

  async function remove(photo: ValvePhoto) {
    if (!window.confirm(t("valves.confirmRemovePhoto"))) return;
    setPending(true);
    try {
      const response = await fetch("/api/proexel/valves/photos", {
        method: "DELETE",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ id: photo.id }),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string };
      if (!response.ok || !result.accepted) throw new Error(result.message ?? t("command.rejected"));
      await fetch("/api/proexel/attachments", {
        method: "DELETE",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ ref: photo.blob_ref }),
      });
      toast.success(t("valves.photoRemoved"));
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t("command.failed"));
    } finally {
      setPending(false);
    }
  }

  return (
    <div className="space-y-3">
      {editable ? (
        <div className="flex flex-wrap items-center gap-3">
          <input
            ref={input}
            className="sr-only"
            type="file"
            accept="image/png,image/jpeg,image/webp"
            onChange={(event) => upload(event.target.files?.[0])}
          />
          <Button type="button" variant="outline" disabled={pending} onClick={() => input.current?.click()}>
            {pending ? <Loader2 className="animate-spin" /> : <Upload />}
            {t("valves.uploadPhoto")}
          </Button>
          <span className="text-muted-foreground text-xs">{t("valves.photoHint")}</span>
        </div>
      ) : null}
      {photos.length ? (
        <div className="grid grid-cols-2 gap-3">
          {photos.map((photo) => (
            <div key={photo.id} className="group relative overflow-hidden rounded-md border">
              <Image
                src={`/api/proexel/attachments?ref=${encodeURIComponent(photo.blob_ref)}`}
                alt={`${valveTag} - ${photo.id}`}
                width={640}
                height={480}
                unoptimized
                className="aspect-4/3 w-full object-cover"
              />
              {editable ? (
                <Button
                  type="button"
                  size="icon-sm"
                  variant="destructive"
                  className="absolute top-2 right-2"
                  title={t("valves.removePhoto")}
                  disabled={pending}
                  onClick={() => remove(photo)}
                >
                  <Trash2 />
                  <span className="sr-only">{t("valves.removePhoto")}</span>
                </Button>
              ) : null}
            </div>
          ))}
        </div>
      ) : (
        <div className="flex min-h-32 items-center justify-center text-muted-foreground text-sm">
          <Camera className="mr-2 size-5" />
          {t("valves.noPhotos")}
        </div>
      )}
    </div>
  );
}
