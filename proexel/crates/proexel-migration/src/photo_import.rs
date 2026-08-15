use proexel_application::ApplicationState;
use proexel_domain::{normalize_tag, PhotoAsset, PhotoOwnerType, PhotoPurpose};

use crate::{
    migration_support::{find_item_by_tag, hash},
    report::inc,
    LegacyBundle, MigrationReport,
};

pub(crate) fn import_photos(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.valve_photos {
        let tag = normalize_tag(&old.tag);
        let Some(item) = find_item_by_tag(state, &tag) else {
            report
                .warnings
                .push(format!("photo skipped: item {tag} not found"));
            continue;
        };
        let owner_id = item.id.clone();
        let id = format!("legacy-photo-{}", hash(&format!("{tag}|{}", old.blob_ref)));
        if state.photos.iter().any(|photo| photo.id == id) {
            continue;
        }
        state.photos.push(PhotoAsset {
            id,
            owner_type: PhotoOwnerType::MachineItem,
            owner_id,
            purpose: PhotoPurpose::Reference,
            blob_ref: old.blob_ref.clone(),
            description: Some("Legacy reference photo".to_string()),
            created_by: "migration-tool".to_string(),
            created_at_ms: now,
        });
        inc(report, "photos");
    }
}
