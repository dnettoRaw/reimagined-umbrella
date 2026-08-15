use appcore_bin::application::{
    CommandEnvelope, CommandHandler, CommandName, CommandResult, EventEnvelope, EventName,
    RuntimeContext, RuntimeResult,
};
use proexel_infrastructure::JsonFileStore;

pub(crate) struct ProexelCommandHandler {
    pub(crate) command: CommandName,
    pub(crate) store: JsonFileStore,
}

impl CommandHandler for ProexelCommandHandler {
    fn command_name(&self) -> CommandName {
        self.command.clone()
    }

    fn handle(
        &self,
        command: &CommandEnvelope,
        _context: &dyn RuntimeContext,
    ) -> RuntimeResult<CommandResult> {
        let Some(idempotency_key) = command.idempotency_key.as_deref() else {
            return Ok(CommandResult::rejected("idempotency_key_required"));
        };
        let result = self.store.transact(|state| {
            state.execute(
                command.command_name.as_str(),
                &command.command_id,
                idempotency_key,
                command.issued_at_ms,
                command.payload(),
            )
        });
        let receipt = match result {
            Ok(receipt) => receipt,
            Err(error) => {
                eprintln!(
                    "proexel command rejected command={} command_id={} reason={}",
                    command.command_name.as_str(),
                    command.command_id,
                    error
                );
                return Ok(CommandResult::rejected(error));
            }
        };
        let payload = match serde_json::to_vec(&receipt) {
            Ok(payload) => payload,
            Err(error) => {
                eprintln!(
                    "proexel receipt encoding failed command_id={} reason={error}",
                    command.command_id
                );
                return Ok(CommandResult::rejected("receipt_encode_failed"));
            }
        };
        let event = EventEnvelope::new(
            EventName::new(receipt.event_name)?,
            format!("evt-{}", command.command_id),
            command.app_id.clone(),
            command.node_id.clone(),
            command.issued_at_ms,
            payload,
        )?;
        Ok(CommandResult::accepted(vec![event]))
    }
}
