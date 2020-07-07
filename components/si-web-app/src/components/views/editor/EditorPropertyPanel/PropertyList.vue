<template>
  <div>
    <div v-if="selectedNode">
      <div class="text-red-700" v-if="selectedNode.deleted">
        Will be deleted!
      </div>
      <div
        v-for="entityProperty in propertiesList"
        :key="entityProperty.path.join('-')"
      >
        <div v-if="!entityProperty.hidden" class="flex flex-row">
          <div class="w-full">
            <div class="py-1">
              <div v-if="repeated(entityProperty)">
                <PropRepeated :entityProperty="entityProperty" />
              </div>

              <div v-else-if="propKind(entityProperty, 'object')">
                <PropObject :entityProperty="entityProperty" />
              </div>

              <div v-else-if="propKind(entityProperty, 'text')">
                <PropText :entityProperty="entityProperty" />
              </div>

              <div v-else-if="propKind(entityProperty, 'code')">
                <!-- for now, do nothing! -->
              </div>

              <div v-else-if="propKind(entityProperty, 'number')">
                <PropNumber :entityProperty="entityProperty" />
              </div>

              <div v-else-if="propKind(entityProperty, 'enum')">
                <PropEnum :entityProperty="entityProperty" />
              </div>

              <div v-else-if="propKind(entityProperty, 'bool')">
                <PropBool :entityProperty="entityProperty" />
              </div>

              <div v-else-if="propKind(entityProperty, 'map')">
                <PropMap :entityProperty="entityProperty" />
              </div>

              <div v-else class="text-red-700">
                {{ entityProperty.name }}
              </div>
            </div>
          </div>
        </div>
      </div>
      {{ selectedNode }}
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState, mapGetters } from "vuex";

import { EntityProperty } from "@/store/modules/entity";
import PropText from "./PropText.vue";
import PropObject from "./PropObject.vue";
import PropNumber from "./PropNumber.vue";
import PropEnum from "./PropEnum.vue";
import PropMap from "./PropMap.vue";
import PropRepeated from "./PropRepeated.vue";
import PropBool from "./PropBool.vue";

export default Vue.extend({
  name: "PropertyList",
  components: {
    PropText,
    PropObject,
    PropNumber,
    PropEnum,
    PropMap,
    PropRepeated,
    PropBool,
  },
  methods: {
    propKind(prop: EntityProperty, kindToCheck: string): boolean {
      return prop.kind == kindToCheck;
    },
    repeated(prop: EntityProperty): boolean {
      return prop.repeated;
    },
  },
  computed: {
    ...mapGetters({
      propertiesList: "editor/propertiesList",
    }),
    ...mapState({
      selectedNode: (state: any): any => state.editor.selectedNode,
    }),
  },
});
</script>
