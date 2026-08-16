import { operationalGuide } from "./maintenance-guide";
import type {
  AuditEvent,
  ComplexityLevel,
  ItemCategory,
  ItemInspection,
  Machine,
  MachineItem,
  OperationalStatus,
  RestockRequest,
  ServiceOrder,
  StockItem,
  Supplier,
  UserAccount,
} from "./types";

export const DEMO_STATE_KEY = "proexel.demo.state.v1";
export const DEMO_OPERATIONS_KEY = "proexel.demo.operations.v1";
export const DEMO_OPERATIONS_COOKIE = "proexel_demo_ops";

export interface DemoState {
  categories: ItemCategory[];
  machines: Machine[];
  orders: ServiceOrder[];
  inspections: ItemInspection[];
  stock: StockItem[];
  restockRequests: RestockRequest[];
  suppliers: Supplier[];
  users: UserAccount[];
  audit: AuditEvent[];
}

export interface DemoOperation {
  id: string;
  endpoint: string;
  method: string;
  data: Record<string, unknown>;
  at: number;
}

export function sanitizeDemoOperationData(endpoint: string, data: Record<string, unknown>) {
  if (endpoint === "/api/proexel/inspections") {
    if (typeof data.order_id === "string") return pick(data, ["order_id", "task_id"]);
    return pick(data, ["id", "status_after", "step_results", "findings", "notes", "maintenance_action"]);
  }
  if (endpoint === "/api/proexel/photos")
    return pick(data, ["id", "owner_type", "owner_id", "purpose", "blob_ref", "description"]);
  if (endpoint !== "/api/proexel/users") return data;
  const sanitized: Record<string, unknown> = {
    ...data,
    has_pin: typeof data.pin === "string" && data.pin.length > 0,
  };
  delete sanitized.password;
  delete sanitized.pin;
  delete sanitized.password_hash;
  delete sanitized.pin_hash;
  return sanitized;
}

function pick(data: Record<string, unknown>, keys: string[]) {
  return Object.fromEntries(keys.filter((key) => data[key] !== undefined).map((key) => [key, data[key]]));
}

const BASE_TIME = Date.UTC(2026, 7, 16, 9);
const DAY = 86_400_000;
const STATUSES: OperationalStatus[] = [
  "ok",
  "ok",
  "attention",
  "ok",
  "critical",
  "under_maintenance",
  "ok",
  "maintenance_required",
  "ok",
  "ok",
];
const ZONES = [
  "Linha A",
  "Linha A",
  "Linha B",
  "Linha B",
  "Utilidades",
  "Utilidades",
  "Embalagem",
  "Embalagem",
  "Armazém",
  "Qualidade",
];
const MACHINE_NAMES = [
  "Misturador",
  "Prensa",
  "Enchedora",
  "Transportador",
  "Compressor",
  "Caldeira",
  "Rotuladora",
  "Paletizador",
  "Elevador",
  "Bancada de teste",
];
const MANUFACTURERS = ["ABB", "Siemens", "Schneider", "Emerson"];

function category(id: string, code: string, name: string, complexity: ComplexityLevel): ItemCategory {
  return {
    id,
    code,
    code_normalized: code.toLowerCase(),
    name,
    description: `${name} industrial com roteiro preventivo padronizado.`,
    icon: null,
    default_complexity_level: complexity,
    maintenance_guide: {
      version: 1,
      steps: [
        {
          id: `${id}-visual`,
          title: "Inspeção visual",
          description: "Verifique desgaste, ruído, folgas e sinais de aquecimento.",
          instructions: "Inspecione o conjunto com a máquina parada e bloqueada.",
          step_type: "confirmation",
          required: true,
          reference_photo_ids: [],
          safety_warning: "Aplicar bloqueio e etiquetagem antes de intervir.",
          expected_value: null,
          options: [],
          order: 0,
        },
      ],
    },
    custom_field_definitions: [],
    recommended_parts: [{ manufacturer: MANUFACTURERS[complexity % MANUFACTURERS.length], part_number: `${code}-STD` }],
    guide_photos: [],
    active: true,
    created_at_ms: BASE_TIME - 400 * DAY,
    updated_at_ms: BASE_TIME - 20 * DAY,
  };
}

