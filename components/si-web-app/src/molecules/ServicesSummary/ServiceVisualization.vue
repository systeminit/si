<template>
  <div class="flex flex-row h-6 w-9 flex-nowrap service-indicator">
    <div class="flex items-center content-center w-6 h-full">
      <div class="w-full text-center capacity-indicator">{{ capacity }}%</div>
    </div>

    <div class="flex flex-col justify-between w-3 h-full status-indicator">
      <div class="state-indicator" :class="stateStyle()" />
      <div class="health-indicator" :class="healthStyle()" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import {
  Service,
  ServiceState,
  ServiceHealth,
} from "@/api/visualization/servicesData";

/*
  User should be able to configure what is displayed on the service visualization
*/

export default Vue.extend({
  name: "ServiceVisualization",
  components: {},
  props: {
    data: {
      type: Object as PropType<Service>,
    },
  },
  computed: {
    state(): ServiceState {
      for (let serviceInstance of this.data.instances) {
        if (serviceInstance.state == ServiceState.Running) {
          return ServiceState.Running;
        }
      }
      return ServiceState.Stopped;
    },
    health(): ServiceHealth {
      let healthy = 0;
      let unhealthy = 0;
      let running = 0;

      for (let serviceInstance of this.data.instances) {
        if (serviceInstance.state == ServiceState.Running) {
          running++;
        }
        if (serviceInstance.health == ServiceHealth.Healthy) {
          healthy++;
        } else if (serviceInstance.health == ServiceHealth.Unhealthy) {
          unhealthy++;
        }
      }

      if (healthy === running) {
        return ServiceHealth.Healthy;
      } else if (unhealthy === running) {
        return ServiceHealth.Unhealthy;
      } else {
        return ServiceHealth.Degraded;
      }
    },
    capacity(): number {
      const instances = this.data.instances.length;
      let running = 0;

      for (let serviceInstance of this.data.instances) {
        if (serviceInstance.state == ServiceState.Running) {
          running++;
        }
      }

      let capacityPercentage: number;

      if (running === instances) {
        capacityPercentage = 100;
        return capacityPercentage;
      } else {
        capacityPercentage = (running / instances) * 100;
        return capacityPercentage;
      }
    },
  },
  methods: {
    capacityStyle(): Record<string, any> {
      let classes: Record<string, any> = {};
      if (this.state == ServiceState.Running) {
        classes["running"] = true;
        classes["stopped"] = false;
      } else if (this.state == ServiceState.Stopped) {
        classes["running"] = false;
        classes["stopped"] = true;
      }
      return classes;
    },
    stateStyle(): Record<string, any> {
      let classes: Record<string, any> = {};
      if (this.state == ServiceState.Running) {
        classes["running"] = true;
        classes["stopped"] = false;
      } else if (this.state == ServiceState.Stopped) {
        classes["running"] = false;
        classes["stopped"] = true;
      }
      return classes;
    },
    healthStyle(): Record<string, any> {
      let classes: Record<string, any> = {};
      if (this.health == ServiceHealth.Healthy) {
        classes["healthy"] = true;
        classes["unhealthy"] = false;
        classes["degraded"] = false;
      } else if (this.state == ServiceState.Stopped) {
        classes["healthy"] = false;
        classes["unhealthy"] = true;
        classes["degraded"] = false;
      } else {
        classes["healthy"] = false;
        classes["unhealthy"] = false;
        classes["degraded"] = true;
      }
      return classes;
    },
  },
});
</script>

<style scoped>
.service-indicator {
  background-color: #444550;
}

.capacity-indicator {
  font-size: 8px;
  font-weight: 600;
  color: #ced0e2;
}

.status-indicator {
  padding-left: 1px;
  padding-right: 1px;
  padding-top: 2px;
  padding-bottom: 2px;
}

.state-indicator {
  background-color: #a0e1e2;
  height: 40%;
}

.health-indicator {
  height: 40%;
}

.running {
  background-color: #a0e1e2;
}

.stopped {
  background-color: #aab0b1;
}

.healthy {
  background-color: #a6e2a5;
}

.unhealthy {
  background-color: #e2a5a5;
}

.degraded {
  background-color: #e2c8a5;
}
</style>
