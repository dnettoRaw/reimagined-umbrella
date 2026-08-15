use std::collections::BTreeSet;

use proexel_domain::{ComplexityLevel, UserAccount};
use serde::Deserialize;

use crate::{
    state::{
        action, json_string, parse_data, require_permission, require_text, role_name, Action,
        CommandPayload,
    },
    ApplicationState, Role,
};

impl ApplicationState {
    pub fn seed_users(&mut self, mut users: Vec<UserAccount>) -> Result<(), String> {
        if !self.user_accounts.is_empty() {
            return Ok(());
        }
        let unique_ids = users
            .iter()
            .map(|user| user.id.as_str())
            .collect::<BTreeSet<_>>();
        if unique_ids.len() != users.len() {
            return Err("user_id_already_exists".to_string());
        }
        for user in &mut users {
            user.email = normalize_email(&user.email)?;
            require_text(&user.name, "user_name_required")?;
            validate_role_name(&user.role)?;
            validate_auth_hash(&user.password_hash, "password_hash_invalid")?;
            if let Some(pin_hash) = user.pin_hash.as_deref() {
                validate_auth_hash(pin_hash, "pin_hash_invalid")?;
            }
        }
        let unique_emails = users
            .iter()
            .map(|user| user.email.as_str())
            .collect::<BTreeSet<_>>();
        if unique_emails.len() != users.len() {
            return Err("user_email_already_exists".to_string());
        }
        if !users.iter().any(|user| user.active && user.role == "admin") {
            return Err("active_admin_required".to_string());
        }
        self.user_accounts = users;
        Ok(())
    }

    pub(crate) fn create_user(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "admin.users.manage")?;
        let input: CreateUser = parse_data(payload)?;
        let email = normalize_email(&input.email)?;
        require_text(&input.name, "user_name_required")?;
        validate_auth_hash(&input.password_hash, "password_hash_invalid")?;
        if let Some(pin_hash) = input.pin_hash.as_deref() {
            validate_auth_hash(pin_hash, "pin_hash_invalid")?;
        }
        if self.user_accounts.iter().any(|user| user.email == email) {
            return Err("user_email_already_exists".to_string());
        }
        let id = format!("user-{command_id}");
        let user = UserAccount {
            id: id.clone(),
            email,
            name: input.name.trim().to_string(),
            role: role_name(input.role).to_string(),
            password_hash: input.password_hash,
            pin_hash: input.pin_hash,
            active: true,
            maximum_repair_level: input.maximum_repair_level,
            auth_version: 1,
            created_at_ms: now,
            updated_at_ms: now,
        };
        let after = user_audit_value(&user);
        self.user_accounts.push(user);
        Ok(action(
            "admin.users.manage",
            "user_account",
            id,
            None,
            after,
            "User created",
        ))
    }

    pub(crate) fn update_user(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "admin.users.manage")?;
        let input: UpdateUser = parse_data(payload)?;
        let email = normalize_email(&input.email)?;
        require_text(&input.name, "user_name_required")?;
        let index = self
            .user_accounts
            .iter()
            .position(|user| user.id == input.id)
            .ok_or_else(|| "user_not_found".to_string())?;
        if self
            .user_accounts
            .iter()
            .any(|user| user.id != input.id && user.email == email)
        {
            return Err("user_email_already_exists".to_string());
        }
        let current = &self.user_accounts[index];
        let removes_active_admin = current.active
            && current.role == "admin"
            && (!input.active || input.role != Role::Admin);
        if removes_active_admin
            && self
                .user_accounts
                .iter()
                .filter(|user| user.active && user.role == "admin")
                .count()
                <= 1
        {
            return Err("last_active_admin_required".to_string());
        }
        let before = user_audit_value(current);
        let user = &mut self.user_accounts[index];
        user.email = email;
        user.name = input.name.trim().to_string();
        user.role = role_name(input.role).to_string();
        user.active = input.active;
        user.maximum_repair_level = input.maximum_repair_level;
        user.auth_version = user.auth_version.saturating_add(1);
        user.updated_at_ms = now;
        let after = user_audit_value(user);
        Ok(action(
            "admin.users.manage",
            "user_account",
            input.id,
            before,
            after,
            "User updated",
        ))
    }

    pub(crate) fn reset_user_credentials(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "admin.users.manage")?;
        let input: ResetUserCredentials = parse_data(payload)?;
        if input.password_hash.is_none() && input.pin_hash.is_none() && !input.clear_pin {
            return Err("credential_change_required".to_string());
        }
        if let Some(password_hash) = input.password_hash.as_deref() {
            validate_auth_hash(password_hash, "password_hash_invalid")?;
        }
        if let Some(pin_hash) = input.pin_hash.as_deref() {
            validate_auth_hash(pin_hash, "pin_hash_invalid")?;
        }
        let user = self
            .user_accounts
            .iter_mut()
            .find(|user| user.id == input.id)
            .ok_or_else(|| "user_not_found".to_string())?;
        let before = user_audit_value(user);
        if let Some(password_hash) = input.password_hash {
            user.password_hash = password_hash;
        }
        if input.clear_pin {
            user.pin_hash = None;
        } else if let Some(pin_hash) = input.pin_hash {
            user.pin_hash = Some(pin_hash);
        }
        user.auth_version = user.auth_version.saturating_add(1);
        user.updated_at_ms = now;
        let after = user_audit_value(user);
        Ok(action(
            "admin.users.manage",
            "user_account",
            input.id,
            before,
            after,
            "User credentials reset",
        ))
    }
}

fn normalize_email(value: &str) -> Result<String, String> {
    let email = value.trim().to_lowercase();
    let (local, domain) = email
        .split_once('@')
        .ok_or_else(|| "user_email_invalid".to_string())?;
    if local.is_empty() || !domain.contains('.') || email.contains(char::is_whitespace) {
        return Err("user_email_invalid".to_string());
    }
    Ok(email)
}

fn validate_role_name(role: &str) -> Result<(), String> {
    if matches!(role, "admin" | "chefe" | "compras" | "tecnico") {
        Ok(())
    } else {
        Err("user_role_invalid".to_string())
    }
}

fn validate_auth_hash(value: &str, error: &str) -> Result<(), String> {
    let mut parts = value.split('$');
    let Some(algorithm) = parts.next() else {
        return Err(error.to_string());
    };
    let Some(salt) = parts.next() else {
        return Err(error.to_string());
    };
    let Some(digest) = parts.next() else {
        return Err(error.to_string());
    };
    if algorithm != "scrypt"
        || salt.is_empty()
        || digest.len() < 32
        || !digest.len().is_multiple_of(2)
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        || parts.next().is_some()
    {
        return Err(error.to_string());
    }
    Ok(())
}

fn user_audit_value(user: &UserAccount) -> Option<String> {
    json_string(&serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "role": user.role,
        "active": user.active,
        "maximum_repair_level": user.maximum_repair_level,
        "has_pin": user.pin_hash.is_some(),
        "auth_version": user.auth_version,
    }))
}

#[derive(Deserialize)]
struct CreateUser {
    email: String,
    name: String,
    role: Role,
    password_hash: String,
    pin_hash: Option<String>,
    maximum_repair_level: ComplexityLevel,
}

#[derive(Deserialize)]
struct UpdateUser {
    id: String,
    email: String,
    name: String,
    role: Role,
    active: bool,
    maximum_repair_level: ComplexityLevel,
}

#[derive(Deserialize)]
struct ResetUserCredentials {
    id: String,
    password_hash: Option<String>,
    pin_hash: Option<String>,
    #[serde(default)]
    clear_pin: bool,
}
