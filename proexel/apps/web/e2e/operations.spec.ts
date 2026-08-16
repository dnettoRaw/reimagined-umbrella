import { expect, type Page, test } from "@playwright/test";

const password = process.env.PROEXEL_E2E_PASSWORD ?? "";

async function login(page: Page, email: string, credential = password) {
  await page.goto("/auth/login");
  await page.getByLabel("Email").fill(email);
  await page.getByLabel("Palavra-passe ou PIN").fill(credential);
  await page.getByRole("button", { name: "Entrar" }).click();
  await expect(page).toHaveURL(/\/dashboard\/overview/);
}

async function logout(page: Page) {
  await page.request.post("/api/auth/logout");
  await page.goto("/auth/login");
}

async function createResource(page: Page, endpoint: string, body: Record<string, unknown>) {
  const response = await page.request.post(endpoint, { data: body });
  const result = (await response.json()) as { accepted?: boolean; message?: string; resource_id?: string };
  expect(response.ok(), result.message).toBe(true);
  expect(result.accepted, result.message).toBe(true);
  expect(result.resource_id).toBeTruthy();
  return result.resource_id as string;
}

test("representative roles complete the machine maintenance workflow", async ({ page }) => {
  test.setTimeout(180_000);
  test.skip(!password, "run proexel/scripts/e2e.sh to provide ephemeral credentials");
  const suffix = Date.now();
  await login(page, "admin-e2e@proexel.local");

  const categoryId = await createResource(page, "/api/proexel/categories", {
    code: `MOTOR-${suffix}`,
    name: "Motor elétrico E2E",
    description: "Categoria de validação do fluxo guiado",
    default_complexity_level: 2,
    active: true,
    custom_field_definitions: [],
    recommended_parts: [],
    maintenance_guide: {
      version: 1,
      steps: [
        {
          id: `confirm-${suffix}`,
          title: "Confirmar identificação",
          description: null,
          instructions: "Confirme o código e a unidade física instalada.",
          step_type: "confirmation",
          required: true,
          reference_photo_ids: [],
          safety_warning: null,
          expected_value: null,
          options: [],
          order: 0,
        },
      ],
    },
  });
  const machineId = await createResource(page, "/api/proexel/machines", {
    code: `M-${suffix}`,
    name: "Prensa E2E",
    description: "Máquina do cenário E2E",
    zone: "Zona E2E",
    location: "Linha de testes",
    manufacturer: "PROEXEL Test",
    model: "PX-E2E",
    serial_number: `SN-${suffix}`,
    active: true,
  });
  const itemId = await createResource(page, "/api/proexel/machine-items", {
    machine_id: machineId,
    category_id: categoryId,
    name: "Motor principal E2E",
    code: `ITEM-${suffix}`,
    complexity_level: 2,
    status: "unknown",
    custom_field_values: {},
    installed_component: {
      manufacturer: "WEG",
      model: "W22",
      part_number: "W22-E2E",
      serial_number: `UNIT-${suffix}`,
      installed_at: "2026-08-15",
      technical_specifications: { voltage: "400 V" },
    },
    replacement_specification: {
      manufacturer: "WEG",
      model: "W22",
      part_number: "W22-E2E",
      serial_number: null,
      technical_specifications: { voltage: "400 V" },
      compatibility_notes: "Montagem B3",
      equivalent_parts: [],
      supplier_reference: "SUP-E2E",
      photo_ids: [],
    },
  });

  await page.goto(`/dashboard/machines/${machineId}`);
  await expect(page.getByRole("heading", { name: /Prensa E2E/ }).first()).toBeVisible();
  await expect(page.getByText("Motor principal E2E").first()).toBeVisible();
  const upload = page.locator('input[type="file"]').first();
  await upload.setInputFiles({
    name: "machine.png",
    mimeType: "image/png",
    buffer: Buffer.from(
      "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=",
      "base64",
    ),
  });
  await expect(page.getByText("Operação concluída")).toBeVisible();

  const orderId = await createResource(page, "/api/proexel/orders", {
    machine_id: machineId,
    all_items: false,
    item_ids: [itemId],
    description: "Inspeção guiada E2E",
    priority: "normal",
    scheduled_for: null,
    assigned_operator_id: "e2e-tecnico",
  });

  await logout(page);
  await login(page, "tecnico-e2e@proexel.local");
  await page.goto(`/dashboard/execution/${orderId}`);
  await page.getByRole("button", { name: "Iniciar ordem" }).click();
  await page.getByRole("button", { name: "Iniciar inspeção" }).click();
  const nextStep = page.getByRole("button", { name: "Seguinte" });
  await expect(nextStep).toBeDisabled();
  await page.getByRole("button", { name: "Feito" }).click();
  await expect(nextStep).toBeEnabled();
  await nextStep.click();
  const inspectionPhoto = page.locator('input[type="file"]');
  await page.getByLabel("Finalidade").selectOption("before");
  await inspectionPhoto.setInputFiles({
    name: "before.png",
    mimeType: "image/png",
    buffer: Buffer.from(
      "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=",
      "base64",
    ),
  });
  await expect(nextStep).toBeEnabled();
  await nextStep.click();
  await page.getByRole("button", { name: "Feito" }).click();
  await nextStep.click();
  await page.getByLabel("Finalidade").selectOption("after");
  await inspectionPhoto.setInputFiles({
    name: "after.png",
    mimeType: "image/png",
    buffer: Buffer.from(
      "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=",
      "base64",
    ),
  });
  await expect(nextStep).toBeEnabled();
  await nextStep.click();
  await expect(page.getByRole("heading", { name: "Resultado" })).toBeInViewport();
  await page.getByRole("button", { name: "Concluir inspeção" }).click();
  await expect(page.getByText("1 de 1 concluídos")).toBeVisible();
  await page.getByRole("button", { name: "Concluir ordem" }).click();

  await page.goto("/dashboard/purchasing");
  await page.getByRole("button", { name: "Solicitar reposição" }).click();
  await page.getByLabel("Referência", { exact: true }).fill("PECA E2E");
  await page.getByLabel("Motivo").fill("Reposição E2E");
  await page.getByRole("button", { name: "Confirmar" }).click();
  await expect(page.getByText("Operação concluída")).toBeVisible();

  await logout(page);
  await login(page, "chefe-e2e@proexel.local");
  await expect
    .poll(async () => {
      const response = await page.request.get("/api/proexel/purchasing");
      const result = (await response.json()) as { items?: Array<{ reason?: string }> };
      return result.items?.some((item) => item.reason === "Reposição E2E") ?? false;
    })
    .toBe(true);
  await page.goto("/dashboard/purchasing");
  const requestRow = page.getByRole("row").filter({ hasText: "Reposição E2E" });
  await requestRow.getByRole("button", { name: "Aprovar" }).click();
  await expect(requestRow).toContainText("Aprovada");

  await logout(page);
  await login(page, "admin-e2e@proexel.local");
  await page.goto("/dashboard/reports");
  const downloadPromise = page.waitForEvent("download");
  await page.getByRole("button", { name: "Exportar PDF" }).click();
  expect((await downloadPromise).suggestedFilename()).toMatch(/\.pdf$/);

  await page.goto("/dashboard/admin");
  await page.getByRole("button", { name: "Novo usuário" }).click();
  const userDialog = page.getByRole("dialog");
  await userDialog.getByLabel("Nome").fill("Técnico PIN E2E");
  await userDialog.getByLabel("Email").fill(`pin-${suffix}@proexel.local`);
  await userDialog.getByLabel("Papel").selectOption("tecnico");
  await userDialog.getByLabel("Nível técnico máximo").selectOption("2");
  await userDialog.getByLabel("Palavra-passe inicial").fill("PasswordE2E123!");
  await userDialog.getByLabel("PIN opcional (4 a 8 dígitos)").fill("2468");
  await userDialog.getByRole("button", { name: "Confirmar" }).click();
  await expect(page.getByRole("row").filter({ hasText: `pin-${suffix}@proexel.local` })).toBeVisible();
  await page.getByRole("tab", { name: "Histórico de usuários" }).click();
  await expect(page.getByText("Usuário criado")).toBeVisible();

  await logout(page);
  await login(page, `pin-${suffix}@proexel.local`, "2468");
  await expect(page.getByRole("heading", { name: "Visão geral" })).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/dashboard/overview");
  await expect(page.getByRole("heading", { name: "Visão geral" })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
});
