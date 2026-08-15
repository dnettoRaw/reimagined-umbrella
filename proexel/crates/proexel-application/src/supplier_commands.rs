use proexel_domain::Supplier;
use serde::Deserialize;

use crate::{
    state::{
        action, clean_optional, json_string, parse_data, require_permission, require_text, Action,
        CommandPayload,
    },
    ApplicationState,
};

impl ApplicationState {
    pub(crate) fn create_supplier(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "supplier.create_update_delete")?;
        let input: SupplierInput = parse_data(payload)?;
        require_text(&input.name, "supplier_name_required")?;
        require_text(&input.contact, "supplier_contact_required")?;
        validate_supplier_links(input.email.as_deref(), input.website.as_deref())?;
        let id = format!("supplier-{command_id}");
        let supplier = Supplier {
            id: id.clone(),
            name: input.name.trim().to_string(),
            contact: input.contact.trim().to_string(),
            email: clean_optional(input.email),
            website: clean_optional(input.website),
            notes: clean_optional(input.notes),
            created_by: payload.actor.name.clone(),
            created_at_ms: now,
            updated_at_ms: now,
        };
        let after = json_string(&supplier);
        self.suppliers.push(supplier);
        Ok(action(
            "supplier.create_update_delete",
            "supplier",
            id,
            None,
            after,
            "Supplier created",
        ))
    }

    pub(crate) fn update_supplier(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "supplier.create_update_delete")?;
        let input: UpdateSupplier = parse_data(payload)?;
        require_text(&input.name, "supplier_name_required")?;
        require_text(&input.contact, "supplier_contact_required")?;
        validate_supplier_links(input.email.as_deref(), input.website.as_deref())?;
        let supplier = self
            .suppliers
            .iter_mut()
            .find(|supplier| supplier.id == input.id)
            .ok_or_else(|| "supplier_not_found".to_string())?;
        let before = json_string(supplier);
        supplier.name = input.name.trim().to_string();
        supplier.contact = input.contact.trim().to_string();
        supplier.email = clean_optional(input.email);
        supplier.website = clean_optional(input.website);
        supplier.notes = clean_optional(input.notes);
        supplier.updated_at_ms = now;
        let after = json_string(supplier);
        Ok(action(
            "supplier.create_update_delete",
            "supplier",
            input.id,
            before,
            after,
            "Supplier updated",
        ))
    }

    pub(crate) fn delete_supplier(&mut self, payload: &CommandPayload) -> Result<Action, String> {
        require_permission(payload.actor.role, "supplier.create_update_delete")?;
        let input: DeleteById = parse_data(payload)?;
        let index = self
            .suppliers
            .iter()
            .position(|supplier| supplier.id == input.id)
            .ok_or_else(|| "supplier_not_found".to_string())?;
        let supplier = self.suppliers.remove(index);
        Ok(action(
            "supplier.create_update_delete",
            "supplier",
            supplier.id.clone(),
            json_string(&supplier),
            None,
            "Supplier deleted",
        ))
    }
}

fn validate_supplier_links(email: Option<&str>, website: Option<&str>) -> Result<(), String> {
    if let Some(email) = email.map(str::trim).filter(|value| !value.is_empty()) {
        let (local, domain) = email
            .split_once('@')
            .ok_or_else(|| "supplier_email_invalid".to_string())?;
        if local.is_empty() || !domain.contains('.') || email.contains(char::is_whitespace) {
            return Err("supplier_email_invalid".to_string());
        }
    }
    if let Some(website) = website.map(str::trim).filter(|value| !value.is_empty()) {
        if website.contains(char::is_whitespace)
            || !(website.starts_with("https://") || website.starts_with("http://"))
        {
            return Err("supplier_website_invalid".to_string());
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct SupplierInput {
    name: String,
    contact: String,
    email: Option<String>,
    website: Option<String>,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct UpdateSupplier {
    id: String,
    name: String,
    contact: String,
    email: Option<String>,
    website: Option<String>,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct DeleteById {
    id: String,
}
