import type { ComponentType } from "react";

import { DatabasesPage } from "../features/databases/pages/databases-page";
import { DashboardPage } from "../features/dashboard/pages/dashboard-page";
import { LogsPage } from "../features/logs/pages/logs-page";
import { ProjectsPage } from "../features/projects/pages/projects-page";
import { RuntimesPage } from "../features/runtimes/pages/runtimes-page";
import { SecurityPage } from "../features/security/pages/security-page";
import { ServicesPage } from "../features/services/pages/services-page";
import { SettingsPage } from "../features/settings/pages/settings-page";

export type AppRouteId =
  | "dashboard"
  | "projects"
  | "services"
  | "runtimes"
  | "databases"
  | "logs"
  | "security"
  | "settings";

export interface AppRoute {
  id: AppRouteId;
  label: string;
  component: ComponentType;
}

export const routes: AppRoute[] = [
  { id: "dashboard", label: "Dashboard", component: DashboardPage },
  { id: "projects", label: "Projects", component: ProjectsPage },
  { id: "services", label: "Services", component: ServicesPage },
  { id: "runtimes", label: "Runtimes", component: RuntimesPage },
  { id: "databases", label: "Databases", component: DatabasesPage },
  { id: "logs", label: "Logs", component: LogsPage },
  { id: "security", label: "Security", component: SecurityPage },
  { id: "settings", label: "Settings", component: SettingsPage },
];
