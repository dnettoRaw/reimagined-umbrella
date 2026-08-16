"use client";

import { useEffect } from "react";

import { RotateCcw } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useI18n } from "@/lib/i18n/provider";

import {
  applyDemoOperations,
  createDemoState,
  DEMO_OPERATIONS_COOKIE,
  DEMO_OPERATIONS_KEY,
  DEMO_STATE_KEY,
  type DemoOperation,
  decodeDemoOperations,
  encodeDemoOperations,
  sanitizeDemoOperationData,
} from "./demo-data";

const COOKIE_LIMIT = 3_700;

export function DemoClientBridge() {
  useEffect(() => {
    const originalFetch = window.fetch.bind(window);
    synchronizeStorage();

    window.fetch = async (input, init) => {
      const request = input instanceof Request ? input : null;
      const url = new URL(request?.url ?? String(input), window.location.origin);
      const method = (init?.method ?? request?.method ?? "GET").toUpperCase();
      if (!url.pathname.startsWith("/api/proexel/") || method === "GET") return originalFetch(input, init);

      if (url.pathname === "/api/proexel/attachments" && method === "POST") {
        return jsonResponse({ ref: `machine-photos/${crypto.randomUUID()}.png` });
      }

      const data = sanitizeDemoOperationData(url.pathname, await requestData(request, init));
      const operation: DemoOperation = {
        id: demoOperationId(),
        endpoint: url.pathname,
        method,
        data,
        at: Date.now(),
      };
      const operations = readOperations();
      operations.push(operation);
      persist(operations);
      return jsonResponse({ accepted: true, message: "demo", resource_id: `demo-${operation.id}` });
    };

    return () => {
      window.fetch = originalFetch;
    };
  }, []);

  return null;
}

export function DemoResetButton() {
  const { t } = useI18n();
  function reset() {
    localStorage.removeItem(DEMO_STATE_KEY);
    localStorage.removeItem(DEMO_OPERATIONS_KEY);
    // biome-ignore lint/suspicious/noDocumentCookie: the server-rendered demo reads its compact operation log from this cookie
    document.cookie = `${DEMO_OPERATIONS_COOKIE}=; Path=/; Max-Age=0; SameSite=Lax`;
    window.location.assign("/dashboard/overview");
  }
  return (
    <>
      <Badge variant="secondary" className="hidden md:inline-flex">
        {t("demo.label")}
      </Badge>
      <Button type="button" variant="outline" size="icon" onClick={reset} title={t("demo.reset")}>
        <RotateCcw />
        <span className="sr-only">{t("demo.reset")}</span>
      </Button>
    </>
  );
}

function synchronizeStorage() {
  const cookieOperations = decodeDemoOperations(readCookie(DEMO_OPERATIONS_COOKIE));
  const localOperations = readOperations();
  const operations = localOperations.length >= cookieOperations.length ? localOperations : cookieOperations;
  persist(operations);
}

function readOperations(): DemoOperation[] {
  try {
    const stored = localStorage.getItem(DEMO_OPERATIONS_KEY);
    const operations = stored ? (JSON.parse(stored) as DemoOperation[]) : [];
    return Array.isArray(operations) ? operations : [];
  } catch {
    return [];
  }
}

function persist(input: DemoOperation[]) {
  const operations = compactCookieOperations(input);
  let encoded = encodeDemoOperations(operations);
  while (encoded.length > COOKIE_LIMIT && operations.length > 1) {
    operations.shift();
    encoded = encodeDemoOperations(operations);
  }
  localStorage.setItem(DEMO_OPERATIONS_KEY, JSON.stringify(input));
  localStorage.setItem(DEMO_STATE_KEY, JSON.stringify(applyDemoOperations(createDemoState(), input)));
  // biome-ignore lint/suspicious/noDocumentCookie: the server-rendered demo reads its compact operation log from this cookie
  document.cookie = `${DEMO_OPERATIONS_COOKIE}=${encoded}; Path=/; Max-Age=2592000; SameSite=Lax`;
}

function compactCookieOperations(input: DemoOperation[]) {
  const operations = [...input];
  let encoded = encodeDemoOperations(operations);
  while (encoded.length > COOKIE_LIMIT) {
    const optional = operations.findIndex((operation) => operation.endpoint === "/api/proexel/photos");
    if (optional === -1) break;
    operations.splice(optional, 1);
    encoded = encodeDemoOperations(operations);
  }
  return operations;
}

function demoOperationId() {
  const random = crypto.getRandomValues(new Uint32Array(1))[0].toString(36);
  return `${Date.now().toString(36)}-${random}`;
}

async function requestData(request: Request | null, init?: RequestInit): Promise<Record<string, unknown>> {
  const body = init?.body;
  if (typeof body === "string") return parseJson(body);
  if (body instanceof FormData) return Object.fromEntries(body.entries());
  if (request) {
    const contentType = request.headers.get("content-type") ?? "";
    if (contentType.includes("application/json")) return parseJson(await request.clone().text());
    if (contentType.includes("multipart/form-data"))
      return Object.fromEntries((await request.clone().formData()).entries());
  }
  return {};
}

function parseJson(value: string): Record<string, unknown> {
  try {
    const parsed = JSON.parse(value);
    return parsed && typeof parsed === "object" ? (parsed as Record<string, unknown>) : {};
  } catch {
    return {};
  }
}

function readCookie(name: string) {
  const prefix = `${name}=`;
  return document.cookie
    .split("; ")
    .find((item) => item.startsWith(prefix))
    ?.slice(prefix.length);
}

function jsonResponse(data: Record<string, unknown>) {
  return new Response(JSON.stringify(data), { status: 200, headers: { "content-type": "application/json" } });
}
