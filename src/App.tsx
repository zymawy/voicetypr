import { Toaster } from "sonner";
import { AppErrorBoundary } from "./components/ErrorBoundary";
import { AppContainer } from "./components/AppContainer";
import { ReadinessProvider } from "./contexts/ReadinessContext";
import { SettingsProvider } from "./contexts/SettingsContext";
import { ModelManagementProvider } from "./contexts/ModelManagementContext";

export default function App() {
  return (
    <AppErrorBoundary>
      <SettingsProvider>
        <ReadinessProvider>
          <ModelManagementProvider>
            <AppContainer />
            <Toaster position="top-center" />
          </ModelManagementProvider>
        </ReadinessProvider>
      </SettingsProvider>
    </AppErrorBoundary>
  );
}
