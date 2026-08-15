use std::collections::BTreeSet;

use proexel_domain::{
    ItemCategory, ItemCategorySnapshot, MachineItem, MachineItemSnapshot, PhotoAsset,
    PhotoOwnerType,
};

pub(crate) fn item_snapshot(
    item: &MachineItem,
    category: &ItemCategory,
    photos: &[PhotoAsset],
) -> MachineItemSnapshot {
    let step_ids = category
        .maintenance_guide
        .steps
        .iter()
        .map(|step| step.id.as_str())
        .collect::<BTreeSet<_>>();
    MachineItemSnapshot {
        id: item.id.clone(),
        machine_id: item.machine_id.clone(),
        category: ItemCategorySnapshot {
            id: category.id.clone(),
            code: category.code.clone(),
            name: category.name.clone(),
            guide_version: category.maintenance_guide.version,
            maintenance_guide: category.maintenance_guide.clone(),
            guide_reference_photos: photos
                .iter()
                .filter(|photo| {
                    photo.owner_type == PhotoOwnerType::GuideStep
                        && step_ids.contains(photo.owner_id.as_str())
                        && category.maintenance_guide.steps.iter().any(|step| {
                            step.id == photo.owner_id
                                && step.reference_photo_ids.contains(&photo.id)
                        })
                })
                .cloned()
                .collect(),
        },
        name: item.name.clone(),
        code: item.code.clone(),
        complexity_level: item.complexity_level,
        location_description: item.location_description.clone(),
        installed_component: item.installed_component.clone(),
    }
}
