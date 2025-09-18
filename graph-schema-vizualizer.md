# GraphSchemaVisualizer Implementation Plan

## Overview

This document outlines the implementation plan for a Vue Component called `GraphSchemaVisualizer` that provides a database schema visualization interface similar to modern database tools. The component will integrate with existing System Initiative types and follow established architectural patterns.

## Requirements Analysis

### Phase 1 - Initial implementation

### 1.1 Screenshot Analysis
The target interface shows:
- **Main Area**: Graph visualization of database tables as nodes with relationships between foreign keys
- **Table Nodes**: Rectangular boxes showing entity names with column listing 
- **Relationships**: Lines connecting entities showing foreign key relationships
- **Layout**: Hierarchical arrangement with clear visual grouping
- **UI Elements**: Clean database tool interface with collapsible panels and connection indicators

### 1.2 Existing Type Integration
The component will leverage these existing types:
- **ComponentInList**: Lightweight component representation for list views
- **ComponentList**: Workspace-level collection of components
- **ActionView**: Actions that can be performed on components
- **SchemaVariant**: Different versions/variants of schemas
- **View**: View configurations
- **FuncRunView**: Function execution tracking
- **AuditLog**: Audit trail for system activities

## 1.3 Component Architecture

### 1.3.1 Core Component Structure

**Main Component: `GraphSchemaVisualizer.vue`**

```typescript
interface Props {
  // Data sources
  // Data Source will be dynamically generated based on a pre-seeded 
  // schema that defines the relationships between schemas/variants/components/etc. which will be used to populate the tables/columns for those entities. 
  
  
  // Visualization options
  selectedEntity?: string
  showRelationships: boolean
  layoutAlgorithm: 'hierarchical' | 'layered' | 'force' // just reuse the layout algorithm used for map.vue 
}

interface EntityNode {
  id: string
  entityKind: EntityKind
  entityData: string
  position: { x: number, y: number }
  dimensions: { width: number, height: number }
}

```

### 1.3.2 Data Integration Strategy

**Type Mapping & Data Flow:**
```typescript
// Transform existing types for graph visualization
const transformToSchemaNodes = (
  schemas: SchemaVariant[],
  components: ComponentInList[]
): SchemaNode[] => {
  return schemas.map(schema => ({
    id: schema.id,
    schemaVariant: schema,
    components: components.filter(c => c.schema_variant_id === schema.id),
    position: { x: 0, y: 0 }, // Will be calculated by ELK
    dimensions: calculateNodeDimensions(schema, components)
  }))
}

// Extract relationships from component connections
const extractSchemaRelationships = (
  components: ComponentInList[],
  actions: ActionView[]
): SchemaRelationship[] => {
  // Use component socket connections and action dependencies
  // to infer schema-level relationships
}
```

## 1.4 Data Sources Integration:**
- **SchemaVariant**: Primary node data (name, category, color, description)
- **ComponentInList**: Component instances per schema, qualification stats
- **ActionView**: Dependencies between components → schema relationships  
- **FuncRunView**: Execution context for interactive debugging
- **AuditLog**: Historical changes for timeline view

## 1.5 Visualization Design

### 1.5.1 Graph Layout (ELK.js + D3.js)

**Layout Configuration:**
```typescript
const elkLayoutOptions = {
  'elk.algorithm': 'layered', // or 'hierarchical' for tree-like schemas
  'elk.direction': 'DOWN',
  'elk.spacing.nodeNode': '50',
  'elk.layered.spacing.nodeNodeBetweenLayers': '100',
  'elk.spacing.portPort': '10',
  'elk.portConstraints': 'FIXED_SIDE'
}

// Follow existing Map.vue patterns:
const useGraphLayout = () => {
  const layoutNodes = async (nodes: SchemaNode[], edges: SchemaRelationship[]) => {
    const elkGraph = transformToElkGraph(nodes, edges)
    const laidOutGraph = await elk.layout(elkGraph)
    return transformFromElkGraph(laidOutGraph)
  }
}
```

**D3.js Rendering Strategy:**
- **SVG-based rendering** following Map.vue patterns
- **Zoom/pan behavior** using d3-zoom
- **Smooth transitions** for layout changes
- REUSE AS MUCH FROM Map.vue or turn into composables as necessary for common behavior

