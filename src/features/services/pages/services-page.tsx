import { useCallback, useEffect, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { WarningPanel } from "../../../shared/components/feedback/warning-panel";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";
import {
  getServiceStatus,
  listServices,
  restartService,
  startService,
  stopService,
} from "../api/service.commands";
import { ServiceControlPanel } from "../components/service-control-panel";
import type { ManagedService, ServiceAction } from "../types/service.types";

interface BusyState {
  serviceId: string;
  action: ServiceAction | "check";
}

function getErrorMessage(error: unknown) {
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = (error as { message?: unknown }).message;

    if (typeof message === "string" && message.trim().length > 0) {
      return message;
    }
  }

  return "Service command failed safely. Check the application logs for details.";
}

export function ServicesPage() {
  const [services, setServices] = useState<ManagedService[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string>();
  const [noticeMessage, setNoticeMessage] = useState<string>();
  const [busyState, setBusyState] = useState<BusyState>();

  const loadServices = useCallback(async () => {
    setIsLoading(true);
    setErrorMessage(undefined);

    try {
      setServices(await listServices());
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadServices();
  }, [loadServices]);

  const replaceService = useCallback((updatedService: ManagedService) => {
    setServices((currentServices) =>
      currentServices.map((service) =>
        service.id === updatedService.id ? updatedService : service,
      ),
    );
  }, []);

  const handleCheck = useCallback(
    async (serviceId: string) => {
      setBusyState({ serviceId, action: "check" });
      setErrorMessage(undefined);
      setNoticeMessage(undefined);

      try {
        const service = await getServiceStatus(serviceId);
        replaceService(service);
        setNoticeMessage(`${service.name} status checked.`);
      } catch (error) {
        setErrorMessage(getErrorMessage(error));
      } finally {
        setBusyState(undefined);
      }
    },
    [replaceService],
  );

  const runAction = useCallback(
    async (
      serviceId: string,
      action: ServiceAction,
      command: (serviceId: string) => Promise<{ service: ManagedService; message: string }>,
    ) => {
      setBusyState({ serviceId, action });
      setErrorMessage(undefined);
      setNoticeMessage(undefined);

      try {
        const outcome = await command(serviceId);
        replaceService(outcome.service);
        setNoticeMessage(outcome.message);
      } catch (error) {
        setErrorMessage(getErrorMessage(error));
      } finally {
        setBusyState(undefined);
      }
    },
    [replaceService],
  );

  return (
    <PageShell
      title="Services"
      description="Service lifecycle requests are executed only through Rust backend adapters with allowlisted OS commands and platform-specific drivers."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {noticeMessage ? <WarningPanel message={noticeMessage} /> : null}
      {isLoading ? <LoadingState label="Loading service inventory" /> : null}
      {!isLoading && services.length === 0 ? (
        <EmptyState
          title="No services registered"
          description="The backend returned no service definitions. Check the service manager configuration."
        />
      ) : null}
      {services.length > 0 ? (
        <ServiceControlPanel
          busyAction={busyState?.action}
          busyServiceId={busyState?.serviceId}
          isLoading={isLoading}
          onCheck={handleCheck}
          onRefresh={loadServices}
          onRestart={(serviceId) => void runAction(serviceId, "restart", restartService)}
          onStart={(serviceId) => void runAction(serviceId, "start", startService)}
          onStop={(serviceId) => void runAction(serviceId, "stop", stopService)}
          services={services}
        />
      ) : null}
    </PageShell>
  );
}
