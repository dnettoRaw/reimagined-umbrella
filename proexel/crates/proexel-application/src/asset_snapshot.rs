use std::collections::BTreeSet;

use proexel_domain::{
    GuideStepType, ItemCategory, ItemCategorySnapshot, MachineItem, MachineItemSnapshot,
    MaintenanceGuide, MaintenanceGuideStep, PhotoAsset, PhotoOwnerType,
};

pub const SAFETY_STEP_ID: &str = "proexel-safety-lockout";
pub const BEFORE_PHOTO_STEP_ID: &str = "proexel-visual-before";
pub const AFTER_PHOTO_STEP_ID: &str = "proexel-work-after";

pub(crate) fn item_snapshot(
    item: &MachineItem,
    category: &ItemCategory,
    photos: &[PhotoAsset],
) -> MachineItemSnapshot {
    let source_guide = item
        .maintenance_guide_override
        .as_ref()
        .unwrap_or(&category.maintenance_guide);
    let maintenance_guide = operational_guide(source_guide);
    let step_ids = maintenance_guide
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
            guide_version: maintenance_guide.version,
            maintenance_guide: maintenance_guide.clone(),
            guide_reference_photos: photos
                .iter()
                .filter(|photo| {
                    photo.owner_type == PhotoOwnerType::GuideStep
                        && step_ids.contains(photo.owner_id.as_str())
                        && maintenance_guide.steps.iter().any(|step| {
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
        installed_component: item.installed_component.clone(),
    }
}

pub(crate) fn operational_guide(source: &MaintenanceGuide) -> MaintenanceGuide {
    let mut steps = vec![standard_step(
        SAFETY_STEP_ID,
        "Secure the machine",
        "Apply lockout and tagout, isolate every energy source and confirm zero energy before intervention.",
        GuideStepType::Confirmation,
        Some("Do not proceed until the machine is safely isolated."),
    )];
    steps.push(standard_step(
        BEFORE_PHOTO_STEP_ID,
        "Visual inspection and initial photo",
        "Inspect the component and take a clear photo showing its condition before maintenance.",
        GuideStepType::Photo,
        None,
    ));
    steps.extend(
        source
            .steps
            .iter()
            .filter(|step| {
                !matches!(
                    step.id.as_str(),
                    SAFETY_STEP_ID | BEFORE_PHOTO_STEP_ID | AFTER_PHOTO_STEP_ID
                )
            })
            .cloned(),
    );
    steps.push(standard_step(
        AFTER_PHOTO_STEP_ID,
        "Photo of completed work",
        "Take a clear photo of the completed maintenance before concluding this component.",
        GuideStepType::Photo,
        None,
    ));
    for (order, step) in steps.iter_mut().enumerate() {
        step.order = order as u32;
    }
    MaintenanceGuide {
        version: source.version,
        steps,
    }
}

fn standard_step(
    id: &str,
    title: &str,
    instructions: &str,
    step_type: GuideStepType,
    safety_warning: Option<&str>,
) -> MaintenanceGuideStep {
    MaintenanceGuideStep {
        id: id.to_string(),
        title: title.to_string(),
        description: None,
        instructions: instructions.to_string(),
        step_type,
        required: true,
        reference_photo_ids: Vec::new(),
        safety_warning: safety_warning.map(str::to_string),
        expected_value: None,
        options: Vec::new(),
        order: 0,
    }
}