function installed(machineIndex: number, itemIndex: number, code: string) {
  return {
    installation_id: `installation-${machineIndex + 1}-${itemIndex + 1}`,
    manufacturer: MANUFACTURERS[(machineIndex + itemIndex) % MANUFACTURERS.length],
    model: `PX-${200 + machineIndex * 10 + itemIndex}`,
    part_number: `${code}-PN`,
    serial_number: `SN-${20260000 + machineIndex * 4 + itemIndex + 1}`,
    installed_at: "2025-11-10",
    technical_specifications: { voltage: "400 V", protection: "IP55" },
  };
}

function machineItem(
  machineId: string,
  machineIndex: number,
  itemIndex: number,
  categories: ItemCategory[],
): MachineItem {
  const selected = categories[itemIndex];
  const code = `M-${String(machineIndex + 1).padStart(3, "0")}-${selected.code}`;
  const status =
    itemIndex === 0 ? STATUSES[machineIndex] : itemIndex === 2 && machineIndex % 3 === 0 ? "attention" : "ok";
  const component = installed(machineIndex, itemIndex, code);
  return {
    id: `item-${machineIndex + 1}-${itemIndex + 1}`,
    machine_id: machineId,
    category_id: selected.id,
    category: selected,
    name: `${selected.name} principal`,
    code,
    code_normalized: code.toLowerCase(),
    complexity_level: selected.default_complexity_level,
    status,
    position: itemIndex + 1,
    custom_field_values: {},
    maintenance_guide_override: null,
    installed_component: component,
    replacement_specification: {
      manufacturer: component.manufacturer,
      part_number: component.part_number,
      model: component.model,
      serial_number: null,
      technical_specifications: component.technical_specifications,
      compatibility_notes: "Substituição direta conforme especificação do fabricante.",
      equivalent_parts: [],
      supplier_reference: `SUP-${selected.code}`,
      photo_ids: [],
    },
    notes: null,
    active: true,
    removed_at_ms: null,
    created_at_ms: BASE_TIME - (300 - machineIndex * 4 - itemIndex) * DAY,
    updated_at_ms: BASE_TIME - (machineIndex + itemIndex + 1) * DAY,
    photos: [],
    replacement_history: [],
  };
}

function createMachines(categories: ItemCategory[]): Machine[] {
  return Array.from({ length: 10 }, (_, index) => {
    const id = `machine-${index + 1}`;
    return {
      id,
      code: `M-${String(index + 1).padStart(3, "0")}`,
      code_normalized: `m-${String(index + 1).padStart(3, "0")}`,
      name: MACHINE_NAMES[index],
      description: `${MACHINE_NAMES[index]} de produção da ${ZONES[index]}.`,
      zone: ZONES[index],
      location: `Setor ${String.fromCharCode(65 + (index % 5))}-${index + 1}`,
      manufacturer: MANUFACTURERS[index % MANUFACTURERS.length],
      model: `PRO-${200 + index * 10}`,
      serial_number: `MACH-${202600 + index + 1}`,
      status: STATUSES[index],
      main_photo_id: null,
      active: true,
      created_at_ms: BASE_TIME - (500 - index) * DAY,
      updated_at_ms: BASE_TIME - (index + 2) * DAY,
      items: Array.from({ length: 4 }, (_, itemIndex) => machineItem(id, index, itemIndex, categories)),
      photos: [],
    };
  });
}

function snapshot(item: MachineItem) {
  const selected = item.category as ItemCategory;
  const guide = operationalGuide(item.maintenance_guide_override ?? selected.maintenance_guide);
  return {
    id: item.id,
    machine_id: item.machine_id,
    category: {
      id: selected.id,
      code: selected.code,
      name: selected.name,
      guide_version: guide.version,
      maintenance_guide: guide,
      guide_reference_photos: [],
    },
    name: item.name,
    code: item.code,
    complexity_level: item.complexity_level,
    installed_component: item.installed_component,
  };
}

