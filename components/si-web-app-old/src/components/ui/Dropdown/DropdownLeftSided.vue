<template>
  <div class="relative w-auto">
    <button @click="isOpen = !isOpen" class="w-full focus:outline-none">
      <div class="flex flex-row items-center">
        <menu-icon size="1.0x" class="text-gray-200" />

        <div
          class="ml-1 text-gray-200 text-left font-medium truncate subpixel-antialiased tracking-tight"
        >
          {{ selected }}
        </div>
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
      class="absolute left-0 w-56 bg-gray-700 shadow-md border border-gray-600"
    >
      <div
        class="block px-4 text-gray-300 text-sm subpixel-antialiased tracking-tight hover:bg-teal-600 hover:text-white cursor-pointer"
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
import { MenuIcon } from "vue-feather-icons";

export default {
  name: "DropdownLeftSided",
  components: {
    MenuIcon,
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
