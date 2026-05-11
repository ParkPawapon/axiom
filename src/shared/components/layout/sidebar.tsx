import { navigationConfig } from "../../../core/config/navigation.config";

interface SidebarRoute {
  id: string;
  label: string;
}

interface SidebarProps {
  activeRouteId: string;
  routes: SidebarRoute[];
  onNavigate: (routeId: string) => void;
}

export function Sidebar({ activeRouteId, routes, onNavigate }: SidebarProps) {
  return (
    <aside className="border-r border-voicebox-border bg-white">
      <nav aria-label={navigationConfig.primaryLabel} className="flex flex-col gap-1 p-3">
        {routes.map((route) => {
          const isActive = route.id === activeRouteId;

          return (
            <button
              aria-current={isActive ? "page" : undefined}
              className={`border px-3 py-3 text-left text-sm font-bold transition-colors ${
                isActive
                  ? "border-voicebox-black bg-voicebox-black text-white"
                  : "border-transparent bg-white text-voicebox-secondary hover:border-voicebox-border hover:text-voicebox-black"
              }`}
              key={route.id}
              onClick={() => onNavigate(route.id)}
              type="button"
            >
              {route.label}
            </button>
          );
        })}
      </nav>
    </aside>
  );
}
