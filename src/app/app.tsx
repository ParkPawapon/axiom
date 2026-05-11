import { AppProvider } from "./providers/app-provider";
import { routes } from "./routes";
import { AppShell } from "../shared/components/layout/app-shell";

export default function App() {
  return (
    <AppProvider>
      <AppShell routes={routes} />
    </AppProvider>
  );
}