function createOrders(machines: Machine[]): ServiceOrder[] {
  const definitions = [
    [4, "Revisão do compressor", "in_progress", "urgent", 2],
    [2, "Calibração dos sensores", "pending", "high", 2],
    [7, "Inspeção preventiva", "pending", "normal", 3],
    [0, "Lubrificação geral", "completed", "normal", 2],
    [5, "Teste de segurança", "completed", "high", 2],
    [3, "Troca de rolamentos", "pending", "high", 2],
  ] as const;
  return definitions.map(([machineIndex, description, status, priority, taskCount], index) => {
    const machine = machines[machineIndex];
    const completed = status === "completed" ? taskCount : status === "in_progress" ? 1 : 0;
    const tasks = machine.items.slice(0, taskCount).map((item, taskIndex) => ({
      id: `task-${index + 1}-${taskIndex + 1}`,
      machine_item_id: item.id,
      item_snapshot: snapshot(item),
      complexity_snapshot: item.complexity_level,
      assigned_operator_id: taskIndex % 2 ? "user-5" : "user-4",
      status:
        taskIndex < completed
          ? ("completed" as const)
          : status === "in_progress" && taskIndex === completed
            ? ("in_progress" as const)
            : ("pending" as const),
      started_at_ms: taskIndex <= completed && status !== "pending" ? BASE_TIME - (index + 1) * DAY : null,
      completed_at_ms: taskIndex < completed ? BASE_TIME - index * DAY : null,
      inspection_id: taskIndex < completed ? `inspection-${index + 1}-${taskIndex + 1}` : null,
    }));
    return {
      id: `order-${index + 1}`,
      machine_id: machine.id,
      machine_snapshot: {
        id: machine.id,
        code: machine.code,
        name: machine.name,
        zone: machine.zone,
        location: machine.location,
      },
      description,
      priority,
      status,
      created_by: "Marcos Silva",
      scheduled_for: new Date(BASE_TIME + index * DAY).toISOString().slice(0, 10),
      tasks,
      maximum_complexity_level: Math.max(...tasks.map((task) => task.complexity_snapshot)) as ComplexityLevel,
      completed_tasks: completed,
      created_at_ms: BASE_TIME - (8 - index) * DAY,
      started_at_ms: status === "pending" ? null : BASE_TIME - (index + 1) * DAY,
      completed_at_ms: status === "completed" ? BASE_TIME - index * DAY : null,
      updated_at_ms: BASE_TIME - index * DAY,
    };
  });
}

function createInspections(orders: ServiceOrder[]): ItemInspection[] {
  return orders
    .flatMap((order) => order.tasks.filter((task) => task.status === "completed").map((task) => ({ order, task })))
    .map(({ order, task }, index) => ({
      id: task.inspection_id ?? `inspection-${index + 1}`,
      service_order_task_id: task.id,
      service_order_id: order.id,
      machine_id: order.machine_id,
      machine_item_id: task.machine_item_id,
      category_snapshot: task.item_snapshot.category,
      operator_id: task.assigned_operator_id ?? "user-4",
      operator_name: index % 2 ? "Sofia Ramos" : "Carlos Mendes",
      status: "completed" as const,
      started_at_ms: BASE_TIME - (index + 3) * DAY,
      completed_at_ms: BASE_TIME - (index + 3) * DAY + 3_600_000,
      status_before: "attention" as const,
      status_after: "ok" as const,
      step_results: [{ step_id: `${task.item_snapshot.category.id}-visual`, value: true, photo_ids: [] }],
      findings: [],
      photo_ids: [],
      notes: "Inspeção concluída sem anomalias críticas.",
      maintenance_action: "Limpeza, reaperto e teste funcional.",
      photos: [],
    }));
}

