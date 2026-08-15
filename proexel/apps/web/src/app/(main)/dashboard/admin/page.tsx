import { Check, History, KeyRound, Minus, Pencil, Plus, ShieldCheck, UserRoundCog, UsersRound } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { INTL_LOCALES } from "@/lib/i18n/config";
import type { TranslationKey } from "@/lib/i18n/messages";
import { getI18n } from "@/lib/i18n/server";
import { requirePermission } from "@/lib/proexel/auth-server";
import { listAudit, listUsers } from "@/lib/proexel/service";
import type { UserAccount } from "@/lib/proexel/types";

import { CommandDialog } from "../_components/command-dialog";
import { PageHeader } from "../_components/page-header";
import { ProexelEmptyState } from "../_components/proexel-empty-state";

const rows: readonly [TranslationKey, boolean, boolean, boolean, boolean][] = [
  ["admin.readValves", true, true, false, true],
  ["admin.createValves", true, true, false, false],
  ["admin.editTechnical", true, false, false, false],
  ["admin.registerMaintenance", true, true, false, true],
  ["admin.manageOrders", true, true, false, false],
  ["admin.adjustStock", true, true, true, false],
  ["admin.reviewRestock", true, true, false, false],
  ["admin.manageSuppliers", true, false, false, false],
  ["admin.readAudit", true, true, false, false],
  ["admin.manageUsers", true, false, false, false],
];
const roleNames = ["admin", "chefe", "compras", "tecnico"] as const;

export const dynamic = "force-dynamic";

