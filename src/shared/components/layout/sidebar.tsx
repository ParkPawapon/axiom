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
    <aside className="border-b border-voicebox-border bg-white lg:border-b-0 lg:border-r">
      <nav
        aria-label={navigationConfig.primaryLabel}
        className="flex gap-1 overflow-x-auto p-3 lg:flex-col lg:overflow-x-visible"
      >
        {routes.map((route) => {
          const isActive = route.id === activeRouteId;

          return (
            <button
              aria-current={isActive ? "page" : undefined}
              className={`min-w-max border px-3 py-3 text-left text-sm font-bold transition-colors lg:min-w-0 ${
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
