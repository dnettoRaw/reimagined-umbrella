use proexel_domain::{PhotoAsset, PhotoOwnerType, PhotoPurpose};
use serde::Deserialize;

use crate::{
    state::{
        clean_optional, domain_action, json_string, parse_data, require_permission, require_text,
        Action, CommandPayload,
    },
    ApplicationState,
};

impl ApplicationState {
    pub(crate) fn add_photo(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        let input: AddPhoto = parse_data(payload)?;
        let permission = self.photo_permission(&input.owner_type, &input.owner_id, payload)?;
        require_permission(payload.actor.role, permission)?;
        require_text(&input.blob_ref, "photo_ref_required")?;
        if self
            .photos
            .iter()
            .any(|photo| photo.blob_ref == input.blob_ref)
        {
            return Err("photo_already_exists".to_string());
        }
        let id = format!("photo-{command_id}");
        let photo = PhotoAsset {
            id: id.clone(),
            owner_type: input.owner_type,
            owner_id: input.owner_id,
            purpose: input.purpose,
            blob_ref: input.blob_ref.trim().to_string(),
            description: clean_optional(input.description),
            created_by: payload.actor.name.clone(),
            created_at_ms: now,
        };
        let after = json_string(&photo);
        if photo.owner_type == PhotoOwnerType::Machine && photo.purpose == PhotoPurpose::Main {
            if let Some(machine) = self
                .machines
                .iter_mut()
                .find(|machine| machine.id == photo.owner_id)
            {
                machine.main_photo_id = Some(id.clone());
                machine.updated_at_ms = now;
            }
        }
        if photo.owner_type == PhotoOwnerType::GuideStep {
            for category in &mut self.item_categories {
                if let Some(step) = category
                    .maintenance_guide
                    .steps
                    .iter_mut()
                    .find(|step| step.id == photo.owner_id)
                {
                    if !step.reference_photo_ids.contains(&id) {
                        step.reference_photo_ids.push(id.clone());
                        category.maintenance_guide.version =
                            category.maintenance_guide.version.saturating_add(1);
                    }
                    category.updated_at_ms = now;
                    break;
                }
            }
        }
        self.photos.push(photo);
        Ok(domain_action(
            permission,
            "photo.added",
            "photo",
            id,
            None,
            after,
            "Photo added",
        ))
    }

    pub(crate) fn delete_photo(&mut self, payload: &CommandPayload) -> Result<Action, String> {
        let input: DeletePhoto = parse_data(payload)?;
        let index = self
            .photos
            .iter()
            .position(|photo| photo.id == input.id)
            .ok_or_else(|| "photo_not_found".to_string())?;
        let photo = self.photos[index].clone();
        if input.blob_ref != photo.blob_ref {
            return Err("photo_ref_mismatch".to_string());
        }
        let permission = self.photo_permission(&photo.owner_type, &photo.owner_id, payload)?;
        require_permission(payload.actor.role, permission)?;
        if self.service_orders.iter().any(|order| {
            order.tasks.iter().any(|task| {
                task.item_snapshot
                    .category
                    .guide_reference_photos
                    .iter()
                    .any(|snapshot| snapshot.id == photo.id)
            })
        }) {
            return Err("photo_in_use_by_service_order".to_string());
        }
        self.photos.remove(index);
        for machine in &mut self.machines {
            if machine.main_photo_id.as_deref() == Some(&photo.id) {
                machine.main_photo_id = None;
            }
        }
        for category in &mut self.item_categories {
            let mut changed = false;
            for step in &mut category.maintenance_guide.steps {
                let previous_len = step.reference_photo_ids.len();
                step.reference_photo_ids.retain(|id| id != &photo.id);
                changed |= step.reference_photo_ids.len() != previous_len;
            }
            if changed {
                category.maintenance_guide.version =
                    category.maintenance_guide.version.saturating_add(1);
            }
        }
        Ok(domain_action(
            permission,
            "photo.removed",
            "photo",
            photo.id.clone(),
            json_string(&photo),
            None,
            "Photo removed",
        ))
    }
}

#[derive(Deserialize)]
struct AddPhoto {
    owner_type: PhotoOwnerType,
    owner_id: String,
    purpose: PhotoPurpose,
    blob_ref: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct DeletePhoto {
    id: String,
    blob_ref: String,
}
