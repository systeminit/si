<!-- <template>
  <div class="custom-select" :tabindex="tabindex" @blur="open = false">
    <div class="selected" :class="{open: open}" @click="open = !open">{{ selected }}</div>
    <div class="items" :class="{selectHide: !open}">
      <div
        class="item"
        v-for="(option, i) of options"
        :key="i"
        @click="selected=option; open=false; $emit('input', option)"
      >{{ option }}</div>
    </div>
  </div>
</template>

    <div class="flex-1 px-2 py-2 ml-2">
      <menu-icon size="1.5x" class="text-white"></menu-icon>
    </div>
          <a href="#" class="block px-4 py-2 text-gray-800 hover:bg-indigo-500 hover:text-white">Account settings</a>
      <a href="#" class="block px-4 py-2 text-gray-800 hover:bg-indigo-500 hover:text-white">Support</a>
      <a href="#" class="block px-4 py-2 text-gray-800 hover:bg-indigo-500 hover:text-white">Sign out</a>
 

text-l text-white font-medium subpixel-antialiased tracking-tight relative w-56
 -->

<template>
  <div class="flex flex-col subpixel-antialiased tracking-tight">

    <div class="flex flex-row">
      <dir class="w-40 text-white text-left text-l font-medium truncate">{{selected}}</dir>
      
      <button @click="isOpen = !isOpen" class="ml-4 focus:outline-none focus:border-white">
         <menu-icon size="1.5x" class="text-white"></menu-icon>
      </button>
      <button v-if="isOpen" @click="isOpen = false" class="cursor-default"></button>
    </div>
    
    
    <div v-if="isOpen" class="w-48 bg-transparent text-white">

      <div
        class="w-48 text-left bg-gray-600 text-sm hover:bg-indigo-500 hover:text-white cursor-pointer"
        v-for="(option, i) of options"
        :key="i"
        @click="onSelect(option)"
      >{{ option }}</div>
    </div>
  </div>
</template>

<script>
import { MenuIcon } from "vue-feather-icons";

export default {
  name: "Dropdown",
  components: {
    MenuIcon
  },
  props: {
    default: {
      type: String,
      required: true
    },
    options: {
      type: Array,
      required: true
    }
  },
  data() {

    let selected = this.default
    return {
      isOpen: false,
      selected
    }
  },
  created() {
    const handleEscape = (e) => {
      if (e.key === 'Esc' || e.key === 'Escape') {
        this.isOpen = false
      }
    }
    document.addEventListener('keydown', handleEscape)
    this.$once('hook:beforeDestroy', () => {
      document.removeEventListener('keydown', handleEscape)
    })
  },
  methods: {
    onSelect(option) {
      this.selected = option
      this.$emit('selected', option)
      this.isOpen = false
    }
  }
}
</script>





<!-- <script>
export default {
  props: {
    options: {
      type: Array,
      required: true
    }
  },
  data() {
    return {
      selected: this.options.length > 0 ? this.options[0] : null,
      open: false
    };
  },
  mounted() {
    this.$emit("input", this.selected);
  }
};
</script> -->