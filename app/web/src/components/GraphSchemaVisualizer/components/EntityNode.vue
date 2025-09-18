<template>
  <g
    :class="nodeClasses"
    :transform="`translate(${node.position.x}, ${node.position.y})`"
    @click="handleClick"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
    @contextmenu="handleContextMenu"
  >
    <!-- Main node rectangle -->
    <rect
      :width="node.dimensions.width"
      :height="node.dimensions.height"
      :class="nodeRectClasses"
      rx="6"
      ry="6"
    />
    
    <!-- Header section -->
    <g class="node-header">
      <!-- Header background -->
      <rect
        :width="node.dimensions.width"
        height="32"
        :class="headerClasses"
        rx="6"
        ry="6"
      />
      <!-- Bottom corners need to be squared off -->
      <rect
        :width="node.dimensions.width"
        height="6"
        y="26"
        :class="headerClasses"
      />
      
      <!-- Icon -->
      <g v-if="entityIcon" :transform="`translate(8, 8)`">
        <foreignObject width="16" height="16">
          <Icon :name="entityIcon" size="sm" />
        </foreignObject>
      </g>
      
      <!-- Title -->
      <text
        :x="entityIcon ? 32 : 12"
        y="21"
        :class="headerTextClasses"
        font-size="12"
        font-weight="600"
      >
        {{ truncatedTitle }}
      </text>
      
      <!-- Table link button -->
      <g class="table-link" :transform="`translate(${node.dimensions.width - 24}, 8)`">
        <rect
          width="16"
          height="16"
          :class="linkButtonClasses"
          rx="2"
          @click.stop="handleTableLinkClick"
        />
        <foreignObject width="16" height="16">
          <Icon name="data-table" size="xs" />
        </foreignObject>
      </g>
    </g>
    
    <!-- Body section -->
    <g class="node-body" :transform="`translate(0, 32)`">
      <!-- All properties list -->
      <g v-if="allProperties.length > 0" :transform="`translate(0, 0)`">
        <g
          v-for="(prop, index) in allProperties"
          :key="prop.name"
          :transform="`translate(0, ${index * 18})`"
        >
          <!-- Property row background -->
          <rect
            :width="node.dimensions.width"
            height="18"
            :class="getPropertyRowClasses(prop, index)"
          />
          
          <!-- Property icon -->
          <g :transform="`translate(8, 4)`">
            <rect
              width="10"
              height="10"
              :class="getPropertyIconClasses(prop)"
              rx="1"
            />
          </g>
          
          <!-- Property name -->
          <text
            x="24"
            y="13"
            :class="getPropertyNameClasses(prop)"
            font-size="11"
            font-weight="500"
          >
            {{ prop.name }}
          </text>
          
          <!-- Property type -->
          <text
            :x="node.dimensions.width - 8"
            y="13"
            text-anchor="end"
            :class="getPropertyTypeClasses()"
            font-size="10"
          >
            {{ prop.type }}
          </text>
        </g>
      </g>
    </g>
    
    
    <!-- Selection overlay -->
    <rect
      v-if="isSelected"
      :width="node.dimensions.width"
      :height="node.dimensions.height"
      class="selection-overlay"
      rx="6"
      ry="6"
      fill="none"
      stroke="rgb(59 130 246)"
      stroke-width="2"
      stroke-dasharray="4 2"
    />
  </g>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { Icon } from "@si/vue-lib/design-system";
import type { EntityNode } from "../types/schema-graph";

interface Props {
  node: EntityNode;
  isSelected?: boolean;
  isHovered?: boolean;
}

interface Emits {
  (e: "click", node: EntityNode, event: MouseEvent): void;
  (e: "contextmenu", node: EntityNode, event: MouseEvent): void;
  (e: "mouseenter", node: EntityNode): void;
  (e: "mouseleave", node: EntityNode): void;
  (e: "table-link-click", node: EntityNode): void;
}

const props = withDefaults(defineProps<Props>(), {
  isSelected: false,
  isHovered: false
});

const emit = defineEmits<Emits>();

// Entity type checks
const isSchemaVariant = computed(() => props.node.entityKind === 'SchemaVariant');
const isComponent = computed(() => props.node.entityKind === 'Component');
const isFunction = computed(() => props.node.entityKind === 'Function');

// Display properties
const entityIcon = computed(() => {
  switch (props.node.entityKind) {
    case 'SchemaVariant': return 'schematics';
    case 'Component': return 'grid';
    case 'Schema': return 'code-deployed';
    case 'Function': return 'func';
    case 'Action': return 'code-deployed';
    default: return 'schematics';
  }
});

const truncatedTitle = computed(() => {
  const title = (props.node.entityKind as string);
  return title.length > 18 ? title.substring(0, 15) + '...' : title;
});

// All properties to show in body
const allProperties = computed(() => {
  // Check if properties exists and is an array
  if (props.node.properties && Array.isArray(props.node.properties)) {
    return props.node.properties;
  }
  return [];
});