export function createDemoState(): DemoState {
  const categories = [
    category("category-motor", "MOTOR", "Motor", 3),
    category("category-pump", "BOMBA", "Bomba", 4),
    category("category-sensor", "SENSOR", "Sensor", 2),
    category("category-valve", "VALVULA", "Válvula", 2),
  ];
  const machines = createMachines(categories);
  const orders = createOrders(machines);
  return {
    categories,
    machines,
    orders,
    inspections: createInspections(orders),
    stock: [
      ["ROL-6204", 4, 8, "SKF", "A-01-03"],
      ["SEN-PT100", 12, 6, "Emerson", "B-02-01"],
      ["VED-40MM", 2, 5, "FlowSeal", "A-03-04"],
      ["OLE-ISO46", 36, 20, "LubePro", "C-01-02"],
      ["COR-A42", 7, 6, "Gates", "A-02-07"],
      ["FUS-10A", 28, 15, "Siemens", "B-01-05"],
      ["VAL-DN25", 3, 4, "Emerson", "A-04-02"],
      ["FIL-COMP", 9, 4, "Atlas", "C-02-06"],
    ].map(([reference, quantity, minimum_quantity, manufacturer, location], index) => ({
      id: `stock-${index + 1}`,
      reference: String(reference),
      quantity: Number(quantity),
      minimum_quantity: Number(minimum_quantity),
      manufacturer: String(manufacturer),
      location: String(location),
    })),
    restockRequests: [
      {
        id: "restock-1",
        reference: "ROL-6204",
        reason: "Saldo abaixo do mínimo",
        requested_by: "Carlos Mendes",
        status: "pending",
      },
      {
        id: "restock-2",
        reference: "VED-40MM",
        reason: "Reposição preventiva",
        requested_by: "Sofia Ramos",
        status: "approved",
        reviewed_by: "Rui Martins",
        reviewed_at_ms: BASE_TIME - DAY,
      },
      {
        id: "restock-3",
        reference: "VAL-DN25",
        reason: "Reserva de manutenção",
        requested_by: "Carlos Mendes",
        status: "pending",
      },
    ],
    suppliers: [
      {
        id: "supplier-1",
        name: "Industrial Norte",
        contact: "Paulo Neves",
        email: "vendas@industrialnorte.demo",
        website: "https://example.com",
        notes: "Componentes mecânicos",
      },
      {
        id: "supplier-2",
        name: "TecnoSensor",
        contact: "Marta Lopes",
        email: "comercial@tecnosensor.demo",
        website: "https://example.com",
        notes: "Instrumentação",
      },
      {
        id: "supplier-3",
        name: "LubePro",
        contact: "João Reis",
        email: "pedidos@lubepro.demo",
        website: "https://example.com",
        notes: "Lubrificantes",
      },
      {
        id: "supplier-4",
        name: "MecaParts",
        contact: "Sara Vidal",
        email: "info@mecaparts.demo",
        website: "https://example.com",
        notes: "Transmissão",
      },
    ],
    users: [
      ["Marcos Silva", "admin@proexel.demo", "admin", 5, true, true],
      ["Rui Martins", "chefe@proexel.demo", "chefe", 5, true, true],
      ["Laura Pinto", "compras@proexel.demo", "compras", 3, true, false],
      ["Carlos Mendes", "tecnico@proexel.demo", "tecnico", 4, true, true],
      ["Sofia Ramos", "sofia@proexel.demo", "tecnico", 3, true, true],
    ].map(([name, email, role, level, active, hasPin], index) => ({
      id: `user-${index + 1}`,
      name: String(name),
      email: String(email),
      role: role as UserAccount["role"],
      maximum_repair_level: Number(level) as ComplexityLevel,
      active: Boolean(active),
      has_pin: Boolean(hasPin),
      auth_version: 1,
      created_at_ms: BASE_TIME - (120 - index) * DAY,
      updated_at_ms: BASE_TIME - index * DAY,
    })),
    audit: Array.from({ length: 8 }, (_, index) => ({
      id: `audit-${index + 1}`,
      actor: ["Marcos Silva", "Carlos Mendes", "Laura Pinto", "Rui Martins"][index % 4],
      role: (["admin", "tecnico", "compras", "chefe"] as const)[index % 4],
      operation: ["updated", "completed", "adjusted", "created"][index % 4],
      aggregate: ["machine", "inspection", "stock_item", "service_order"][index % 4],
      aggregate_id: `demo-${index + 1}`,
      description: ["machine_updated", "inspection_completed", "stock_adjusted", "order_created"][index % 4],
      trace_id: `trace-demo-${index + 1}`,
      before_json: null,
      after_json: null,
      result: "success",
      created_at_ms: BASE_TIME - index * DAY,
    })),
  };
}

