import {
  BarChart3,
  Bell,
  Building2,
  Calendar,
  ClipboardCheck,
  Factory,
  History,
  LayoutDashboard,
  type LucideIcon,
  PackageSearch,
  Settings,
  ShieldCheck,
  ShoppingCart,
  Tags,
} from "lucide-react";

import type { TranslationKey } from "@/lib/i18n/messages";
import type { Role } from "@/lib/proexel/types";

export interface NavSubItem {
  title: TranslationKey;
  url: string;
  icon?: LucideIcon;
  comingSoon?: boolean;
  newTab?: boolean;
  isNew?: boolean;
}

export interface NavMainItem {
  title: TranslationKey;
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
  label?: TranslationKey;
  items: NavMainItem[];
}

export const sidebarItems: NavGroup[] = [
  {
    id: 1,
    label: "nav.operation",
    items: [
      {
        title: "nav.overview",
        url: "/dashboard/overview",
        icon: LayoutDashboard,
      },
      {
        title: "nav.machines",
        url: "/dashboard/machines",
        icon: Factory,
        roles: ["admin", "chefe", "tecnico"],
      },
      {
        title: "nav.categories",
        url: "/dashboard/categories",
        icon: Tags,
        roles: ["admin"],
      },
      {
        title: "nav.execution",
        url: "/dashboard/execution",
        icon: ClipboardCheck,
        roles: ["admin", "chefe", "tecnico"],
      },
      {
        title: "nav.orders",
        url: "/dashboard/orders",
        icon: Calendar,
        roles: ["admin", "chefe", "tecnico"],
      },
      {
        title: "nav.notifications",
        url: "/dashboard/notifications",
        icon: Bell,
      },
    ],
  },
  {
    id: 2,
    label: "nav.supplies",
    items: [
      {
        title: "nav.stock",
        url: "/dashboard/stock",
        icon: PackageSearch,
        roles: ["admin", "chefe", "compras"],
      },
      {
        title: "nav.purchasing",
        url: "/dashboard/purchasing",
        icon: ShoppingCart,
        roles: ["admin", "chefe", "compras", "tecnico"],
      },
      {
        title: "nav.suppliers",
        url: "/dashboard/suppliers",
        icon: Building2,
        roles: ["admin"],
      },
    ],
  },
  {
    id: 3,
    label: "nav.control",
    items: [
      {
        title: "nav.audit",
        url: "/dashboard/audit",
        icon: History,
        roles: ["admin", "chefe"],
      },
      {
        title: "nav.reports",
        url: "/dashboard/reports",
        icon: BarChart3,
        roles: ["admin", "chefe"],
      },
      {
        title: "nav.admin",
        url: "/dashboard/admin",
        icon: ShieldCheck,
        roles: ["admin"],
      },
      {
        title: "nav.settings",
        url: "/dashboard/settings",
        icon: Settings,
      },
    ],
  },
];
