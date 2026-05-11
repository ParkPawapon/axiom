import type { ComponentType } from "react";
import { useMemo, useState } from "react";

import { ariaLabels } from "../../../core/accessibility/aria-labels";
import { appConfig } from "../../../core/config/app.config";
import { Sidebar } from "./sidebar";
import { TopBar } from "./top-bar";

export interface ShellRoute {
  id: string;
  label: string;
  component: ComponentType;
}

interface AppShellProps {
  routes: ShellRoute[];
}

export function AppShell({ routes }: AppShellProps) {
  const [activeRouteId, setActiveRouteId] = useState(routes[0]?.id ?? "");

  const activeRoute = useMemo(
    () => routes.find((route) => route.id === activeRouteId) ?? routes[0],
    [activeRouteId, routes],
  );

  if (!activeRoute) {
    return null;
  }

  const ActivePage = activeRoute.component;

  return (
    <div className="min-h-screen bg-voicebox-background text-voicebox-black">
      <TopBar appName={appConfig.name} />
      <div className="grid min-h-[calc(100vh-4rem)] grid-cols-[16rem_1fr] border-t border-voicebox-border">
        <Sidebar activeRouteId={activeRoute.id} onNavigate={setActiveRouteId} routes={routes} />
        <main aria-label={ariaLabels.mainContent} className="min-w-0 p-6">
          <ActivePage />
        </main>
      </div>
    </div>
  );
}