### Schema Node Rendering

**Node Structure:**
```typescript
interface SchemaNodeVisual {
  header: {
    title: string        // SchemaVariant.display_name
    category: string     // SchemaVariant.category
    color: string        // SchemaVariant.color
    icon?: string        // Based on category
  }
  body: {
    componentCount: number
    qualificationStats: ComponentQualificationStats
    properties: PropTree // Condensed view of key properties
  }
  footer: {
    status: 'healthy' | 'warning' | 'error' // Based on qualification totals
    lastModified?: Date
  }
}
*** Create interfaces for all of these entities: 
- Component (combine ComponentInList and Component)
- Schema (including SchemaMembers)
- SchemaVariant
- Functions (from FuncRunLogView)
- AuditLogEntry (from si/lib/si-frontend-types-rs/src/audit_log.rs)
```

## 1.6 Visual Design:**
- **Rectangle nodes** with rounded corners matching SI design system
- **Header bar** with entity kind (1 per entity kind)
- **Property preview** showing key entity properties (for example, for EntityKind: SchemaVariant, properties include the properties in the type in rust. Same for Component)
- **Link to table view for that entity kind** for showing a table view for that entity kind (example, SchemaVariant node has a link that when clicked, will eventually show a table view of all schemavariants in the workspace)

### 1.6.1 Relationship Visualization

**Edge Types & Styling:**
```typescript
interface RelationshipEdge {
  id: string
  source: string
  target: string
}
```

**Relationship Detection:**
- **Entity Dependencies**: Create edges that reflect the foreign key for a given entity. For example, Component has a SchemaVariantId, so there should be an edge from SchemaVariant node's Id property to the Component's schemaVariantId property

## User Interaction Features

### 1.7.1 Navigation & Selection
- **Pan/Zoom**: D3 zoom behavior with wheel and drag support - reused from Map.vue, creating shared composables as necessary

### Interactive Features - Phase 2
- **Hover States**: Show component count, qualification stats
- **Context Menus**: Right-click for schema/component actions
- **Keyboard Navigation**: Arrow keys, Enter for selection
- **Minimap Integration**: Overview navigation for large graphs

### 2.0.0 State Management
```typescript
const useGraphInteraction = () => {
  const selectedSchemas = ref<Set<string>>(new Set())
  const hoveredSchema = ref<string | null>(null)
  const searchQuery = ref('')
  
  // URL state persistence like existing Map.vue
  const syncWithURL = () => {
    // Persist selection and view state in URL params
  }
}
```

## 2.1.0 Implementation Structure

### 2.1.1 File Organization
```
app/web/src/components/GraphSchemaVisualizer/
├── GraphSchemaVisualizer.vue          # Main component
├── composables/
│   ├── useSchemaGraph.ts             # Graph data logic if necessary
│   ├── useSchemaLayout.ts            # ELK layout logic if necessary
├── components/
│   ├── EntityNode.vue                # Individual node rendering
│   ├── EntityEdge.vue                # Edge rendering
└── types/
    └── schema-graph.ts               # TypeScript interfaces
```

## 2.2.0 Dependencies

### Required Libraries
- **elkjs**: Layout algorithms (already in use)
- **d3**: SVG manipulation and rendering (already in use)
- **graphology**: Graph data structures (available)

### Existing Patterns to Follow
- **Map.vue**: SVG rendering, zoom/pan, layout patterns
- **MiniMap.vue**: Overview navigation, clustering
- **Component architecture**: Composition API, TypeScript, Pinia stores
- **SI Design System**: Colors, typography, component styles

## 2.3.0 Implementation Phases

### 2.3.1 Phase 1: Core Infrastructure
1. Create component structure and file organization
2. Implement data transformation composables
3. Set up basic SVG rendering with D3.js
4. Integrate ELK.js layout computation

### 2.3.2 Phase 2: Node Rendering
1. Implement EntityNode.vue component
2. Add visual design matching SI patterns using tailwind css
3. Implement hover states and selection
4. Add component count and status indicators

### 3.0.0 Phase 3: Relationships & Interactions
1. Implement edge rendering and relationship detection
2. Add pan/zoom behavior and viewport management
3. Implement selection and filtering features
4. Add keyboard navigation support

