<template>
  <div class="relative w-auto">
    <button @click="isOpen = !isOpen" class="w-full focus:outline-none">
      <div
        class="flex flex-row justify-between h-5 w-full items-center menu border"
      >
        <div
          class="block ml-4 text-gray-200 text-sm font-light truncate subpixel-antialiased tracking-tight"
        >
          {{ selected }}
        </div>

        <chevron-down-icon size="1.2x" class="self-center text-gray-200" />
      </div>
    </button>

    <button
      v-if="isOpen"
      @click="isOpen = false"
      tabindex="-1"
      class="fixed inset-0 h-full w-full cursor-default focus:outline-none"
    />

    <div
      v-if="isOpen"
      class="absolute left-0 -mt-05 w-full options shadow-md border"
    >
      <div
        class="block text-left px-4 text-gray-300 text-sm font-light subpixel-antialiased tracking-tight options hover:text-white cursor-pointer"
        v-for="(option, i) of options"
        :key="i"
        @click="onSelect(option)"
      >
        {{ option }}
      </div>
    </div>
  </div>
</template>

<script>
import { ChevronDownIcon } from "vue-feather-icons";

export default {
  name: "DropdownStandard",
  components: {
    ChevronDownIcon,
  },
  props: {
    default: {
      type: String,
      required: true,
    },
    options: {
      type: Array,
      required: true,
    },
  },
  data() {
    let selected = this.default;
    return {
      isOpen: false,
      selected,
    };
  },
  created() {
    const handleEscape = e => {
      if (e.key === "Esc" || e.key === "Escape") {
        this.isOpen = false;
      }
    };
    document.addEventListener("keydown", handleEscape);
    this.$once("hook:beforeDestroy", () => {
      document.removeEventListener("keydown", handleEscape);
    });
  },
  methods: {
    onSelect(option) {
      this.selected = option;
      this.$emit("selected", option);
      this.isOpen = false;
    },
  },
};
</script>

<style>
.menu {
  background-color: #2d3748;
  border-color: #485359;
}

.options {
  background-color: #1f2631;
  border-color: #485359;
}
.options:hover {
  background-color: #3d4b62;
  border-color: #454d3e;
}
</style>
