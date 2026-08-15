use proexel_domain::{
    normalize_identifier, ComplexityLevel, CustomFieldDefinition, ItemCategory, MaintenanceGuide,
    RecommendedPart,
};
use serde::Deserialize;

use crate::{
    state::{
        clean_optional, domain_action, json_string, parse_data, require_permission, require_text,
        Action, CommandPayload,
    },
    ApplicationState,
};

use crate::asset_audit::SupplementalAudit;
use crate::asset_validation::{
    default_true, normalized_fields, normalized_guide, validate_category_definition,
};

impl ApplicationState {
    pub(crate) fn create_item_category(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "item_category.manage")?;
        let input: CategoryInput = parse_data(payload)?;
        let code = normalize_identifier(&input.code);
        require_text(&code, "category_code_required")?;
        require_text(&input.name, "category_name_required")?;
        if self
            .item_categories
            .iter()
            .any(|category| category.code_normalized == code)
        {
            return Err("category_code_already_exists".to_string());
        }
        let mut guide = input.maintenance_guide;
        guide.version = 1;
        validate_category_definition(&input.custom_field_definitions, &guide)?;
        let id = format!("category-{command_id}");
        let category = ItemCategory {
            id: id.clone(),
            code: code.clone(),
            code_normalized: code,
            name: input.name.trim().to_string(),
            description: clean_optional(input.description),
            icon: clean_optional(input.icon),
            default_complexity_level: input.default_complexity_level,
            maintenance_guide: normalized_guide(guide),
            custom_field_definitions: normalized_fields(input.custom_field_definitions),
            recommended_parts: input.recommended_parts,
            active: input.active,
            created_at_ms: now,
            updated_at_ms: now,
        };
        let after = json_string(&category);
        self.item_categories.push(category);
        Ok(domain_action(
            "item_category.manage",
            "item_category.created",
            "item_category",
            id,
            None,
            after,
            "Item category created",
        ))
    }

    pub(crate) fn update_item_category(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "item_category.manage")?;
        let input: UpdateCategory = parse_data(payload)?;
        let code = normalize_identifier(&input.category.code);
        require_text(&code, "category_code_required")?;
        require_text(&input.category.name, "category_name_required")?;
        let index = self
            .item_categories
            .iter()
            .position(|category| category.id == input.id)
            .ok_or_else(|| "category_not_found".to_string())?;
        if self
            .item_categories
            .iter()
            .any(|category| category.id != input.id && category.code_normalized == code)
        {
            return Err("category_code_already_exists".to_string());
        }
        validate_category_definition(
            &input.category.custom_field_definitions,
            &input.category.maintenance_guide,
        )?;
        let before = json_string(&self.item_categories[index]);
        let previous_guide = self.item_categories[index].maintenance_guide.clone();
        let guide_changed = previous_guide.steps != input.category.maintenance_guide.steps;
        let version = if guide_changed {
            previous_guide.version.saturating_add(1)
        } else {
            previous_guide.version
        };
        let category = &mut self.item_categories[index];
        category.code = code.clone();
        category.code_normalized = code;
        category.name = input.category.name.trim().to_string();
        category.description = clean_optional(input.category.description);
        category.icon = clean_optional(input.category.icon);
        category.default_complexity_level = input.category.default_complexity_level;
        category.maintenance_guide = normalized_guide(MaintenanceGuide {
            version,
            steps: input.category.maintenance_guide.steps,
        });
        category.custom_field_definitions =
            normalized_fields(input.category.custom_field_definitions);
        category.recommended_parts = input.category.recommended_parts;
        category.active = input.category.active;
        category.updated_at_ms = now;
        let guide_after = json_string(&category.maintenance_guide);
        let after = json_string(category);
        if guide_changed {
            self.push_supplemental_audit(
                payload,
                now,
                SupplementalAudit {
                    operation: "maintenance_guide.updated",
                    aggregate: "item_category",
                    aggregate_id: &input.id,
                    before: json_string(&previous_guide),
                    after: guide_after,
                    description: "Maintenance guide updated",
                },
            );
        }
        Ok(domain_action(
            "item_category.manage",
            "item_category.updated",
            "item_category",
            input.id,
            before,
            after,
            "Item category updated",
        ))
    }
}

#[derive(Deserialize)]
struct CategoryInput {
    code: String,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    default_complexity_level: ComplexityLevel,
    #[serde(default)]
    maintenance_guide: MaintenanceGuide,
    #[serde(default)]
    custom_field_definitions: Vec<CustomFieldDefinition>,
    #[serde(default)]
    recommended_parts: Vec<RecommendedPart>,
    #[serde(default = "default_true")]
    active: bool,
}

#[derive(Deserialize)]
struct UpdateCategory {
    id: String,
    #[serde(flatten)]
    category: CategoryInput,
}
