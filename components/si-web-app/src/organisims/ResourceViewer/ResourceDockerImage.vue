<template>
  <div class="flex flex-col w-full text-sm" v-if="data">
    <ResourceField label="Id" :value="data['Id']" />
    <ResourceFieldArray label="RepoTags" :value="repoTags" noLabels />
    <ResourceFieldArray label="ExposedPorts" :value="exposedPorts" noLabels />
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import { Resource } from "si-entity";
import _ from "lodash";

import ResourceField from "./ResourceField.vue";
import ResourceFieldArray, {
  ResourceFieldArrayValue,
} from "./ResourceFieldArray.vue";

export default Vue.extend({
  name: "ResourceDockerImage",
  props: {
    resource: {
      type: Object as PropType<Resource>,
    },
  },
  components: {
    ResourceField,
    ResourceFieldArray,
  },
  computed: {
    data(): Record<string, any> | null {
      // @ts-ignore
      if (this.resource.data.inspect && this.resource.data.inspect.length > 0) {
        // @ts-ignore
        return this.resource.data.inspect[0];
      } else {
        return null;
      }
    },
    repoTags(): ResourceFieldArrayValue {
      let results: ResourceFieldArrayValue = [];
      if (this.data && this.data["RepoTags"]) {
        results = _.map(this.data["RepoTags"], t => {
          return { value: t, label: t };
        });
      }
      return results;
    },
    exposedPorts(): ResourceFieldArrayValue {
      if (
        this.data &&
        this.data["Config"] &&
        this.data["Config"]["ExposedPorts"]
      ) {
        let results = [];
        for (const port of Object.keys(this.data["Config"]["ExposedPorts"])) {
          results.push({ label: port, value: port });
        }
        return results;
      } else {
        return [];
      }
    },
  },
});
</script>