// Get property row background classes
const getPropertyRowClasses = (prop: any, index: number) => {
  const baseClasses = ['transition-all duration-200'];
  
  // Alternating row colors for better readability
  if (index % 2 === 0) {
    return baseClasses.concat(
      isDarkTheme.value ? ['fill-neutral-750'] : ['fill-neutral-50']
    );
  } else {
    return baseClasses.concat(
      isDarkTheme.value ? ['fill-neutral-800'] : ['fill-white']
    );
  }
};

// Get property icon classes (colored squares like in the target design)
const getPropertyIconClasses = (prop: any) => {
  const baseClasses = ['transition-all duration-200'];
  
  if (prop.kind === 'primary_key') {
    return baseClasses.concat(['fill-yellow-500']); // Golden diamond for primary keys
  } else if (prop.kind === 'foreign_key') {
    return baseClasses.concat(['fill-blue-500']); // Blue diamond for foreign keys
  } else {
    return baseClasses.concat(
      isDarkTheme.value ? ['fill-neutral-500'] : ['fill-neutral-400']
    ); // Gray diamond for regular properties
  }
};

// Get property name text classes
const getPropertyNameClasses = (prop: any) => {
  const baseClasses = isDarkTheme.value ? ['fill-white'] : ['fill-neutral-900'];
  
  if (prop.kind === 'primary_key') {
    return baseClasses.concat(['font-semibold']);
  }
  
  return baseClasses;
};

// Get property type text classes  
const getPropertyTypeClasses = () => {
  return isDarkTheme.value ? ['fill-neutral-400'] : ['fill-neutral-500'];
};

// Theme detection
const isDarkTheme = computed(() => {
  return document.documentElement.classList.contains('dark');
});

// Styling classes
const nodeClasses = computed(() => [
  'entity-node',
  'cursor-pointer',
  'transition-all duration-200',
  props.isHovered && 'opacity-90'
]);

const nodeRectClasses = computed(() => [
  'transition-all duration-200',
  isDarkTheme.value ? [
    'fill-neutral-800',
    'stroke-neutral-600',
    props.isSelected ? 'stroke-action-400' : 'stroke-neutral-600',
    props.isHovered ? 'stroke-neutral-500' : 'stroke-neutral-600'
  ] : [
    'fill-white',
    'stroke-neutral-300',
    props.isSelected ? 'stroke-action-500' : 'stroke-neutral-300',
    props.isHovered ? 'stroke-neutral-400' : 'stroke-neutral-300'
  ],
  'stroke-1'
]);

const headerClasses = computed(() => {
  const baseClasses = ['transition-all duration-200'];
  
  if (isSchemaVariant.value) {
    return baseClasses.concat(
      isDarkTheme.value ? ['fill-blue-900'] : ['fill-blue-100']
    );
  } else if (isComponent.value) {
    return baseClasses.concat(
      isDarkTheme.value ? ['fill-green-900'] : ['fill-green-100']
    );
  } else if (isFunction.value) {
    return baseClasses.concat(
      isDarkTheme.value ? ['fill-purple-900'] : ['fill-purple-100']
    );
  }
  
  return baseClasses.concat(
    isDarkTheme.value ? ['fill-neutral-700'] : ['fill-neutral-100']
  );
});

const headerTextClasses = computed(() => 
  isDarkTheme.value ? ['fill-white'] : ['fill-black']
);

const badgeTextClasses = computed(() => 
  isDarkTheme.value ? ['fill-neutral-300'] : ['fill-neutral-600']
);

const footerStatusClasses = computed(() => {
  return isDarkTheme.value ? ['fill-green-400'] : ['fill-green-500'];
});

const linkButtonClasses = computed(() => [
  'cursor-pointer',
  'transition-all duration-200',
  isDarkTheme.value ? [
    'fill-neutral-700',
    'stroke-neutral-500',
    'hover:fill-neutral-600'
  ] : [
    'fill-neutral-200',
    'stroke-neutral-400',
    'hover:fill-neutral-300'
  ]
]);

// Event handlers
const handleClick = (event: MouseEvent) => {
  emit("click", props.node, event);
};

const handleContextMenu = (event: MouseEvent) => {
  event.preventDefault();
  emit("contextmenu", props.node, event);
};

const handleMouseEnter = () => {
  emit("mouseenter", props.node);
};

const handleMouseLeave = () => {
  emit("mouseleave", props.node);
};

const handleTableLinkClick = () => {
  console.log("Internal link click");
  emit("table-link-click", props.node);
};
</script>

<style scoped>
.entity-node:hover .table-link {
  opacity: 1;
}

.table-link {
  opacity: 0.6;
  transition: opacity 0.2s;
}

.selection-overlay {
  pointer-events: none;
  animation: selection-pulse 2s infinite;
}

@keyframes selection-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}
</style>