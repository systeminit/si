<template>
  <div class="flex flex-row" v-if="fieldValue.length || editorMode == 'edit'">
    <div class="input-label">{{ entityProperty.name }}:</div>

    <div>
      <div
        v-for="(mapEntry, index) in fieldValue"
        :key="index"
        class="flex pb-2"
      >
        <div v-if="editorMode == 'view'">
          <div class="input-label text-sm leading-tight text-white">
            {{ mapEntry.key }}: {{ mapEntry.value }}
          </div>
        </div>
        <div v-else>
          <input
            class="appearance-none text-sm leading-tight focus:outline-none input-bg-color appearance-none border-none text-gray-400 pl-2 h-5 w-32"
            type="text"
            aria-label="key"
            v-model="mapEntry.key"
            @input="
              updateMap(index, mapEntry.key, mapEntry.value, ...arguments)
            "
            placeholder="key"
          />

          <input
            class="appearance-none text-sm leading-tight focus:outline-none input-bg-color appearance-none border-none text-gray-400 pl-2 ml-2 h-5 w-32"
            type="text"
            aria-label="val"
            v-model="mapEntry.value"
            @input="
              updateMap(index, mapEntry.key, mapEntry.value, ...arguments)
            "
            placeholder="value"
          />

          <button
            class="text-gray-600 pl-1 focus:outline-none"
            type="button"
            @click="removeFromMap(index)"
          >
            <!-- 
                @click="removeItem($event, objectModel, index)"
              -->
            <x-icon size="0.8x"></x-icon>
          </button>
        </div>
      </div>
      <div class="flex text-gray-500" v-if="editorMode == 'edit'">
        <button class="focus:outline-none" type="button" @click="addToMap">
          <plus-square-icon size="1.25x"></plus-square-icon>
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { Store, mapState, mapGetters } from "vuex";
import { PlusSquareIcon, XIcon } from "vue-feather-icons";
import _ from "lodash";

import { RootStore } from "@/store";
import { EntityProperty } from "@/store/modules/entity";
import { debouncedFieldValueSet } from "@/store/modules/editor";

interface MapEntries {
  [index: number]: { key: string; value: string };
}

export default Vue.extend({
  name: "PropMap",
  props: {
    entityProperty: Object as () => EntityProperty,
  },
  components: {
    PlusSquareIcon,
    XIcon,
  },
  methods: {
    updateMap(
      index: number,
      key: string,
      value: string,
      ...event: any[]
    ): void {
      let current = this.fieldValue;
      current[index] = { key, value };
      this.fieldValue = current;
    },
    addToMap(): void {
      let current = this.fieldValue;
      current.push({ key: "", value: "" });
      this.fieldValue = current;
    },
    removeFromMap(index: number): void {
      let current = _.cloneDeep(this.fieldValue);
      current.splice(index, 1);
      this.fieldValue = current;
    },
  },
  computed: {
    ...mapState({
      editorMode: (state: any): RootStore["editor"]["mode"] =>
        state.editor.mode,
      selectedNode: (state: any): any => state.editor.selectedNode,
    }),
    fieldValue: {
      get(): MapEntries {
        let mapValues = this.$store.getters["editor/getEditValue"](
          this.entityProperty.path,
        );
        if (mapValues != undefined) {
          return mapValues;
        } else {
          return [];
        }
      },
      async set(value: any) {
        await debouncedFieldValueSet({
          store: this.$store,
          path: this.entityProperty.path,
          value: value,
          map: true,
        });
      },
    },
  },
});
</script>

<style scoped>
.property-editor-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292c2d;
}

.input-bg-color {
  background-color: #25788a;
}

.input-label {
  @apply pr-2 text-sm text-gray-400 text-right w-40;
}

input[type="number"]::-webkit-inner-spin-button,
input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
</style>
