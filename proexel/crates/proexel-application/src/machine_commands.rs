use proexel_domain::{normalize_identifier, Machine, OperationalStatus};
use serde::Deserialize;

use crate::asset_validation::default_true;
use crate::{
    state::{
        clean_optional, domain_action, json_string, parse_data, require_permission, require_text,
        Action, CommandPayload,
    },
    ApplicationState,
};

impl ApplicationState {
    pub(crate) fn create_machine(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "machine.create")?;
        let input: MachineInput = parse_data(payload)?;
        let code = normalize_identifier(&input.code);
        require_text(&code, "machine_code_required")?;
        require_text(&input.name, "machine_name_required")?;
        require_text(&input.zone, "zone_required")?;
        if self
            .machines
            .iter()
            .any(|machine| machine.code_normalized == code)
        {
            return Err("machine_code_already_exists".to_string());
        }
        let id = format!("machine-{command_id}");
        let machine = Machine {
            id: id.clone(),
            code: code.clone(),
            code_normalized: code,
            name: input.name.trim().to_string(),
            description: clean_optional(input.description),
            zone: input.zone.trim().to_string(),
            location: clean_optional(input.location),
            manufacturer: clean_optional(input.manufacturer),
            model: clean_optional(input.model),
            serial_number: clean_optional(input.serial_number),
            status: OperationalStatus::Unknown,
            main_photo_id: None,
            active: input.active,
            created_at_ms: now,
            updated_at_ms: now,
        };
        let after = json_string(&machine);
        self.machines.push(machine);
        Ok(domain_action(
            "machine.create",
            "machine.created",
            "machine",
            id,
            None,
            after,
            "Machine created",
        ))
    }

    pub(crate) fn update_machine(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "machine.update")?;
        let input: UpdateMachine = parse_data(payload)?;
        let code = normalize_identifier(&input.machine.code);
        require_text(&code, "machine_code_required")?;
        require_text(&input.machine.name, "machine_name_required")?;
        require_text(&input.machine.zone, "zone_required")?;
        let index = self
            .machines
            .iter()
            .position(|machine| machine.id == input.id)
            .ok_or_else(|| "machine_not_found".to_string())?;
        if self
            .machines
            .iter()
            .any(|machine| machine.id != input.id && machine.code_normalized == code)
        {
            return Err("machine_code_already_exists".to_string());
        }
        let before = json_string(&self.machines[index]);
        let machine = &mut self.machines[index];
        machine.code = code.clone();
        machine.code_normalized = code;
        machine.name = input.machine.name.trim().to_string();
        machine.description = clean_optional(input.machine.description);
        machine.zone = input.machine.zone.trim().to_string();
        machine.location = clean_optional(input.machine.location);
        machine.manufacturer = clean_optional(input.machine.manufacturer);
        machine.model = clean_optional(input.machine.model);
        machine.serial_number = clean_optional(input.machine.serial_number);
        machine.active = input.machine.active;
        machine.updated_at_ms = now;
        let after = json_string(machine);
        Ok(domain_action(
            "machine.update",
            "machine.updated",
            "machine",
            input.id,
            before,
            after,
            "Machine updated",
        ))
    }
}

#[derive(Deserialize)]
struct MachineInput {
    code: String,
    name: String,
    description: Option<String>,
    zone: String,
    location: Option<String>,
    manufacturer: Option<String>,
    model: Option<String>,
    serial_number: Option<String>,
    #[serde(default = "default_true")]
    active: bool,
}

#[derive(Deserialize)]
struct UpdateMachine {
    id: String,
    #[serde(flatten)]
    machine: MachineInput,
}