export function applyDemoOperations(base: DemoState, operations: DemoOperation[]): DemoState {
  const state = JSON.parse(JSON.stringify(base)) as DemoState;
  for (const operation of operations) applyOperation(state, operation);
  return state;
}

function applyOperation(state: DemoState, operation: DemoOperation) {
  const { endpoint, method, data, id, at } = operation;
  const resourceId = `demo-${id}`;
  if (endpoint === "/api/proexel/machines") {
    if (method === "POST") state.machines.push(newMachine(resourceId, data, at));
    if (method === "PATCH") updateById(state.machines, text(data.id), data, at);
  } else if (endpoint === "/api/proexel/categories") {
    if (method === "POST") state.categories.push(newCategory(resourceId, data, at));
    if (method === "PATCH") updateById(state.categories, text(data.id), data, at);
  } else if (endpoint === "/api/proexel/machine-items") {
    applyMachineItemOperation(state, operation, resourceId);
  } else if (endpoint === "/api/proexel/orders") {
    applyOrderOperation(state, operation, resourceId);
  } else if (endpoint === "/api/proexel/inspections") {
    applyInspectionOperation(state, operation, resourceId);
  } else if (endpoint === "/api/proexel/stock") {
    applyStockOperation(state, operation, resourceId);
  } else if (endpoint === "/api/proexel/purchasing") {
    applyPurchasingOperation(state, operation, resourceId);
  } else if (endpoint === "/api/proexel/suppliers") {
    applyCrud(state.suppliers, operation, resourceId, {
      name: "",
      contact: "",
      email: null,
      website: null,
      notes: null,
    });
  } else if (endpoint === "/api/proexel/users") {
    applyUserOperation(state, operation, resourceId);
  }
  state.audit.unshift({
    id: `audit-${id}`,
    actor: "Marcos Silva",
    role: "admin",
    operation: method === "POST" ? "created" : method === "DELETE" ? "deleted" : "updated",
    aggregate: auditAggregate(endpoint),
    aggregate_id: text(data.id) || resourceId,
    description: auditDescription(endpoint, method),
    trace_id: `trace-${id}`,
    before_json: null,
    after_json: JSON.stringify(data),
    result: "success",
    created_at_ms: at,
  });
}

function newMachine(id: string, data: Record<string, unknown>, at: number): Machine {
  return {
    id,
    code: text(data.code),
    code_normalized: text(data.code).toLowerCase(),
    name: text(data.name),
    description: nullable(data.description),
    zone: text(data.zone),
    location: nullable(data.location),
    manufacturer: nullable(data.manufacturer),
    model: nullable(data.model),
    serial_number: nullable(data.serial_number),
    status: "unknown",
    main_photo_id: null,
    active: data.active !== false,
    created_at_ms: at,
    updated_at_ms: at,
    items: [],
    photos: [],
  };
}

function newCategory(id: string, data: Record<string, unknown>, at: number): ItemCategory {
  const fallback = category(id, text(data.code), text(data.name), level(data.default_complexity_level));
  return {
    ...fallback,
    ...data,
    id,
    code_normalized: text(data.code).toLowerCase(),
    created_at_ms: at,
    updated_at_ms: at,
  } as ItemCategory;
}

