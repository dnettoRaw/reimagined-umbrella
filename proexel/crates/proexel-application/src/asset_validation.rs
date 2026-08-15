use std::collections::{BTreeMap, BTreeSet};

use proexel_domain::{
    CustomFieldDefinition, CustomFieldType, GuideStepType, InspectionStepResult, ItemCategory,
    MaintenanceGuide, PhotoAsset, PhotoOwnerType,
};
use serde_json::Value;

use crate::state::require_text;

pub(crate) fn normalized_fields(
    mut fields: Vec<CustomFieldDefinition>,
) -> Vec<CustomFieldDefinition> {
    fields.sort_by_key(|field| field.order);
    fields
}

pub(crate) fn normalized_guide(mut guide: MaintenanceGuide) -> MaintenanceGuide {
    guide.steps.sort_by_key(|step| step.order);
    guide
}

pub(crate) fn validate_category_definition(
    fields: &[CustomFieldDefinition],
    guide: &MaintenanceGuide,
) -> Result<(), String> {
    let mut field_ids = BTreeSet::new();
    let mut field_keys = BTreeSet::new();
    for field in fields {
        require_text(&field.id, "custom_field_id_required")?;
        require_text(&field.key, "custom_field_key_required")?;
        require_text(&field.label, "custom_field_label_required")?;
        if !field_ids.insert(field.id.trim().to_string())
            || !field_keys.insert(field.key.trim().to_lowercase())
        {
            return Err("custom_field_duplicate".to_string());
        }
        if field.field_type == CustomFieldType::Choice && field.options.is_empty() {
            return Err("custom_field_choice_options_required".to_string());
        }
        if field
            .minimum
            .zip(field.maximum)
            .is_some_and(|(minimum, maximum)| minimum > maximum)
        {
            return Err("custom_field_range_invalid".to_string());
        }
    }
    let mut step_ids = BTreeSet::new();
    for step in &guide.steps {
        require_text(&step.id, "guide_step_id_required")?;
        require_text(&step.title, "guide_step_title_required")?;
        require_text(&step.instructions, "guide_step_instructions_required")?;
        if !step_ids.insert(step.id.trim().to_string()) {
            return Err("guide_step_duplicate".to_string());
        }
        if step.step_type == GuideStepType::Choice && step.options.is_empty() {
            return Err("guide_step_choice_options_required".to_string());
        }
        if matches!(
            step.step_type,
            GuideStepType::Numeric | GuideStepType::Measurement
        ) && step.expected_value.as_ref().is_some_and(|expected| {
            expected
                .minimum
                .zip(expected.maximum)
                .is_some_and(|(minimum, maximum)| minimum > maximum)
        }) {
            return Err("guide_step_expected_range_invalid".to_string());
        }
    }
    Ok(())
}

pub(crate) fn validate_custom_values(
    category: &ItemCategory,
    values: &BTreeMap<String, Value>,
) -> Result<(), String> {
    let definitions = category
        .custom_field_definitions
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<BTreeMap<_, _>>();
    if values
        .keys()
        .any(|key| !definitions.contains_key(key.as_str()))
    {
        return Err("custom_field_unknown".to_string());
    }
    for field in &category.custom_field_definitions {
        let value = values.get(&field.key);
        if field.required && value.is_none_or(is_empty_value) {
            return Err(format!("custom_field_required:{}", field.key));
        }
        let Some(value) = value.filter(|value| !is_empty_value(value)) else {
            continue;
        };
        let valid = match field.field_type {
            CustomFieldType::Text | CustomFieldType::Date => value.as_str().is_some(),
            CustomFieldType::Boolean => value.as_bool().is_some(),
            CustomFieldType::Choice => value
                .as_str()
                .is_some_and(|selected| field.options.iter().any(|option| option == selected)),
            CustomFieldType::Number => value.as_f64().is_some_and(|number| {
                field.minimum.is_none_or(|minimum| number >= minimum)
                    && field.maximum.is_none_or(|maximum| number <= maximum)
            }),
        };
        if !valid {
            return Err(format!("custom_field_value_invalid:{}", field.key));
        }
    }
    Ok(())
}

pub(crate) fn is_empty_value(value: &Value) -> bool {
    value.is_null() || value.as_str().is_some_and(|text| text.trim().is_empty())
}

pub(crate) fn validate_step_results(
    guide: &MaintenanceGuide,
    results: &[InspectionStepResult],
    photos: &[PhotoAsset],
    inspection_id: &str,
) -> Result<(), String> {
    let result_map = results
        .iter()
        .map(|result| (result.step_id.as_str(), result))
        .collect::<BTreeMap<_, _>>();
    if result_map.len() != results.len() {
        return Err("inspection_step_result_duplicate".to_string());
    }
    if results
        .iter()
        .any(|result| !guide.steps.iter().any(|step| step.id == result.step_id))
    {
        return Err("inspection_step_unknown".to_string());
    }
    for step in &guide.steps {
        let result = result_map.get(step.id.as_str()).copied();
        if step.required && result.is_none() {
            return Err(format!("inspection_step_required:{}", step.id));
        }
        let Some(result) = result else {
            continue;
        };
        let valid = match step.step_type {
            GuideStepType::Information | GuideStepType::Warning => true,
            GuideStepType::Confirmation | GuideStepType::Boolean => {
                result.value.as_bool().is_some()
            }
            GuideStepType::Choice => result
                .value
                .as_str()
                .is_some_and(|selected| step.options.iter().any(|option| option == selected)),
            GuideStepType::Numeric | GuideStepType::Measurement => result.value.as_f64().is_some(),
            GuideStepType::Text => result
                .value
                .as_str()
                .is_some_and(|text| !text.trim().is_empty()),
            GuideStepType::Photo => !result.photo_ids.is_empty(),
        };
        if !valid {
            return Err(format!("inspection_step_value_invalid:{}", step.id));
        }
        if step.step_type == GuideStepType::Measurement {
            let expected_unit = step
                .expected_value
                .as_ref()
                .and_then(|expected| expected.unit.as_deref());
            if expected_unit != result.unit.as_deref() {
                return Err(format!("inspection_measurement_unit_invalid:{}", step.id));
            }
        }
        for photo_id in &result.photo_ids {
            if !photos.iter().any(|photo| {
                photo.id == *photo_id
                    && photo.owner_type == PhotoOwnerType::Inspection
                    && photo.owner_id == inspection_id
            }) {
                return Err(format!("inspection_photo_invalid:{}", step.id));
            }
        }
    }
    Ok(())
}

pub(crate) fn default_true() -> bool {
    true
}
