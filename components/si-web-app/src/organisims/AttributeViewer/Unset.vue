<template>
  <button @click="unset" v-if="hasManualValue">
    <Trash2Icon size="1x" class="ml-1" />
  </button>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { Trash2Icon } from "vue-feather-icons";

import { OpSource, EditField } from "si-entity";
import { Entity } from "@/api/sdf/model/entity";

export default Vue.extend({
  name: "Unset",
  components: {
    Trash2Icon,
  },
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
    editField: {
      type: Object as PropType<EditField>,
      required: true,
    },
    systemId: {
      type: String,
    },
  },
  computed: {
    hasManualValue(): boolean {
      if (this.entity) {
        return this.entity.hasValueFrom({
          path: this.editField.path,
          source: OpSource.Manual,
          system: this.systemId,
        });
      } else {
        return false;
      }
    },
  },
  methods: {
    unset(): void {
      this.$emit("unset");
    },
  },
});
</script>
