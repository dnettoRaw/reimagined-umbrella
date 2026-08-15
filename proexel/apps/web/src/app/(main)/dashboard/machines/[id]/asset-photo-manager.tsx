"use client";

import { useState } from "react";

import Image from "next/image";
import { useRouter } from "next/navigation";

import { Camera, Trash2 } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useI18n } from "@/lib/i18n/provider";
import type { PhotoAsset, PhotoOwnerType, PhotoPurpose } from "@/lib/proexel/types";

type AttachmentKind = "machine-photos" | "item-photos" | "guide-photos" | "inspection-photos" | "replacement-photos";

export function AssetPhotoManager({
  ownerType,
  ownerId,
  photos,
  kind,
  canEdit,
  defaultPurpose = "reference",
  onPhotoAdded,
  onPhotoRemoved,
}: {
  ownerType: PhotoOwnerType;
  ownerId: string;
  photos: PhotoAsset[];
  kind: AttachmentKind;
  canEdit: boolean;
  defaultPurpose?: PhotoPurpose;
  onPhotoAdded?: (photo: PhotoAsset) => void;
  onPhotoRemoved?: (photo: PhotoAsset) => void;
}) {
  const { t } = useI18n();
  const router = useRouter();
  const [pending, setPending] = useState(false);
  const [description, setDescription] = useState("");
  const [purpose, setPurpose] = useState<PhotoPurpose>(defaultPurpose);

  async function upload(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file || !["image/png", "image/jpeg", "image/webp"].includes(file.type) || file.size > 8 * 1024 * 1024) {
      toast.error(t("photos.invalid"));
      return;
    }
    setPending(true);
    let ref: string | undefined;
    try {
      const form = new FormData();
      form.set("kind", kind);
      form.set("file", file);
      const uploaded = await fetch("/api/proexel/attachments", { method: "POST", body: form });
      const uploadResult = (await uploaded.json()) as { ref?: string };
      if (!uploaded.ok || !uploadResult.ref) throw new Error(t("photos.invalid"));
      ref = uploadResult.ref;
      const response = await fetch("/api/proexel/photos", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          owner_type: ownerType,
          owner_id: ownerId,
          purpose,
          blob_ref: ref,
          description: description || null,
        }),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string; resource_id?: string };
      if (!response.ok || !result.accepted || !result.resource_id)
        throw new Error(result.message ?? t("command.rejected"));
      onPhotoAdded?.({
        id: result.resource_id,
        owner_type: ownerType,
        owner_id: ownerId,
        purpose,
        blob_ref: ref,
        description: description || null,
        created_by: "",
        created_at_ms: Date.now(),
      });
      setDescription("");
      toast.success(t("command.success"));
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t("command.failed"));
    } finally {
      setPending(false);
      event.target.value = "";
    }
  }

  async function remove(photo: PhotoAsset) {
    setPending(true);
    try {
      const response = await fetch("/api/proexel/photos", {
        method: "DELETE",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ id: photo.id, blob_ref: photo.blob_ref }),
      });
      const result = (await response.json()) as { accepted?: boolean; message?: string };
      if (!response.ok || !result.accepted) throw new Error(result.message ?? t("command.rejected"));
      onPhotoRemoved?.(photo);
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t("command.failed"));
    } finally {
      setPending(false);
    }
  }

  return (
    <div className="space-y-3">
      {canEdit ? (
        <div className="grid gap-2 sm:grid-cols-[160px_1fr_auto]">
          <select
            className="h-9 rounded-md border bg-background px-3 text-sm"
            value={purpose}
            onChange={(event) => setPurpose(event.target.value as PhotoPurpose)}
          >
            {(["main", "general", "reference", "before", "during", "after", "defect", "evidence"] as const).map(
              (value) => (
                <option key={value} value={value}>
                  {t(`photos.${value}`)}
                </option>
              ),
            )}
          </select>
          <Input
            value={description}
            onChange={(event) => setDescription(event.target.value)}
            placeholder={t("photos.description")}
          />
          <Button asChild variant="outline" disabled={pending}>
            <label className="cursor-pointer">
              <Camera />
              {t("photos.add")}
              <input
                className="sr-only"
                type="file"
                accept="image/png,image/jpeg,image/webp"
                onChange={upload}
                disabled={pending}
              />
            </label>
          </Button>
        </div>
      ) : null}
      {photos.length ? (
        <div className="grid grid-cols-2 gap-2 md:grid-cols-4">
          {photos.map((photo) => (
            <figure key={photo.id} className="group relative overflow-hidden rounded-md border bg-muted">
              <Image
                unoptimized
                src={`/api/proexel/attachments?ref=${encodeURIComponent(photo.blob_ref)}`}
                alt={photo.description ?? t("common.photo")}
                width={480}
                height={320}
                className="aspect-[3/2] w-full object-cover"
              />
              <figcaption className="p-2 text-xs">
                <strong>{t(`photos.${photo.purpose}`)}</strong>
                {photo.description ? (
                  <span className="mt-1 block text-muted-foreground">{photo.description}</span>
                ) : null}
              </figcaption>
              {canEdit ? (
                <Button
                  className="absolute top-1 right-1 opacity-0 group-hover:opacity-100"
                  size="icon-sm"
                  variant="destructive"
                  title={t("common.remove")}
                  onClick={() => remove(photo)}
                >
                  <Trash2 />
                </Button>
              ) : null}
            </figure>
          ))}
        </div>
      ) : null}
    </div>
  );
}
