import { routes } from "./routes";
import { AppProvider } from "./providers/app-provider";
import { AppShell } from "../shared/components/layout/app-shell";

export function App() {
  return (
    <AppProvider>
      <AppShell routes={routes} />
    </AppProvider>
  );
}
