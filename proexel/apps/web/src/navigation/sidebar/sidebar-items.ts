import {
  Activity,
  BarChart3,
  Building2,
  Calendar,
  History,
  LayoutDashboard,
  type LucideIcon,
  PackageSearch,
  Settings,
  ShieldCheck,
  ShoppingCart,
  Wrench,
} from "lucide-react";

import type { Role } from "@/lib/proexel/types";

export interface NavSubItem {
  title: string;
  url: string;
  icon?: LucideIcon;
  comingSoon?: boolean;
  newTab?: boolean;
  isNew?: boolean;
}

export interface NavMainItem {
  title: string;
  url: string;
  icon?: LucideIcon;
  subItems?: NavSubItem[];
  comingSoon?: boolean;
  newTab?: boolean;
  isNew?: boolean;
  roles?: Role[];
}

export interface NavGroup {
  id: number;
  label?: string;
  items: NavMainItem[];
}

export const sidebarItems: NavGroup[] = [
  {
    id: 1,
    label: "Operação",
    items: [
      {
        title: "Visão geral",
        url: "/dashboard/overview",
        icon: LayoutDashboard,
      },
      {
        title: "Válvulas",
        url: "/dashboard/valves",
        icon: Activity,
        roles: ["admin", "chefe", "tecnico"],
      },
      {
        title: "Manutenção",
        url: "/dashboard/maintenance",
        icon: Wrench,
        roles: ["admin", "chefe", "tecnico"],
      },
      {
        title: "Ordens de serviço",
        url: "/dashboard/orders",
        icon: Calendar,
        roles: ["admin", "chefe", "tecnico"],
      },
    ],
  },
  {
    id: 2,
    label: "Suprimentos",
    items: [
      {
        title: "Estoque",
        url: "/dashboard/stock",
        icon: PackageSearch,
        roles: ["admin", "chefe", "compras"],
      },
      {
        title: "Compras",
        url: "/dashboard/purchasing",
        icon: ShoppingCart,
        roles: ["admin", "chefe", "compras", "tecnico"],
      },
      {
        title: "Fornecedores",
        url: "/dashboard/suppliers",
        icon: Building2,
        roles: ["admin"],
      },
    ],
  },
  {
    id: 3,
    label: "Controle",
    items: [
      {
        title: "Histórico / Auditoria",
        url: "/dashboard/audit",
        icon: History,
        roles: ["admin", "chefe"],
      },
      {
        title: "Relatórios",
        url: "/dashboard/reports",
        icon: BarChart3,
        roles: ["admin", "chefe"],
      },
      {
        title: "Administração",
        url: "/dashboard/admin",
        icon: ShieldCheck,
        roles: ["admin"],
      },
      {
        title: "Configurações",
        url: "/dashboard/settings",
        icon: Settings,
      },
    ],
  },
];