function applyMachineItemOperation(state: DemoState, operation: DemoOperation, resourceId: string) {
  const { method, data, at } = operation;
  const machine =
    state.machines.find((item) => item.id === text(data.machine_id)) ??
    state.machines.find((item) => item.items.some((component) => component.id === text(data.id)));
  if (!machine) return;
  if (method === "POST") {
    const selected = state.categories.find((item) => item.id === text(data.category_id)) ?? state.categories[0];
    const item = machineItem(machine.id, 0, 0, state.categories);
    machine.items.push({
      ...item,
      ...data,
      id: resourceId,
      category: selected,
      category_id: selected.id,
      code_normalized: text(data.code).toLowerCase(),
      position: machine.items.length,
      created_at_ms: at,
      updated_at_ms: at,
    } as MachineItem);
  } else if (method === "DELETE") {
    const item = machine.items.find((component) => component.id === text(data.id));
    if (item) {
      item.active = false;
      item.removed_at_ms = at;
    }
  } else if (method === "PATCH") {
    updateById(machine.items, text(data.id), data, at);
    const item = machine.items.find((component) => component.id === text(data.id));
    const selected = state.categories.find((category) => category.id === item?.category_id);
    if (item && selected) item.category = selected;
  } else if (method === "PUT" && data.action === "reorder" && Array.isArray(data.item_ids)) {
    for (const [position, itemId] of data.item_ids.entries()) {
      const item = machine.items.find((component) => component.id === itemId);
      if (item) item.position = position + 1;
    }
  } else if (method === "PUT") {
    const item = machine.items.find((component) => component.id === text(data.id));
    if (item)
      item.installed_component = {
        ...item.installed_component,
        installation_id: resourceId,
        manufacturer: nullable(data.manufacturer),
        model: nullable(data.model),
        part_number: nullable(data.part_number),
        serial_number: nullable(data.serial_number),
        installed_at: nullable(data.installed_at),
        technical_specifications: object(data.technical_specifications),
      };
  }
}

function applyOrderOperation(state: DemoState, operation: DemoOperation, resourceId: string) {
  const { method, data, at } = operation;
  if (method === "POST") {
    const machine = state.machines.find((item) => item.id === text(data.machine_id));
    if (!machine) return;
    const selectedIds = data.all_items ? machine.items.map((item) => item.id) : array(data.item_ids);
    const selected = machine.items.filter((item) => selectedIds.includes(item.id));
    state.orders.unshift({
      id: resourceId,
      machine_id: machine.id,
      machine_snapshot: {
        id: machine.id,
        code: machine.code,
        name: machine.name,
        zone: machine.zone,
        location: machine.location,
      },
      description: text(data.description),
      priority: (text(data.priority) || "normal") as ServiceOrder["priority"],
      status: "pending",
      created_by: "Marcos Silva",
      scheduled_for: nullable(data.scheduled_for),
      tasks: selected.map((item, index) => ({
        id: `${resourceId}-task-${index + 1}`,
        machine_item_id: item.id,
        item_snapshot: snapshot(item),
        complexity_snapshot: item.complexity_level,
        assigned_operator_id: nullable(data.assigned_operator_id),
        status: "pending",
      })),
      maximum_complexity_level: Math.max(1, ...selected.map((item) => item.complexity_level)) as ComplexityLevel,
      completed_tasks: 0,
      created_at_ms: at,
      updated_at_ms: at,
    });
    return;
  }
  const order = state.orders.find((item) => item.id === text(data.id));
  if (!order) return;
  if (method === "DELETE") state.orders = state.orders.filter((item) => item.id !== order.id);
  else if (method === "PUT") {
    order.status = "completed";
    order.completed_tasks = order.tasks.length;
    order.completed_at_ms = at;
    order.tasks.forEach((task) => {
      task.status = "completed";
      task.completed_at_ms = at;
    });
  } else if (data.action === "assign") {
    const task = order.tasks.find((item) => item.id === text(data.task_id));
    if (task) task.assigned_operator_id = nullable(data.operator_id);
  } else {
    order.status = "in_progress";
    order.started_at_ms = at;
  }
  order.updated_at_ms = at;
}

