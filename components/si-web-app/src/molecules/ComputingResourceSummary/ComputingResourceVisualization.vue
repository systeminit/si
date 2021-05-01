<template>
  <div class="flex flex-row h-6 w-9 flex-nowrap service-indicator">
    <div class="flex items-center content-center w-6 h-full"></div>

    <div class="flex flex-col justify-between w-3 h-full status-indicator">
      <div class="state-indicator" :class="stateStyle()" />
      <div class="health-indicator" :class="healthStyle()" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import {
  ComputingResource,
  ResourceState,
  ResourceHealth,
} from "@/api/visualization/computingResourcesData";

/*
  User should be able to configure what is displayed on the service visualization
*/

export default Vue.extend({
  name: "ResourceVisualization",
  components: {},
  props: {
    data: {
      type: Object as PropType<ComputingResource>,
    },
  },
  computed: {
    state(): ResourceState {
      for (let resource of this.data.resources) {
        if (resource.state == ResourceState.Available) {
          return ResourceState.Available;
        }
      }
      return ResourceState.Unavailable;
    },
    health(): ResourceHealth {
      let healthy = 0;
      let unhealthy = 0;
      let available = 0;

      for (let resource of this.data.resources) {
        if (resource.state == ResourceState.Available) {
          available++;
        }
        if (resource.health == ResourceHealth.Healthy) {
          healthy++;
        } else if (resource.health == ResourceHealth.Unhealthy) {
          unhealthy++;
        }
      }

      if (healthy === available) {
        return ResourceHealth.Healthy;
      } else if (unhealthy === available) {
        return ResourceHealth.Unhealthy;
      } else {
        return ResourceHealth.Degraded;
      }
    },
  },
  methods: {
    stateStyle(): Record<string, any> {
      let classes: Record<string, any> = {};
      if (this.state == ResourceState.Available) {
        classes["available"] = true;
        classes["unavailable"] = false;
      } else if (this.state == ResourceState.Unavailable) {
        classes["available"] = false;
        classes["unavailable"] = true;
      }
      return classes;
    },
    healthStyle(): Record<string, any> {
      let classes: Record<string, any> = {};
      if (this.health == ResourceHealth.Healthy) {
        classes["healthy"] = true;
        classes["unhealthy"] = false;
        classes["degraded"] = false;
      } else if (this.state == ResourceState.Unavailable) {
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

.available {
  background-color: #a0e1e2;
}

.unavailable {
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
