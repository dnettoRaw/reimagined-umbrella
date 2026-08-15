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

test("representative roles complete the operational workflow", async ({ page }) => {
  test.setTimeout(90_000);
  test.skip(!password, "run proexel/scripts/e2e.sh to provide ephemeral credentials");
  const tag = `E2E ${Date.now()}`;
  await login(page, "admin-e2e@proexel.local");

  await page.goto("/dashboard/valves");
  await page.getByRole("button", { name: "Nova válvula" }).click();
  await page.getByLabel("TAG").fill(tag);
  await page.getByLabel("Zona").fill("Zona E2E");
  await page.getByLabel("Fabricante").fill("PROEXEL Test");
  await page.getByLabel("Referência do kit").fill("KIT E2E");
  await page.getByRole("button", { name: "Confirmar" }).click();
  await expect(page.getByRole("cell", { name: tag })).toBeVisible();

  await page.getByTitle("Detalhes").click();
  await expect(page.getByRole("heading", { name: tag })).toBeVisible();
  const upload = page.locator('input[type="file"]');
  await upload.setInputFiles({
    name: "valve.png",
    mimeType: "image/png",
    buffer: Buffer.from(
      "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=",
      "base64",
    ),
  });
  await expect(page.getByText("Foto adicionada.")).toBeVisible();

  await page.goto("/dashboard/stock");
  const stockRow = page.getByRole("row").filter({ hasText: "KIT E2E" });
  await stockRow.getByRole("button", { name: "Ajustar" }).click();
  await page.getByLabel("Variação").fill("2");
  await page.getByLabel("Motivo").fill("Preparação E2E");
  await page.getByRole("button", { name: "Confirmar" }).click();
  await expect(stockRow).toContainText("2");

  await page.goto("/dashboard/maintenance");
  await page.getByRole("button", { name: "Iniciar manutenção" }).click();
  const dialog = page.getByRole("dialog");
  await dialog
    .locator("select")
    .first()
    .selectOption({ label: `${tag} · Zona E2E` });
  await dialog.getByRole("button", { name: "Seguinte" }).click();
  await dialog.locator("textarea").first().fill("Inspeção e troca E2E");
  await dialog.getByLabel("Houve troca de kit").check();
  await dialog.getByRole("button", { name: "Seguinte" }).click();
  const canvas = dialog.locator("canvas");
  const box = await canvas.boundingBox();
  if (!box) throw new Error("signature canvas is not visible");
  await page.mouse.move(box.x + 30, box.y + 80);
  await page.mouse.down();
  await page.mouse.move(box.x + 180, box.y + 120, { steps: 8 });
  await page.mouse.move(box.x + 320, box.y + 70, { steps: 8 });
  await page.mouse.up();
  await dialog.getByRole("button", { name: "Seguinte" }).click();
  await dialog.getByRole("button", { name: "Confirmar" }).click();
  await expect(page.getByText("Manutenção registrada com sucesso.")).toBeVisible();
  await expect(page.getByRole("row").filter({ hasText: tag })).toContainText("Consumido");

  await page.goto("/dashboard/orders");
  await page.getByRole("button", { name: "Nova OS" }).click();
  await page.getByLabel("Zona").fill("Zona E2E");
  await page.getByLabel("Válvula (opcional)").selectOption({ label: tag });
  await page.getByLabel("Descrição").fill("Ordem E2E");
  await page.getByRole("button", { name: "Confirmar" }).click();
  await expect(page.getByRole("row").filter({ hasText: "Ordem E2E" })).toBeVisible();

  await page.goto("/dashboard/reports");
  const downloadPromise = page.waitForEvent("download");
  await page.getByRole("button", { name: "Exportar PDF" }).click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toMatch(/\.pdf$/);

  await logout(page);
  await login(page, "tecnico-e2e@proexel.local");
  await page.goto("/dashboard/purchasing");
  await page.getByRole("button", { name: "Solicitar reposição" }).click();
  await page.getByLabel("Referência", { exact: true }).fill("KIT E2E");
  await page.getByLabel("Motivo").fill("Reposição E2E");
  await page.getByRole("button", { name: "Confirmar" }).click();

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
  await page.goto("/dashboard/admin");
  await page.getByRole("button", { name: "Novo usuário" }).click();
  const userDialog = page.getByRole("dialog");
  await userDialog.getByLabel("Nome").fill("Técnico PIN E2E");
  await userDialog.getByLabel("Email").fill("pin-e2e@proexel.local");
  await userDialog.getByLabel("Papel").selectOption("tecnico");
  await userDialog.getByLabel("Palavra-passe inicial").fill("PasswordE2E123!");
  await userDialog.getByLabel("PIN opcional (4 a 8 dígitos)").fill("2468");
  await userDialog.getByRole("button", { name: "Confirmar" }).click();
  await expect(page.getByRole("row").filter({ hasText: "pin-e2e@proexel.local" })).toBeVisible();
  await page.getByRole("tab", { name: "Histórico de usuários" }).click();
  await expect(page.getByText("Usuário criado")).toBeVisible();

  await logout(page);
  await login(page, "pin-e2e@proexel.local", "2468");
  await expect(page.getByRole("heading", { name: "Visão geral" })).toBeVisible();

  await logout(page);
  await login(page, "admin-e2e@proexel.local");
  await page.goto("/dashboard/admin");
  const managedUser = page.getByRole("row").filter({ hasText: "pin-e2e@proexel.local" });
  await managedUser.getByTitle("Editar usuário").click();
  const editUserDialog = page.getByRole("dialog");
  await editUserDialog.getByLabel("Conta ativa").uncheck();
  await editUserDialog.getByRole("button", { name: "Confirmar" }).click();
  await expect(managedUser).toContainText("Desativado");

  await logout(page);
  await page.getByLabel("Email").fill("pin-e2e@proexel.local");
  await page.getByLabel("Palavra-passe ou PIN").fill("2468");
  await page.getByRole("button", { name: "Entrar" }).click();
  await expect(page.getByText("Email ou palavra-passe inválidos.")).toBeVisible();

  await login(page, "admin-e2e@proexel.local");

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/dashboard/overview");
  await expect(page.getByRole("heading", { name: "Visão geral" })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
});
