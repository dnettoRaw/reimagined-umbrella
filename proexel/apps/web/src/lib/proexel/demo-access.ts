import type { ComplexityLevel, Role } from "./types";

export const DEMO_ACCESS_COOKIE = "proexel_demo_access";
export const DEMO_PROFILE_COOKIE = "proexel_demo_profile";
export const DEFAULT_DEMO_PASSWORD = "proexel-demo";

export interface DemoProfile {
  id: string;
  email: string;
  name: string;
  role: Role;
  maximumRepairLevel: ComplexityLevel;
}

export const DEMO_PROFILES: DemoProfile[] = [
  {
    id: "user-1",
    email: "admin@proexel.demo",
    name: "Marcos Silva",
    role: "admin",
    maximumRepairLevel: 5,
  },
  {
    id: "user-2",
    email: "chefe@proexel.demo",
    name: "Rui Martins",
    role: "chefe",
    maximumRepairLevel: 5,
  },
  {
    id: "user-3",
    email: "compras@proexel.demo",
    name: "Laura Pinto",
    role: "compras",
    maximumRepairLevel: 3,
  },
  {
    id: "user-4",
    email: "tecnico@proexel.demo",
    name: "Carlos Mendes",
    role: "tecnico",
    maximumRepairLevel: 4,
  },
];

export function findDemoProfile(id: string | undefined) {
  return DEMO_PROFILES.find((profile) => profile.id === id);
}

export function getDemoPassword() {
  return process.env.PROEXEL_DEMO_PASSWORD?.trim() || DEFAULT_DEMO_PASSWORD;
}

export async function createDemoAccessToken(password: string) {
  const payload = new TextEncoder().encode(`proexel-demo-access:${password}`);
  const digest = await crypto.subtle.digest("SHA-256", payload);
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join("");
}