function applyInspectionOperation(state: DemoState, operation: DemoOperation, resourceId: string) {
  const { method, data, at } = operation;
  if (method === "POST") {
    const order = state.orders.find((item) => item.id === text(data.order_id));
    const task = order?.tasks.find((item) => item.id === text(data.task_id));
    if (!order || !task) return;
    state.inspections.unshift({
      id: resourceId,
      service_order_task_id: task.id,
      service_order_id: order.id,
      machine_id: order.machine_id,
      machine_item_id: task.machine_item_id,
      category_snapshot: task.item_snapshot.category,
      operator_id: "user-1",
      operator_name: "Marcos Silva",
      status: "in_progress",
      started_at_ms: at,
      status_before: "attention",
      step_results: [],
      findings: [],
      photo_ids: [],
      photos: [],
    });
    task.status = "in_progress";
    task.inspection_id = resourceId;
    task.started_at_ms = at;
    order.status = "in_progress";
  } else {
    const inspection = state.inspections.find((item) => item.id === text(data.id));
    if (!inspection) return;
    inspection.status = "completed";
    inspection.completed_at_ms = at;
    inspection.status_after = (text(data.status_after) || "ok") as OperationalStatus;
    inspection.notes = nullable(data.notes);
    inspection.maintenance_action = nullable(data.maintenance_action);
    inspection.step_results = arrayObjects(data.step_results) as unknown as ItemInspection["step_results"];
    const order = state.orders.find((item) => item.id === inspection.service_order_id);
    const task = order?.tasks.find((item) => item.id === inspection.service_order_task_id);
    if (task) {
      task.status = "completed";
      task.completed_at_ms = at;
    }
    if (order) order.completed_tasks = order.tasks.filter((item) => item.status === "completed").length;
  }
}

function applyStockOperation(state: DemoState, operation: DemoOperation, resourceId: string) {
  const { method, data } = operation;
  if (method === "POST") {
    const existing = state.stock.find((item) => item.reference === text(data.reference));
    if (existing) existing.quantity += Number(data.quantity ?? 0);
    else
      state.stock.push({
        id: resourceId,
        reference: text(data.reference),
        quantity: Number(data.quantity ?? 0),
        minimum_quantity: Number(data.minimum_quantity ?? 0),
        manufacturer: nullable(data.manufacturer),
        location: nullable(data.location),
      });
  } else if (method === "PATCH") {
    const item = state.stock.find((entry) => entry.id === text(data.id));
    if (item) item.quantity = Math.max(0, item.quantity + Number(data.delta ?? 0));
  } else if (method === "DELETE") state.stock = state.stock.filter((item) => item.id !== text(data.id));
}

function applyPurchasingOperation(state: DemoState, operation: DemoOperation, resourceId: string) {
  const { method, data, at } = operation;
  if (method === "POST")
    state.restockRequests.unshift({
      id: resourceId,
      reference: text(data.reference),
      reason: text(data.reason),
      requested_by: "Marcos Silva",
      status: "pending",
    });
  else if (method === "PATCH") {
    const item = state.restockRequests.find((entry) => entry.id === text(data.id));
    if (item) {
      item.status = text(data.status) as RestockRequest["status"];
      item.reviewed_by = "Marcos Silva";
      item.reviewed_at_ms = at;
    }
  } else if (method === "DELETE")
    state.restockRequests = state.restockRequests.filter((item) => item.id !== text(data.id));
}

function applyUserOperation(state: DemoState, operation: DemoOperation, resourceId: string) {
  const { method, data, at } = operation;
  if (method === "POST")
    state.users.push({
      id: resourceId,
      name: text(data.name),
      email: text(data.email),
      role: text(data.role) as UserAccount["role"],
      active: true,
      maximum_repair_level: level(data.maximum_repair_level),
      has_pin: data.has_pin === true,
      auth_version: 1,
      created_at_ms: at,
      updated_at_ms: at,
    });
  else if (method === "PATCH") updateById(state.users, text(data.id), data, at);
  else if (method === "PUT") {
    const user = state.users.find((item) => item.id === text(data.id));
    if (user) {
      user.has_pin = data.clear_pin ? false : data.has_pin === true || user.has_pin;
      user.auth_version += 1;
      user.updated_at_ms = at;
    }
  }
}

function applyCrud<T extends { id: string }>(
  items: T[],
  operation: DemoOperation,
  resourceId: string,
  defaults: Omit<T, "id">,
) {
  if (operation.method === "POST") items.push({ ...defaults, ...operation.data, id: resourceId } as T);
  else if (operation.method === "PATCH") updateById(items, text(operation.data.id), operation.data, operation.at);
  else if (operation.method === "DELETE")
    items.splice(0, items.length, ...items.filter((item) => item.id !== text(operation.data.id)));
}

