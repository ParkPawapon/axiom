import { useCallback, useEffect, useMemo, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { Button } from "../../../shared/components/ui/button";
import { Input } from "../../../shared/components/ui/input";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import {
  generateLocalCertificate,
  getSecurityStatus,
  inspectCertificateTrust,
  pruneAuditLog,
  readAuditLog,
  trustCertificateAuthority,
  updateHostsFile,
} from "../api/security.commands";
import { AuditLogPanel } from "../components/audit-log-panel";
import { ElevationPanel } from "../components/elevation-panel";
import { SecurityStatusGrid } from "../components/security-status-grid";
import type {
  AuditLogReadResult,
  CertificateTrustResult,
  HostFileUpdateResult,
  LocalCertificate,
  PermissionElevationRequest,
  SecurityPermissionStatus,
} from "../types/security.types";

type SecurityAction = "certificate" | "hosts" | "refresh" | "retention" | "trust";

export function SecurityPage() {
  const [activeAction, setActiveAction] = useState<SecurityAction>();
  const [address, setAddress] = useState("127.0.0.1");
  const [auditLog, setAuditLog] = useState<AuditLogReadResult>();
  const [certificate, setCertificate] = useState<LocalCertificate>();
  const [domain, setDomain] = useState("project.test");
  const [errorMessage, setErrorMessage] = useState<string>();
  const [hostResult, setHostResult] = useState<HostFileUpdateResult>();
  const [noticeMessage, setNoticeMessage] = useState<string>();
  const [retentionDays, setRetentionDays] = useState(30);
  const [securityStatus, setSecurityStatus] = useState<SecurityPermissionStatus>();
  const [trustResult, setTrustResult] = useState<CertificateTrustResult>();

  const elevationRequests = useMemo(
    () =>
      [hostResult?.elevation, trustResult?.elevation].filter(
        (request): request is PermissionElevationRequest => Boolean(request),
      ),
    [hostResult, trustResult],
  );

  const loadSecurityInventory = useCallback(async () => {
    setActiveAction("refresh");
    setErrorMessage(undefined);

    try {
      const [status, trust, audit] = await Promise.all([
        getSecurityStatus(),
        inspectCertificateTrust(),
        readAuditLog(200),
      ]);
      setSecurityStatus(status);
      setTrustResult(trust);
      setAuditLog(audit);
    } catch (error) {
      setErrorMessage(getErrorMessage(error, "Security inventory could not be loaded safely."));
    } finally {
      setActiveAction(undefined);
    }
  }, []);

  useEffect(() => {
    void loadSecurityInventory();
  }, [loadSecurityInventory]);

  const runSecurityAction = useCallback(
    async (action: SecurityAction, handler: () => Promise<string>) => {
      setActiveAction(action);
      setErrorMessage(undefined);
      setNoticeMessage(undefined);

      try {
        const message = await handler();
        setNoticeMessage(message);
        const [status, trust, audit] = await Promise.all([
          getSecurityStatus(),
          inspectCertificateTrust(),
          readAuditLog(200),
        ]);
        setSecurityStatus(status);
        setTrustResult(trust);
        setAuditLog(audit);
      } catch (error) {
        setErrorMessage(getErrorMessage(error, "Security action failed safely."));
      } finally {
        setActiveAction(undefined);
      }
    },
    [],
  );

  const updateHosts = () => {
    void runSecurityAction("hosts", async () => {
      const result = await updateHostsFile(domain, address);
      setHostResult(result);
      return result.statusMessage;
    });
  };

  const generateCertificate = () => {
    void runSecurityAction("certificate", async () => {
      const result = await generateLocalCertificate(domain);
      setCertificate(result);
      return result.statusMessage;
    });
  };

  const trustCertificate = () => {
    void runSecurityAction("trust", async () => {
      const result = await trustCertificateAuthority();
      setTrustResult(result);
      return result.statusMessage;
    });
  };

  const applyRetention = () => {
    void runSecurityAction("retention", async () => {
      const result = await pruneAuditLog(retentionDays);
      return result.statusMessage;
    });
  };

  const isBusy = Boolean(activeAction);

  return (
    <PageShell
      title="Security"
      description="Local development security controls for hosts mapping, certificates, explicit trust, permission elevation, and audit retention."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {noticeMessage ? (
        <div className="border-2 border-voicebox-black bg-white p-4 text-sm font-semibold text-voicebox-black">
          {noticeMessage}
        </div>
      ) : null}
      {activeAction === "refresh" ? <LoadingState label="Loading security status" /> : null}

      <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_28rem]">
        <div className="grid gap-5">
          <SecurityStatusGrid status={securityStatus} />

          <section className="grid gap-4 border-2 border-voicebox-black bg-white p-4">
            <div>
              <p className="font-mono text-xs uppercase text-voicebox-secondary">Local domain</p>
              <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
                Hosts and TLS
              </h2>
            </div>

            <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_12rem]">
              <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
                Domain
                <Input value={domain} onChange={(event) => setDomain(event.target.value)} />
              </label>
              <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
                Address
                <Input value={address} onChange={(event) => setAddress(event.target.value)} />
              </label>
            </div>

            <div className="flex flex-wrap gap-2">
              <Button disabled={isBusy} onClick={updateHosts}>
                Update hosts
              </Button>
              <Button disabled={isBusy} onClick={generateCertificate} variant="secondary">
                Generate certificate
              </Button>
              <Button disabled={isBusy} onClick={trustCertificate} variant="secondary">
                Trust local CA
              </Button>
            </div>

            {hostResult ? (
              <ResultBlock
                title="Hosts result"
                lines={[
                  hostResult.statusMessage,
                  `Hosts: ${hostResult.hostsFilePath}`,
                  hostResult.backupPath ? `Backup: ${hostResult.backupPath}` : undefined,
                  hostResult.preparedHostsPath
                    ? `Prepared: ${hostResult.preparedHostsPath}`
                    : undefined,
                ]}
              />
            ) : null}

            {certificate ? (
              <ResultBlock
                title="Certificate result"
                lines={[
                  certificate.statusMessage,
                  `Certificate: ${certificate.certificatePath}`,
                  `Private key: ${certificate.privateKeyPath}`,
                  `CA: ${certificate.certificateAuthorityPath}`,
                ]}
              />
            ) : null}
          </section>

          <AuditLogPanel
            auditLog={auditLog}
            isBusy={isBusy}
            retentionDays={retentionDays}
            onPrune={applyRetention}
            onRefresh={() => void loadSecurityInventory()}
            onRetentionDaysChange={setRetentionDays}
          />
        </div>

        <aside className="grid content-start gap-5">
          <section className="grid gap-3 border-2 border-voicebox-black bg-white p-4">
            <div>
              <p className="font-mono text-xs uppercase text-voicebox-secondary">Certificate trust</p>
              <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
                Trust Store
              </h2>
            </div>
            {trustResult ? (
              <ResultBlock
                title={trustResult.status}
                lines={[trustResult.statusMessage, trustResult.certificateAuthorityPath]}
              />
            ) : (
              <p className="text-sm text-voicebox-secondary">
                Trust status will appear after the backend inspects the local certificate authority.
              </p>
            )}
          </section>

          {elevationRequests.map((request) => (
            <ElevationPanel key={`${request.kind}-${request.title}`} request={request} />
          ))}
        </aside>
      </div>
    </PageShell>
  );
}

interface ResultBlockProps {
  title: string;
  lines: Array<string | undefined>;
}

function ResultBlock({ lines, title }: ResultBlockProps) {
  return (
    <section className="grid gap-2 border border-voicebox-border bg-voicebox-surface p-3">
      <p className="font-mono text-xs uppercase text-voicebox-secondary">{title}</p>
      {lines.filter(Boolean).map((line) => (
        <p className="break-all font-mono text-xs leading-relaxed text-voicebox-black" key={line}>
          {line}
        </p>
      ))}
    </section>
  );
}
