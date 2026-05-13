import { useCallback, useEffect, useMemo, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import { listServices } from "../../services/api/service.commands";
import type { ManagedService } from "../../services/types/service.types";
import { DatabaseServiceCard } from "../components/database-service-card";
import { MysqlStatusPanel } from "../components/mysql-status-panel";
import { PostgresStatusPanel } from "../components/postgres-status-panel";

export function DatabasesPage() {
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isLoading, setIsLoading] = useState(true);
  const [services, setServices] = useState<ManagedService[]>([]);

  const loadDatabaseServices = useCallback(async () => {
    setIsLoading(true);
    setErrorMessage(undefined);

    try {
      setServices(await listServices());
    } catch (error) {
      setErrorMessage(
        getErrorMessage(error, "Database service inventory could not be loaded safely."),
      );
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadDatabaseServices();
  }, [loadDatabaseServices]);

  const databaseServices = useMemo(
    () =>
      services.filter(
        (service) => service.serviceType === "mysql" || service.serviceType === "postgresql",
      ),
    [services],
  );

  return (
    <PageShell
      title="Databases"
      description="Database service visibility backed by the same lifecycle adapters used by the Services control surface."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {isLoading ? <LoadingState label="Loading database service inventory" /> : null}

      <div className="grid gap-5 xl:grid-cols-2">
        <MysqlStatusPanel />
        <PostgresStatusPanel />
      </div>

      {databaseServices.length > 0 ? (
        <section className="grid gap-4 xl:grid-cols-2">
          {databaseServices.map((service) => (
            <DatabaseServiceCard key={service.id} service={service} />
          ))}
        </section>
      ) : null}

      {!isLoading && databaseServices.length === 0 ? (
        <EmptyState
          title="No database services registered"
          description="MySQL and PostgreSQL adapters are unavailable from the backend service inventory."
        />
      ) : null}
    </PageShell>
  );
}
