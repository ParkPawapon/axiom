import type { ComponentType } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";

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

function getRouteIdFromHash(routes: ShellRoute[]) {
  const routeId = globalThis.location?.hash.replace("#", "");

  if (routeId && routes.some((route) => route.id === routeId)) {
    return routeId;
  }

  return routes[0]?.id ?? "";
}

export function AppShell({ routes }: AppShellProps) {
  const [activeRouteId, setActiveRouteId] = useState(() => getRouteIdFromHash(routes));

  const activeRoute = useMemo(
    () => routes.find((route) => route.id === activeRouteId) ?? routes[0],
    [activeRouteId, routes],
  );

  useEffect(() => {
    function handleHashChange() {
      setActiveRouteId(getRouteIdFromHash(routes));
    }

    globalThis.addEventListener("hashchange", handleHashChange);

    return () => globalThis.removeEventListener("hashchange", handleHashChange);
  }, [routes]);

  const handleNavigate = useCallback((routeId: string) => {
    setActiveRouteId(routeId);

    if (globalThis.location.hash !== `#${routeId}`) {
      globalThis.location.hash = routeId;
    }
  }, []);

  if (!activeRoute) {
    return null;
  }

  const ActivePage = activeRoute.component;

  return (
    <div className="min-h-screen bg-voicebox-background text-voicebox-black">
      <TopBar
        activeRouteLabel={activeRoute.label}
        appName={appConfig.name}
        tagline={appConfig.tagline}
      />
      <div className="grid min-h-[calc(100vh-4rem)] border-t border-voicebox-border lg:grid-cols-[16rem_1fr]">
        <Sidebar activeRouteId={activeRoute.id} onNavigate={handleNavigate} routes={routes} />
        <main aria-label={ariaLabels.mainContent} className="min-w-0 p-4 md:p-6">
          <ActivePage />
        </main>
      </div>
    </div>
  );
}
