<template>
  <div class="relative z-10 overflow-visible">
    <div class="flex items-center overflow-visible">
      <button
        @click="toggleMenu"
        class="w-full border border-gray-800 border-solid focus:outline-none dropdown-button"
      >
        <div class="flex items-center justify-start">
          <div
            v-bind:class="textClasses"
            class="w-10/12 pl-2 pr-2 text-left dropdown-button"
          >
            {{ buttonLabel }}
          </div>
          <div class="flex justify-end w-full pr-1">
            <slot name="icon">
              <ChevronDownIcon :size="iconSize" />
            </slot>
          </div>
        </div>
      </button>
    </div>
    <div
      v-if="show"
      class="absolute w-12 py-2 mb-10 overflow-x-hidden overflow-y-auto border border-solid shadow-xl options"
      v-bind:class="menuClasses"
    >
      <a
        href="#"
        v-bind:class="textClasses"
        class="block px-4 py-1 option-link"
        v-for="option in options"
        @click="selectOption(option)"
        :key="option.value"
      >
        {{ option.label }}
      </a>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { ChevronDownIcon } from "vue-feather-icons";
import _ from "lodash";

interface Data {
  show: boolean;
  selectedOption: null | DropdownPropsOption;
}

export interface DropdownPropsOption {
  value: string | null;
  label: string;
}

export interface DropdownProps {
  size: "xs" | "sm" | "base" | "lg";
  menuWidth: "10" | "20" | "24" | "32" | "48" | "56" | "full";
  options: DropdownPropsOption[];
  value: String | Object | Number;
  updateLabel: boolean;
}

export default Vue.extend({
  name: "Dropdown2",
  components: {
    ChevronDownIcon,
  },
  props: {
    size: {
      type: String as () => DropdownProps["size"],
      default: "base",
    },
    menuWidth: {
      type: String as () => DropdownProps["menuWidth"],
      default: "32",
    },
    options: {
      type: Array as () => DropdownProps["options"],
    },
    value: {
      type: [String, Object, Number],
    },
    updateLabel: {
      type: Boolean,
      default: true,
    },
    label: {
      type: String,
    },
  },
  data(): Data {
    return {
      show: false,
      selectedOption: null,
    };
  },
  computed: {
    buttonLabel(): string {
      if (this.updateLabel) {
        if (this.selectedOption) {
          if (_.find(this.options, ["label", this.selectedOption.label])) {
            return this.selectedOption.label;
          } else {
            return this.label;
          }
        } else {
          return this.label;
        }
      } else {
        return this.label;
      }
    },
    textClasses(): Record<string, boolean> {
      const result: Record<string, boolean> = {};
      const textSize = `text-${this.size}`;
      result[textSize] = true;
      return result;
    },
    menuClasses(): Record<string, boolean> {
      const result: Record<string, boolean> = {};
      const width = `w-${this.menuWidth}`;
      result[width] = true;
      return result;
    },
    iconSize(): string {
      switch (this.size) {
        case "xs":
          return "0.8x";
        case "sm":
          return "1.0x";
        case "base":
          return "1.5x";
        case "lg":
          return "1.8x";
        default:
          return "1.0x";
      }
    },
  },
  methods: {
    selectOption(option: DropdownPropsOption) {
      this.selectedOption = option;
      this.$emit("input", option.value);
      this.show = false;
    },
    toggleMenu(): void {
      this.show = !this.show;
    },
  },
  created() {
    const handleEscape = (e: any) => {
      if (e.key === "Esc" || e.key === "Escape") {
        this.show = false;
      }
    };
    document.addEventListener("keydown", handleEscape);
    this.$once("hook:beforeDestroy", () => {
      document.removeEventListener("keydown", handleEscape);
    });
  },
});
</script>

<style scoped>
.menu {
  background-color: #2d3748;
  border-color: #485359;
}

.dropdown-button {
  background-color: #2d3748;
  border-color: #485359;
}

.options {
  background-color: #1f2631;
  border-color: #485359;
}
.option-link:hover {
  background-color: #3d4b62;
  border-color: #454d3e;
}
</style>
