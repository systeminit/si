/*
  Computing Resources Summary
  
  Represents the actual resources of computing components.

  Summarize computing resources insights (k8s cluster example)
   - current state (with access to state history)
   - current health (with access to health history)
   - current capacity and utilization (with access to capacity history)
   - this month projected cost (with access to daily, weekly, and monthly cost)

  State
   - if at least one node of a cluster is running, the state is available.

  Health
   - if all available nodes are unhealthy the health is unhealthy.
   - if all available nodes are healthy the health is healthy.
   - if some available nodes are unhealthy the health is degraded. (and by how much %?)

  Capacity and utilization  (%)
   - percentage cpu utilization for the sume of available nodes
   - percentage memory utilization for the sume of available nodes
   - percentage of nodes available vs allocated (that aren't available)
*/

export interface ComputingResource {
  name: string;
  lastDeployed: Date;
  resources: Resource[];
  utilization: ResourceUtilization;
}

interface Resource {
  id: string;
  type: ResourceType;
  state: ResourceState;
  health: ResourceHealth;
  utilization: ResourceUtilization;
}

interface ResourceUtilization {
  cpu: number;
  memory: number;
}

export enum ResourceType {
  AwsEc2Instance = "aws-ec2-instance",
}

export enum ResourceState {
  Available = "available",
  Unavailable = "unavailable",
}

export enum ResourceHealth {
  Healthy = "healthy",
  Unhealthy = "unhealthy",
  Degraded = "degraded",
  Unavailable = "unavailable",
}

const k8sCluster: ComputingResource = {
  name: "kubernetes cluster",
  lastDeployed: new Date("2021-04-17"),
  utilization: {
    cpu: 77,
    memory: 68,
  },
  resources: [
    {
      id: "1",
      type: ResourceType.AwsEc2Instance,
      state: ResourceState.Available,
      health: ResourceHealth.Healthy,
      utilization: {
        cpu: 75,
        memory: 70,
      },
    },
    {
      id: "2",
      type: ResourceType.AwsEc2Instance,
      state: ResourceState.Unavailable,
      health: ResourceHealth.Unavailable,
      utilization: {
        cpu: 0,
        memory: 0,
      },
    },
    {
      id: "3",
      type: ResourceType.AwsEc2Instance,
      state: ResourceState.Available,
      health: ResourceHealth.Healthy,
      utilization: {
        cpu: 60,
        memory: 78,
      },
    },
    {
      id: "4",
      type: ResourceType.AwsEc2Instance,
      state: ResourceState.Available,
      health: ResourceHealth.Healthy,
      utilization: {
        cpu: 72,
        memory: 69,
      },
    },
    {
      id: "5",
      type: ResourceType.AwsEc2Instance,
      state: ResourceState.Available,
      health: ResourceHealth.Healthy,
      utilization: {
        cpu: 81,
        memory: 67,
      },
    },
    {
      id: "6",
      type: ResourceType.AwsEc2Instance,
      state: ResourceState.Available,
      health: ResourceHealth.Healthy,
      utilization: {
        cpu: 75,
        memory: 70,
      },
    },
  ],
};

const database: ComputingResource = {
  name: "account database",
  lastDeployed: new Date("2021-04-57"),
  utilization: {
    cpu: 77,
    memory: 68,
  },
  resources: [
    {
      id: "1",
      type: ResourceType.AwsEc2Instance,
      state: ResourceState.Available,
      health: ResourceHealth.Healthy,
      utilization: {
        cpu: 75,
        memory: 70,
      },
    },
    {
      id: "2",
      type: ResourceType.AwsEc2Instance,
      state: ResourceState.Available,
      health: ResourceHealth.Healthy,
      utilization: {
        cpu: 75,
        memory: 70,
      },
    },
  ],
};

export const computingResourceData: ComputingResource[] = [
  k8sCluster,
  database,
];
