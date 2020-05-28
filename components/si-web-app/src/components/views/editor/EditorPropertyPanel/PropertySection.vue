<template>
  <section :class="accordionClasses">
    
    <div
      class="section-header cursor-pointer pl-2 text-sm text-white property-section-title-bg-color"
      @click="toggleAccordion"
    >
      <div v-if="isOpen" class="flex">
        <chevron-down-icon size="1.5x"></chevron-down-icon>
        {{ sectionTitle }}
      </div>

      <div v-else-if="!isOpen" class="flex">
         <chevron-right-icon size="1.5x"></chevron-right-icon>
        {{ sectionTitle }}
      </div>

    </div>

    <div class="section-content">
      <slot/>
    </div>

  </section>
</template>

<script>
import { ChevronDownIcon, ChevronRightIcon } from "vue-feather-icons"

export default {
  name: "PropertySection",
  components: {
    ChevronDownIcon,
    ChevronRightIcon,
  },
  props: {
    sectionTitle: String,
  },
  data() {
    return {
      isOpen: true
    }
  },
  methods: {
    toggleAccordion: function() {
      this.isOpen = !this.isOpen;
    }
  },
  computed: {
    accordionClasses: function() {
      return {
        'is-closed': !this.isOpen,
        // 'is-primary': this.isOpen,
        // 'is-dark': !this.isOpen
      };
    }
  }
};
</script>

<style scoped>
.property-section-title-bg-color {
  background-color: #292C2D;
}

.section-content  {
  @apply overflow-hidden transition duration-150 ease-in-out;
}

.is-closed .section-content {
  @apply overflow-hidden h-0;
}


</style>