export default async function AdminPage() {
  const [, users, audit, { t, locale }] = await Promise.all([
    requirePermission("admin.manage"),
    listUsers(),
    listAudit({ aggregate: "user_account", page_size: 50 }),
    getI18n(),
  ]);
  const roleOptions = roleNames.map((role) => ({ value: role, label: t(`role.${role}`) }));
  return (
    <div>
      <PageHeader
        title={t("nav.admin")}
        description={t("admin.description")}
        action={
          <CommandDialog
            trigger={
              <Button>
                <Plus />
                {t("admin.newUser")}
              </Button>
            }
            title={t("admin.createUser")}
            description={t("admin.createUserDescription")}
            endpoint="/api/proexel/users"
            fields={[
              { name: "name", label: t("common.name"), required: true },
              { name: "email", label: t("common.email"), type: "email", required: true },
              { name: "role", label: t("common.role"), type: "select", required: true, options: roleOptions },
              { name: "password", label: t("admin.password"), type: "password", required: true },
              { name: "pin", label: t("admin.pinOptional"), type: "password" },
            ]}
          />
        }
      />
      <Tabs defaultValue="users" className="gap-4">
        <TabsList>
          <TabsTrigger value="users">
            <UsersRound />
            {t("admin.users")}
          </TabsTrigger>
          <TabsTrigger value="history">
            <History />
            {t("admin.userHistory")}
          </TabsTrigger>
          <TabsTrigger value="permissions">
            <ShieldCheck />
            {t("admin.permissions")}
          </TabsTrigger>
        </TabsList>
        <TabsContent value="users">
          <UsersCard users={users.items} roleOptions={roleOptions} t={t} locale={locale} />
        </TabsContent>
        <TabsContent value="history">
          <Card>
            <CardHeader>
              <CardTitle>{t("admin.userHistory")}</CardTitle>
              <CardDescription>{t("admin.userHistoryDescription")}</CardDescription>
            </CardHeader>
            <CardContent>
              {audit.items.length ? (
                <div className="overflow-x-auto">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>{t("common.date")}</TableHead>
                        <TableHead>{t("audit.actor")}</TableHead>
                        <TableHead>{t("common.operation")}</TableHead>
                        <TableHead>{t("common.details")}</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {audit.items.map((event) => (
                        <TableRow key={event.id}>
                          <TableCell>
                            {new Intl.DateTimeFormat(INTL_LOCALES[locale], {
                              dateStyle: "short",
                              timeStyle: "short",
                            }).format(new Date(event.created_at_ms))}
                          </TableCell>
                          <TableCell>{event.actor}</TableCell>
                          <TableCell>{userOperation(event.description, t)}</TableCell>
                          <TableCell>
                            <AuditChanges before={event.before_json} after={event.after_json} t={t} />
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              ) : (
                <ProexelEmptyState
                  icon={History}
                  title={t("admin.noUserHistory")}
                  description={t("admin.noUserHistoryDescription")}
                />
              )}
            </CardContent>
          </Card>
        </TabsContent>
        <TabsContent value="permissions">
          <PermissionsCard t={t} />
        </TabsContent>
      </Tabs>
    </div>
  );
}

function UsersCard({
  users,
  roleOptions,
  t,
  locale,
}: {
  users: UserAccount[];
  roleOptions: Array<{ value: string; label: string }>;
  t: Awaited<ReturnType<typeof getI18n>>["t"];
  locale: "pt" | "en" | "es" | "fr";
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("admin.users")}</CardTitle>
        <CardDescription>{t("admin.userCount", { count: users.length })}</CardDescription>
      </CardHeader>
      <CardContent>
        {users.length ? (
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t("common.name")}</TableHead>
                  <TableHead>{t("common.email")}</TableHead>
                  <TableHead>{t("common.role")}</TableHead>
                  <TableHead>{t("common.status")}</TableHead>
                  <TableHead>{t("admin.pin")}</TableHead>
                  <TableHead>{t("admin.updatedAt")}</TableHead>
                  <TableHead className="text-right">{t("common.actions")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {users.map((user) => (
                  <TableRow key={user.id}>
                    <TableCell className="font-medium">{user.name}</TableCell>
                    <TableCell>{user.email}</TableCell>
                    <TableCell>
                      <Badge variant="secondary">{t(`role.${user.role}`)}</Badge>
                    </TableCell>
                    <TableCell>
                      <Badge variant={user.active ? "outline" : "destructive"}>
                        {user.active ? t("admin.active") : t("admin.disabled")}
                      </Badge>
                    </TableCell>
                    <TableCell>{user.has_pin ? t("admin.configured") : t("common.notConfigured")}</TableCell>
                    <TableCell>
                      {user.updated_at_ms
                        ? new Intl.DateTimeFormat(INTL_LOCALES[locale], {
                            dateStyle: "short",
                            timeStyle: "short",
                          }).format(new Date(user.updated_at_ms))
                        : "-"}
                    </TableCell>
                    <TableCell>
                      <div className="flex justify-end gap-2">
                        <CommandDialog
                          trigger={
                            <Button size="icon-sm" variant="ghost" title={t("admin.editUser")}>
                              <Pencil />
                              <span className="sr-only">{t("admin.editUser")}</span>
                            </Button>
                          }
                          title={t("admin.editUser")}
                          description={t("admin.editUserDescription")}
                          endpoint="/api/proexel/users"
                          method="PATCH"
                          fields={[
                            { name: "id", label: "ID", type: "hidden", defaultValue: user.id },
                            { name: "name", label: t("common.name"), required: true, defaultValue: user.name },
                            {
                              name: "email",
                              label: t("common.email"),
                              type: "email",
                              required: true,
                              defaultValue: user.email,
                            },
                            {
                              name: "role",
                              label: t("common.role"),
                              type: "select",
                              required: true,
                              options: roleOptions,
                              defaultValue: user.role,
                            },
                            {
                              name: "active",
                              label: t("admin.activeAccount"),
                              type: "checkbox",
                              defaultValue: user.active,
                            },
                          ]}
                        />
                        <CommandDialog
                          trigger={
                            <Button size="icon-sm" variant="ghost" title={t("admin.credentials")}>
                              <KeyRound />
                              <span className="sr-only">{t("admin.credentials")}</span>
                            </Button>
                          }
                          title={t("admin.credentials")}
                          description={t("admin.credentialsDescription")}
                          endpoint="/api/proexel/users"
                          method="PUT"
                          fields={[
                            { name: "id", label: "ID", type: "hidden", defaultValue: user.id },
                            { name: "password", label: t("admin.newPassword"), type: "password" },
                            { name: "pin", label: t("admin.newPin"), type: "password" },
                            { name: "clear_pin", label: t("admin.clearPin"), type: "checkbox" },
                          ]}
                        />
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        ) : (
          <ProexelEmptyState
            icon={UserRoundCog}
            title={t("admin.noUsers")}
            description={t("admin.noUsersDescription")}
          />
        )}
      </CardContent>
    </Card>
  );
}

function PermissionsCard({ t }: { t: Awaited<ReturnType<typeof getI18n>>["t"] }) {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <ShieldCheck className="size-5" />
          <CardTitle>{t("admin.permissionsByRole")}</CardTitle>
        </div>
        <CardDescription>{t("admin.backendAuthority")}</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="overflow-x-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("common.permission")}</TableHead>
                {roleNames.map((role) => (
                  <TableHead key={role}>{t(`role.${role}`)}</TableHead>
                ))}
              </TableRow>
            </TableHeader>
            <TableBody>
              {rows.map(([name, ...roles]) => (
                <TableRow key={name}>
                  <TableCell className="font-medium">{t(name)}</TableCell>
                  {roles.map((allowed, index) => (
                    <TableCell key={`${name}-${roleNames[index]}`}>
                      <Badge variant={allowed ? "outline" : "secondary"} className="gap-1">
                        {allowed ? <Check /> : <Minus />}
                        {allowed ? t("admin.allowed") : t("admin.denied")}
                      </Badge>
                    </TableCell>
                  ))}
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
}

function AuditChanges({
  before,
  after,
  t,
}: {
  before?: string | null;
  after?: string | null;
  t: Awaited<ReturnType<typeof getI18n>>["t"];
}) {
  return (
    <details>
      <summary className="cursor-pointer text-muted-foreground">{t("admin.viewChanges")}</summary>
      <div className="mt-2 grid gap-2 lg:grid-cols-2">
        <AuditValue label={t("audit.before")} value={before} />
        <AuditValue label={t("audit.after")} value={after} />
      </div>
    </details>
  );
}

function AuditValue({ label, value }: { label: string; value?: string | null }) {
  return (
    <div>
      <strong className="text-xs">{label}</strong>
      <pre className="mt-1 max-h-48 max-w-96 overflow-auto rounded border bg-muted p-2 text-xs">{value ?? "-"}</pre>
    </div>
  );
}

function userOperation(description: string | null | undefined, t: Awaited<ReturnType<typeof getI18n>>["t"]) {
  if (description === "User created") return t("admin.userCreated");
  if (description === "User updated") return t("admin.userUpdated");
  if (description === "User credentials reset") return t("admin.userCredentialsReset");
  return description ?? "-";
}
