import type { ComponentType } from "react";

import { ControlCenterPage } from "../features/control-center/pages/control-center-page";
import { DatabasesPage } from "../features/databases/pages/databases-page";
import { DashboardPage } from "../features/dashboard/pages/dashboard-page";
import { ProjectsPage } from "../features/projects/pages/projects-page";
import { RuntimesPage } from "../features/runtimes/pages/runtimes-page";
import { SecurityPage } from "../features/security/pages/security-page";
import { ServicesPage } from "../features/services/pages/services-page";
import { SettingsPage } from "../features/settings/pages/settings-page";
import { SetupWizardPage } from "../features/setup-wizard/pages/setup-wizard-page";
import { UnifiedLogsPage } from "../features/unified-logs/pages/unified-logs-page";

export type AppRouteId =
  | "control-center"
  | "dashboard"
  | "projects"
  | "services"
  | "runtimes"
  | "databases"
  | "setup"
  | "logs"
  | "security"
  | "settings";

export interface AppRoute {
  id: AppRouteId;
  label: string;
  component: ComponentType;
}

export const routes: AppRoute[] = [
  { id: "control-center", label: "Control Center", component: ControlCenterPage },
  { id: "setup", label: "Setup", component: SetupWizardPage },
  { id: "projects", label: "Projects", component: ProjectsPage },
  { id: "services", label: "Services", component: ServicesPage },
  { id: "runtimes", label: "Runtimes", component: RuntimesPage },
  { id: "databases", label: "Databases", component: DatabasesPage },
  { id: "logs", label: "Logs", component: UnifiedLogsPage },
  { id: "dashboard", label: "Advanced Dashboard", component: DashboardPage },
  { id: "security", label: "Security", component: SecurityPage },
  { id: "settings", label: "Settings", component: SettingsPage },
];
