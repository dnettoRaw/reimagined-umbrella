pub const CREATE_ITEM_CATEGORY: &str = "proexel.item_categories.create";
pub const UPDATE_ITEM_CATEGORY: &str = "proexel.item_categories.update";
pub const CREATE_MACHINE: &str = "proexel.machines.create";
pub const UPDATE_MACHINE: &str = "proexel.machines.update";
pub const ADD_MACHINE_ITEM: &str = "proexel.machine_items.add";
pub const UPDATE_MACHINE_ITEM: &str = "proexel.machine_items.update";
pub const REORDER_MACHINE_ITEMS: &str = "proexel.machine_items.reorder";
pub const REMOVE_MACHINE_ITEM: &str = "proexel.machine_items.remove";
pub const REPLACE_MACHINE_ITEM: &str = "proexel.machine_items.replace";
pub const ADD_PHOTO: &str = "proexel.photos.add";
pub const DELETE_PHOTO: &str = "proexel.photos.delete";
pub const CREATE_SERVICE_ORDER: &str = "proexel.orders.create";
pub const START_SERVICE_ORDER: &str = "proexel.orders.start";
pub const ASSIGN_ORDER_TASK: &str = "proexel.orders.assign_task";
pub const DELETE_SERVICE_ORDER: &str = "proexel.orders.delete";
pub const COMPLETE_SERVICE_ORDER: &str = "proexel.orders.complete";
pub const START_INSPECTION: &str = "proexel.inspections.start";
pub const COMPLETE_INSPECTION: &str = "proexel.inspections.complete";
pub const CREATE_RESTOCK_REQUEST: &str = "proexel.purchasing.create_restock_request";
pub const REVIEW_RESTOCK_REQUEST: &str = "proexel.purchasing.review_restock_request";
pub const DELETE_RESTOCK_REQUEST: &str = "proexel.purchasing.delete_restock_request";
pub const ADJUST_STOCK: &str = "proexel.stock.adjust";
pub const UPSERT_STOCK_ITEM: &str = "proexel.stock.upsert_item";
pub const DELETE_STOCK_ITEM: &str = "proexel.stock.delete_item";
pub const CREATE_SUPPLIER: &str = "proexel.suppliers.create";
pub const UPDATE_SUPPLIER: &str = "proexel.suppliers.update";
pub const DELETE_SUPPLIER: &str = "proexel.suppliers.delete";
pub const CREATE_USER: &str = "proexel.admin.users.create";
pub const UPDATE_USER: &str = "proexel.admin.users.update";
pub const RESET_USER_CREDENTIALS: &str = "proexel.admin.users.reset_credentials";

pub const GET_OVERVIEW: &str = "proexel.overview.get";
pub const LIST_ITEM_CATEGORIES: &str = "proexel.item_categories.list";
pub const LIST_MACHINES: &str = "proexel.machines.list";
pub const LIST_SERVICE_ORDERS: &str = "proexel.orders.list";
pub const LIST_INSPECTIONS: &str = "proexel.inspections.list";
pub const LIST_RESTOCK_REQUESTS: &str = "proexel.purchasing.list_restock_requests";
pub const LIST_STOCK: &str = "proexel.stock.list";
pub const LIST_SUPPLIERS: &str = "proexel.suppliers.list";
pub const LIST_AUDIT: &str = "proexel.audit.list";
pub const GET_REPORTS: &str = "proexel.reports.get";
pub const LIST_USERS: &str = "proexel.admin.users.list";
pub const LIST_OPERATORS: &str = "proexel.operators.list";
pub const RESOLVE_IDENTITY: &str = "proexel.identity.resolve";

pub const COMMANDS: &[&str] = &[
    CREATE_ITEM_CATEGORY,
    UPDATE_ITEM_CATEGORY,
    CREATE_MACHINE,
    UPDATE_MACHINE,
    ADD_MACHINE_ITEM,
    UPDATE_MACHINE_ITEM,
    REORDER_MACHINE_ITEMS,
    REMOVE_MACHINE_ITEM,
    REPLACE_MACHINE_ITEM,
    ADD_PHOTO,
    DELETE_PHOTO,
    CREATE_SERVICE_ORDER,
    START_SERVICE_ORDER,
    ASSIGN_ORDER_TASK,
    DELETE_SERVICE_ORDER,
    COMPLETE_SERVICE_ORDER,
    START_INSPECTION,
    COMPLETE_INSPECTION,
    CREATE_RESTOCK_REQUEST,
    REVIEW_RESTOCK_REQUEST,
    DELETE_RESTOCK_REQUEST,
    ADJUST_STOCK,
    UPSERT_STOCK_ITEM,
    DELETE_STOCK_ITEM,
    CREATE_SUPPLIER,
    UPDATE_SUPPLIER,
    DELETE_SUPPLIER,
    CREATE_USER,
    UPDATE_USER,
    RESET_USER_CREDENTIALS,
];

pub const QUERIES: &[&str] = &[
    GET_OVERVIEW,
    LIST_ITEM_CATEGORIES,
    LIST_MACHINES,
    LIST_SERVICE_ORDERS,
    LIST_INSPECTIONS,
    LIST_RESTOCK_REQUESTS,
    LIST_STOCK,
    LIST_SUPPLIERS,
    LIST_AUDIT,
    GET_REPORTS,
    LIST_USERS,
    LIST_OPERATORS,
    RESOLVE_IDENTITY,
];
