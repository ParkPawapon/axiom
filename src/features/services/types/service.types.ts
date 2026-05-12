export type ServiceType = "php" | "mysql" | "postgresql" | "reverseProxy" | "docker";

export type ServiceStatus = "notConfigured" | "stopped" | "running" | "failed";

export type ServiceAction = "start" | "stop" | "restart";

export type ServiceActionState = "completed" | "blocked";

export interface ManagedService {
  id: string;
  name: string;
  serviceType: ServiceType;
  status: ServiceStatus;
  description: string;
  statusMessage: string;
  canStart: boolean;
  canStop: boolean;
  canRestart: boolean;
}

export interface ServiceActionOutcome {
  action: ServiceAction;
  state: ServiceActionState;
  service: ManagedService;
  message: string;
}
