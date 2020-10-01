<template>
  <div
    class="flex flex-row mt-2 items-top"
    v-if="Object.keys(fieldValue).length || editorMode == 'edit'"
  >
    <div
      class="w-40 px-2 text-sm leading-tight text-right text-white input-label"
    >
      {{ entityProperty.name }}
    </div>

    <div class="w-4/5 ml-2">
      <div
        v-for="[key, value] of Object.entries(fieldValue)"
        :key="key"
        class="flex pb-2"
      >
        <div class="flex w-full row" v-if="editorMode == 'view'">
          <div
            class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
            v-bind:class="mapTextClasses(0, 'value')"
          >
            {{ key }}: {{ value }}
          </div>
        </div>
        <div class="flex flex-row items-center" v-else>
          <div
            class="w-2/5 pl-2 text-sm leading-tight text-gray-400"
            v-if="key"
          >
            {{ key }}:
          </div>
          <input
            class="w-3/5 pl-2 ml-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
            v-bind:class="mapInputClasses(0, 'value')"
            type="text"
            aria-label="val"
            v-model="fieldValue[key]"
            placeholder="value"
            @blur="saveIfModified()"
          />

          <button
            class="pl-1 text-gray-600 focus:outline-none"
            type="button"
            @click="removeFromMap(key)"
          >
            <x-icon size="0.8x"></x-icon>
          </button>
        </div>
      </div>
      <div v-if="hasNew" class="flex pb-2">
        <div class="items-center">
          <input
            class="w-2/5 pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
            v-bind:class="mapInputClasses(0, 'key')"
            type="text"
            aria-label="key"
            v-model="newKey"
            placeholder="key"
          />
          <input
            class="w-2/5 pl-2 ml-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
            v-bind:class="mapInputClasses(0, 'value')"
            type="text"
            aria-label="val"
            v-model="newValue"
            placeholder="value"
            :disabled="!newKey"
            @blur="addNew()"
          />
          <button
            class="pl-1 text-gray-600 focus:outline-none"
            type="button"
            @click="cancelNew"
          >
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
  data() {
    return {
      save: false,
      newKey: "",
      newValue: "",
      hasNew: false,
    };
  },
  computed: {
    ...mapState({
      editorMode: (state: any): RootStore["editor"]["mode"] =>
        state.editor.mode,
    }),
  },
  methods: {
    async addNew() {
      if (this.hasNew && this.newKey && this.newValue) {
        Vue.set(this.fieldValue, this.newKey, this.newValue);
        await this.saveIfModified();
      }
      this.hasNew = false;
      this.newKey = "";
      this.newValue = "";
    },
    cancelNew() {
      this.hasNew = false;
      this.newKey = "";
      this.newValue = "";
    },
    updateMap(
      index: number,
      key: string,
      value: string,
      ...event: any[]
    ): void {
      let current = this.fieldValue;
      this.fieldValue = current;
    },
    addToMap(): void {
      this.newKey = "";
      this.newValue = "";
      this.hasNew = true;
    },
    async removeFromMap(key: string): Promise<void> {
      Vue.delete(this.fieldValue, key);
      await this.saveIfModified();
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
