use appcore_bin::application::{
    ApiRouter, Application, CommandBus, CommandName, CommandRegistry, EventName, EventRegistry,
    QueryName, RuntimeResult,
};
use proexel_application::commands;
use proexel_infrastructure::JsonFileStore;

const EVENTS: &[&str] = &[
    "item_category.created",
    "item_category.updated",
    "maintenance_guide.updated",
    "machine.created",
    "machine.updated",
    "machine_item.created",
    "machine_item.updated",
    "machine_item.removed",
    "machine_item.replaced",
    "photo.added",
    "photo.removed",
    "service_order.created",
    "service_order.started",
    "service_order.updated",
    "service_order.deleted",
    "service_order.completed",
    "service_order_task.started",
    "service_order_task.completed",
    "inspection.started",
    "inspection.completed",
    "item.status_changed",
    "restock_request.changed",
    "stock_item.changed",
    "supplier.changed",
    "user_account.changed",
];

#[derive(Clone)]
pub(crate) struct ProexelApplication {
    pub(crate) store: JsonFileStore,
}

impl Application for ProexelApplication {
    fn register_commands(&self, registry: &mut CommandRegistry) -> RuntimeResult<()> {
        for command in commands::COMMANDS {
            registry.register(CommandName::new(*command)?)?;
        }
        Ok(())
    }

    fn register_events(&self, registry: &mut EventRegistry) -> RuntimeResult<()> {
        for event in EVENTS {
            registry.register(EventName::new(*event)?)?;
        }
        Ok(())
    }

    fn register_handlers(&self, bus: &mut CommandBus) -> RuntimeResult<()> {
        for command in commands::COMMANDS {
            bus.register_handler(crate::command_handler::ProexelCommandHandler {
                command: CommandName::new(*command)?,
                store: self.store.clone(),
            })?;
        }
        Ok(())
    }

    fn register_queries(&self, router: &mut ApiRouter) -> RuntimeResult<()> {
        for query in commands::QUERIES {
            router.register_query(crate::query_endpoint::ProexelQuery {
                name: QueryName::new(*query)?,
                store: self.store.clone(),
            })?;
        }
        Ok(())
    }
}
