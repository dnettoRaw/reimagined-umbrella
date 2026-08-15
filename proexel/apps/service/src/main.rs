mod application;
mod asset_queries;
mod bootstrap;
mod command_handler;
mod operation_queries;
mod query_endpoint;
mod query_response;
mod query_support;
mod release_runtime;
mod summary_queries;

fn main() {
    bootstrap::run();
}

#[cfg(test)]
mod tests;