function updateById<T extends { id: string }>(items: T[], id: string, data: Record<string, unknown>, at: number) {
  const item = items.find((entry) => entry.id === id);
  if (item) Object.assign(item, data, "updated_at_ms" in item ? { updated_at_ms: at } : {});
}
function text(value: unknown) {
  return typeof value === "string" ? value.trim() : "";
}
function nullable(value: unknown) {
  const result = text(value);
  return result || null;
}
function level(value: unknown) {
  const result = Number(value);
  return (result >= 1 && result <= 5 ? result : 1) as ComplexityLevel;
}
function array(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];
}
function arrayObjects(value: unknown) {
  return Array.isArray(value)
    ? value.filter((item): item is Record<string, unknown> => Boolean(item) && typeof item === "object")
    : [];
}
function object(value: unknown) {
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Record<string, unknown>) : {};
}

function auditAggregate(endpoint: string) {
  return (
    (
      {
        "/api/proexel/categories": "item_category",
        "/api/proexel/machines": "machine",
        "/api/proexel/machine-items": "machine_item",
        "/api/proexel/orders": "service_order",
        "/api/proexel/inspections": "inspection",
        "/api/proexel/stock": "stock_item",
        "/api/proexel/purchasing": "restock_request",
        "/api/proexel/suppliers": "supplier",
        "/api/proexel/users": "user_account",
      } as Record<string, string>
    )[endpoint] ?? "demo"
  );
}

function auditDescription(endpoint: string, method: string) {
  if (endpoint === "/api/proexel/users") {
    if (method === "POST") return "User created";
    if (method === "PUT") return "User credentials reset";
    return "User updated";
  }
  return "demo_operation";
}

export function encodeDemoOperations(operations: DemoOperation[]) {
  const bytes = new TextEncoder().encode(JSON.stringify(operations.map((operation) => compactOperation(operation))));
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
}

export function decodeDemoOperations(value?: string | null): DemoOperation[] {
  if (!value) return [];
  try {
    const base64 = value
      .replaceAll("-", "+")
      .replaceAll("_", "/")
      .padEnd(Math.ceil(value.length / 4) * 4, "=");
    const binary = atob(base64);
    const decoded = new TextDecoder().decode(Uint8Array.from(binary, (character) => character.charCodeAt(0)));
    const operations = JSON.parse(decoded) as unknown[];
    return Array.isArray(operations) ? operations.map(expandOperation).filter((item) => item !== null) : [];
  } catch {
    return [];
  }
}

const ENDPOINT_CODES: Record<string, string> = {
  "/api/proexel/categories": "c",
  "/api/proexel/machines": "m",
  "/api/proexel/machine-items": "i",
  "/api/proexel/orders": "o",
  "/api/proexel/inspections": "n",
  "/api/proexel/photos": "p",
  "/api/proexel/stock": "s",
  "/api/proexel/purchasing": "r",
  "/api/proexel/suppliers": "u",
  "/api/proexel/users": "a",
};

const CODE_ENDPOINTS = Object.fromEntries(Object.entries(ENDPOINT_CODES).map(([endpoint, code]) => [code, endpoint]));

function compactOperation(operation: DemoOperation) {
  const methodCodes: Record<string, string> = { POST: "C", PATCH: "H", PUT: "U", DELETE: "D" };
  return [
    operation.id,
    ENDPOINT_CODES[operation.endpoint] ?? operation.endpoint,
    methodCodes[operation.method] ?? operation.method,
    operation.data,
    operation.at,
  ];
}

function expandOperation(value: unknown): DemoOperation | null {
  if (value && typeof value === "object" && !Array.isArray(value)) return value as DemoOperation;
  if (!Array.isArray(value) || value.length !== 5) return null;
  const [id, endpointCode, methodCode, data, at] = value;
  if (
    typeof id !== "string" ||
    typeof endpointCode !== "string" ||
    typeof methodCode !== "string" ||
    !data ||
    typeof data !== "object" ||
    Array.isArray(data) ||
    typeof at !== "number"
  )
    return null;
  const methods: Record<string, string> = { C: "POST", H: "PATCH", U: "PUT", D: "DELETE" };
  return {
    id,
    endpoint: CODE_ENDPOINTS[endpointCode] ?? endpointCode,
    method: methods[methodCode] ?? methodCode,
    data: data as Record<string, unknown>,
    at,
  };
}
