/*
  Services Summary

  Summarize service insights
   - current state (with access to state history)
   - current health (with access to health history)
   - current utilization (with access to utilization history)
   - current capacity (with access to capacity history)
   - this month projected cost (with access to daily, weekly, and monthly cost)

  State
   - if at least one instance is running, the state is running.

  Health
   - if all running instances are unhealthy the health is unhealthy.
   - if all running instances are healthy the health is healthy.
   - if some running instances are unhealthy the health is degraded. (and by how much %?)

  Capacity (%)
   - if all instances are running the capacity is 100%  
  
  Utilization )%)
   - avg utilization (summary of all running instances)
   - utilization per instance
*/

export interface Service {
  name: string;
  lastDeployed: Date;
  utilization: ServiceUtilization;
  instances: ServiceInstance[];
}

interface ServiceInstance {
  id: string;
  state: ServiceState;
  health: ServiceHealth;
  utilization: ServiceUtilization;
}

export enum ServiceState {
  Running = "running",
  Stopped = "stopped",
}

export enum ServiceHealth {
  Healthy = "healthy",
  Unhealthy = "unhealthy",
  Degraded = "degraded",
}

interface ServiceUtilization {
  cpu: number;
  memory: number;
}

const accountService: Service = {
  name: "account",
  lastDeployed: new Date("2021-04-29"),
  utilization: {
    cpu: 85,
    memory: 68,
  },
  instances: [
    {
      id: "1",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 77,
        memory: 68,
      },
    },
    {
      id: "2",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 77,
        memory: 68,
      },
    },
    {
      id: "3",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 77,
        memory: 68,
      },
    },
    {
      id: "4",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 77,
        memory: 68,
      },
    },
    {
      id: "5",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 77,
        memory: 68,
      },
    },
    {
      id: "6",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 77,
        memory: 68,
      },
    },
  ],
};

const dataService: Service = {
  name: "data",
  lastDeployed: new Date("2021-04-27"),
  utilization: {
    cpu: 85,
    memory: 68,
  },
  instances: [
    {
      id: "1",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 85,
        memory: 68,
      },
    },
    {
      id: "2",
      state: ServiceState.Running,
      health: ServiceHealth.Unhealthy,
      utilization: {
        cpu: 85,
        memory: 68,
      },
    },
    {
      id: "3",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 85,
        memory: 68,
      },
    },
    {
      id: "4",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 85,
        memory: 68,
      },
    },
    {
      id: "5",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 85,
        memory: 68,
      },
    },
    {
      id: "6",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 85,
        memory: 68,
      },
    },
  ],
};

const userService: Service = {
  name: "user",
  lastDeployed: new Date("2021-04-25"),
  utilization: {
    cpu: 85,
    memory: 68,
  },
  instances: [
    {
      id: "1",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 85,
        memory: 68,
      },
    },
    {
      id: "2",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 85,
        memory: 68,
      },
    },
    {
      id: "3",
      state: ServiceState.Stopped,
      health: ServiceHealth.Unhealthy,
      utilization: {
        cpu: 85,
        memory: 68,
      },
    },
    {
      id: "4",
      state: ServiceState.Stopped,
      health: ServiceHealth.Unhealthy,
      utilization: {
        cpu: 85,
        memory: 68,
      },
    },
    {
      id: "5",
      state: ServiceState.Stopped,
      health: ServiceHealth.Unhealthy,
      utilization: {
        cpu: 85,
        memory: 68,
      },
    },
    {
      id: "6",
      state: ServiceState.Running,
      health: ServiceHealth.Healthy,
      utilization: {
        cpu: 85,
        memory: 68,
      },
    },
  ],
};

export const servicesData: Service[] = [
  accountService,
  dataService,
  userService,
];
