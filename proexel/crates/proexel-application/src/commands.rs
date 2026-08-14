pub const CREATE_VALVE: &str = "proexel.valves.create";
pub const UPDATE_VALVE: &str = "proexel.valves.update";
pub const REGISTER_MAINTENANCE: &str = "proexel.maintenance.register";
pub const CREATE_SERVICE_ORDER: &str = "proexel.orders.create";
pub const CHANGE_SERVICE_ORDER_STATUS: &str = "proexel.orders.change_status";
pub const CREATE_RESTOCK_REQUEST: &str = "proexel.purchasing.create_restock_request";
pub const REVIEW_RESTOCK_REQUEST: &str = "proexel.purchasing.review_restock_request";
pub const ADJUST_STOCK: &str = "proexel.stock.adjust";
pub const UPSERT_STOCK_ITEM: &str = "proexel.stock.upsert_item";
pub const CREATE_SUPPLIER: &str = "proexel.suppliers.create";
pub const UPDATE_SUPPLIER: &str = "proexel.suppliers.update";
pub const LIST_VALVES: &str = "proexel.valves.list";
pub const GET_OVERVIEW: &str = "proexel.overview.get";
pub const LIST_MAINTENANCE: &str = "proexel.maintenance.list";
pub const LIST_SERVICE_ORDERS: &str = "proexel.orders.list";
pub const LIST_RESTOCK_REQUESTS: &str = "proexel.purchasing.list_restock_requests";
pub const LIST_STOCK: &str = "proexel.stock.list";
pub const LIST_SUPPLIERS: &str = "proexel.suppliers.list";
pub const LIST_AUDIT: &str = "proexel.audit.list";
pub const GET_REPORTS: &str = "proexel.reports.get";

pub const COMMANDS: &[&str] = &[
    CREATE_VALVE,
    UPDATE_VALVE,
    REGISTER_MAINTENANCE,
    CREATE_SERVICE_ORDER,
    CHANGE_SERVICE_ORDER_STATUS,
    CREATE_RESTOCK_REQUEST,
    REVIEW_RESTOCK_REQUEST,
    ADJUST_STOCK,
    UPSERT_STOCK_ITEM,
    CREATE_SUPPLIER,
    UPDATE_SUPPLIER,
];

pub const QUERIES: &[&str] = &[
    GET_OVERVIEW,
    LIST_VALVES,
    LIST_MAINTENANCE,
    LIST_SERVICE_ORDERS,
    LIST_RESTOCK_REQUESTS,
    LIST_STOCK,
    LIST_SUPPLIERS,
    LIST_AUDIT,
    GET_REPORTS,
];
