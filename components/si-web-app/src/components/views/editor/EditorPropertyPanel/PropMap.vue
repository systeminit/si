<template>
  <div
    class="flex flex-row mt-2 items-top"
    v-if="fieldValue.length || editorMode == 'edit'"
  >
    <div
      class="w-40 px-2 text-sm leading-tight text-right text-white input-label"
    >
      {{ entityProperty.name }}
    </div>

    <div class="w-4/5 ml-2">
      <div
        v-for="(mapEntry, index) in fieldValue"
        :key="index"
        class="flex pb-2"
      >
        <div class="flex w-full row" v-if="editorMode == 'view'">
          <div
            class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
            v-bind:class="mapTextClasses(index, 'value')"
          >
            {{ mapEntry.key }}: {{ mapEntry.value }}
          </div>
        </div>
        <div class="items-center" v-else>
          <input
            class="w-2/5 pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
            v-bind:class="mapInputClasses(index, 'key')"
            type="text"
            aria-label="key"
            v-model="mapEntry.key"
            @input="
              updateMap(index, mapEntry.key, mapEntry.value, ...arguments)
            "
            placeholder="key"
          />

          <input
            class="w-2/5 pl-2 ml-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
            v-bind:class="mapInputClasses(index, 'value')"
            type="text"
            aria-label="val"
            v-model="mapEntry.value"
            @input="
              updateMap(index, mapEntry.key, mapEntry.value, ...arguments)
            "
            placeholder="value"
          />

          <button
            class="pl-1 text-gray-600 focus:outline-none"
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
      <ValidationWidget :value="fieldValue" :entityProperty="entityProperty" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { Store, mapState, mapGetters } from "vuex";
import { PlusSquareIcon, XIcon } from "vue-feather-icons";
import _ from "lodash";

import { RootStore } from "@/store";
import { RegistryProperty, debouncedSetFieldValue } from "@/store/modules/node";

import PropMixin from "./PropMixin";
import ValidationWidget from "@/components/ui/ValidationWidget.vue";

interface MapEntries {
  [index: number]: { key: string; value: string };
}

export default PropMixin.extend({
  name: "PropMap",
  components: {
    PlusSquareIcon,
    XIcon,
    ValidationWidget,
  },
  methods: {
    updateMap(
      index: number,
      key: string,
      value: string,
      ...event: any[]
    ): void {
      let current = _.cloneDeep(this.fieldValue);
      current[index] = { key, value };
      this.fieldValue = current;
    },
    addToMap(): void {
      let current = _.cloneDeep(this.fieldValue);
      let index = current.length;
      current.push({ key: `key${index}`, value: `value${index}` });
      this.fieldValue = current;
    },
    removeFromMap(index: number): void {
      let current = _.cloneDeep(this.fieldValue);
      current.splice(index, 1);
      this.fieldValue = current;
    },
    mapTextClasses(index: number, part: string): Record<string, boolean> {
      let results: Record<string, boolean> = {};
      if (this.mapHasBeenEdited(index, part)) {
        results["input-border-gold"] = true;
        results["border"] = true;
      } else {
        results["input-border-grey"] = true;
      }
      return results;
    },
    mapInputClasses(index: number, part: string): Record<string, boolean> {
      let results: Record<string, boolean> = {};
      results["si-property"] = true;
      if (this.mapHasBeenEdited(index, part)) {
        results["input-border-gold"] = true;
        results["input-bg-color-grey"] = true;
      } else {
        results["input-border-grey"] = true;
        results["input-bg-color-grey"] = true;
      }
      return results;
    },
    mapHasBeenEdited(index: number, part: string): boolean {
      const path = _.cloneDeep(this.entityProperty.path);
      path.push(index);
      path.push(part);
      let result = _.find(this.diff.entries, diffEntry => {
        return _.isEqual(diffEntry.path, path);
      });
      if (result) {
        return true;
      } else {
        return false;
      }
    },
  },
  computed: {
    ...mapState({
      editorMode: (state: any): RootStore["editor"]["mode"] =>
        state.editor.mode,
    }),
    fieldValue: {
      get(): MapEntries {
        let mapValues = this.$store.getters["node/getFieldValue"](
          this.entityProperty.path,
        );
        if (mapValues != undefined) {
          return _.cloneDeep(mapValues);
        } else {
          return [];
        }
      },
      async set(value: any) {
        await debouncedSetFieldValue({
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
</style>
