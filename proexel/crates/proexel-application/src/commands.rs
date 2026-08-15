pub const CREATE_VALVE: &str = "proexel.valves.create";
pub const UPDATE_VALVE: &str = "proexel.valves.update";
pub const ADD_VALVE_PHOTO: &str = "proexel.valves.add_photo";
pub const DELETE_VALVE_PHOTO: &str = "proexel.valves.delete_photo";
pub const REGISTER_MAINTENANCE: &str = "proexel.maintenance.register";
pub const CREATE_SERVICE_ORDER: &str = "proexel.orders.create";
pub const CHANGE_SERVICE_ORDER_STATUS: &str = "proexel.orders.change_status";
pub const DELETE_SERVICE_ORDER: &str = "proexel.orders.delete";
pub const CREATE_RESTOCK_REQUEST: &str = "proexel.purchasing.create_restock_request";
pub const REVIEW_RESTOCK_REQUEST: &str = "proexel.purchasing.review_restock_request";
pub const DELETE_RESTOCK_REQUEST: &str = "proexel.purchasing.delete_restock_request";
pub const ADJUST_STOCK: &str = "proexel.stock.adjust";
pub const UPSERT_STOCK_ITEM: &str = "proexel.stock.upsert_item";
pub const DELETE_STOCK_ITEM: &str = "proexel.stock.delete_item";
pub const CREATE_SUPPLIER: &str = "proexel.suppliers.create";
pub const UPDATE_SUPPLIER: &str = "proexel.suppliers.update";
pub const DELETE_SUPPLIER: &str = "proexel.suppliers.delete";
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
    ADD_VALVE_PHOTO,
    DELETE_VALVE_PHOTO,
    REGISTER_MAINTENANCE,
    CREATE_SERVICE_ORDER,
    CHANGE_SERVICE_ORDER_STATUS,
    DELETE_SERVICE_ORDER,
    CREATE_RESTOCK_REQUEST,
    REVIEW_RESTOCK_REQUEST,
    DELETE_RESTOCK_REQUEST,
    ADJUST_STOCK,
    UPSERT_STOCK_ITEM,
    DELETE_STOCK_ITEM,
    CREATE_SUPPLIER,
    UPDATE_SUPPLIER,
    DELETE_SUPPLIER,
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
