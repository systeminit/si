import { computed, ref } from "vue";
import type { 
  EntityKind,
  BifrostActionViewList,
  AttributeTree,
  ComponentInList,
  ComponentDiff,
  ComponentDetails,
  SchemaVariant,
  SchemaMembers
} from "@/workers/types/entity_kind_types";
import { 
  Entity,
  EntityNode, 
  EntitySchemaKind,
  RelationshipEdge, 
  GraphData
} from "../types/schema-graph";
import EntityEdge from "../components/EntityEdge.vue";

export function useEntityGraph() {
  const selectedEntities = ref<Set<string>>(new Set());
  const hoveredEntity = ref<string | null>(null);

  const transformEntitiesToNodes = (entities: Entity[]): EntityNode[] => {
    return entities.map(entity => {
      // Calculate dimensions based on entity type and content
      const dimensions = calculateNodeDimensions(entity);
      
      return {
        id: entity.id,
        name: entity.name,
        entityKind: entity.entityKind,
        properties: entity.properties,
        position: { x: 0, y: 0 }, // Will be calculated by ELK
        dimensions
      };
    });
  };

  const calculateNodeDimensions = (entity: Entity): { width: number; height: number } => {
    // Base dimensions for table-like layout
    let width = 280; // Minimum width for table layout
    let height = 32; // Header height
    
    // Calculate height based on number of properties (18px per row)
    const propertyCount = entity.properties.length;
    const propertyHeight = propertyCount * 18; // 18px per property row
    height += propertyHeight;
    
    // Add footer height
    height += 24;
    
    // Adjust width based on longest property name + type to prevent overflow
    const maxPropertyLength = entity.properties.reduce((max, prop) => {
      // Account for property name + type + spacing
      const displayLength = prop.name.length + prop.type.length + 10; // 10 chars for spacing
      return Math.max(max, displayLength);
    }, entity.name.length + 10); // Include entity name
    
    // Approximate character width in pixels
    const charWidth = 7;
    const calculatedWidth = Math.max(280, maxPropertyLength * charWidth + 40); // 40px padding
    width = Math.min(calculatedWidth, 450); // Cap at 450px for table layout
    
    return { width, height };
  };

  const extractRelationshipsFromEntities = (entities: Entity[]): RelationshipEdge[] => {
    const relationships: RelationshipEdge[] = [];
    
    entities.forEach(entity => {
      // Look for foreign key relationships in properties
      entity.properties.forEach(prop => {
        if (prop.kind === 'foreign_key') {
          // Try to find the target entity (simplified matching)
          const targetEntity = entities.find(e => e.id.includes(prop.type) || e.entityKind === prop.type);
          if (targetEntity) {
            relationships.push({
              id: `${entity.id}-${targetEntity.id}`,
              sourceId: entity.id,
              targetId: targetEntity.id,
              arity: prop.arity || 'one',
              sourceProperty: prop.name,
              targetProperty: 'id',
            });
          }
        }
      });
    });

    return relationships;
  };

  const populateGraph = (): { entities: Entity[], relationships: RelationshipEdge[] } => {
    const entities: Entity[] = [];
    const primaryKeys:  Record<string, Entity> = {};
    // action entity
    const actionEntity: Entity = {
      id: 'actions',
      name: 'Actions',
      entityKind: EntitySchemaKind.Action,
      properties: [
        { name: 'actionId', type: 'string', kind: 'primary_key' },
        { name: 'kind', type: 'string', kind: 'value' },
        { name: 'state', type: 'string', kind: 'value' },
        {name: 'description', type: 'string', kind: 'value' },
        {name: 'originatingChangeSetId', type: 'string', kind: 'value' },
        { name: 'cEntitySchemaKind.ActionId', type: 'string', arity: 'one', kind: 'foreign_key' },
        { name: 'schemaVariantId', type: 'string', arity: 'one', kind: 'foreign_key'},
        {name: 'funcId', type: 'string', arity: 'one', kind: 'foreign_key' },
      ]
    };
    entities.push(actionEntity);
    // schema variant entity
    const schemaVariantEntity: Entity = {
      id: 'schemaVariants',
      name: 'Schema Variants',
      entityKind: EntitySchemaKind.SchemaVariant,
      properties: [
        { name: 'schemaVariantId', type: 'string', kind: 'primary_key' },
        { name: 'schemaName', type: 'string', kind: 'value' },
        { name: 'displayName', type: 'string', kind: 'value' },
        { name: 'category', type: 'string', kind: 'value' },
        { name: 'color', type: 'string', kind: 'value' },
        { name: 'link', type: 'string', kind: 'value' },
        { name: 'description', type: 'string', kind: 'value' },
        { name: 'schemaDocLinks', type: 'string', kind: 'value' }
      ]
    };
    entities.push(schemaVariantEntity);
    // schema entity
    const schemaEntity: Entity = {
      id: 'schemas',
      name: 'Schemas',
      entityKind: EntitySchemaKind.Schema,
      properties: [
        { name: 'schemaId', type: 'string', kind: 'primary_key' },
        { name: 'name', type: 'string', kind: 'value' },
        {name: 'description', type: 'string', kind: 'value' },
        { name: 'defaultVariantId', type: 'string', arity: 'one', kind: 'foreign_key' },
        { name: 'editingVariantId', type: 'string', arity: 'one', kind: 'foreign_key' }
      ]
    };
    entities.push(schemaEntity);
    // component entity
    const componentEntity: Entity = {
      id: 'components',
      name: 'Components',
      entityKind: EntitySchemaKind.Component,
      properties: [
        { name: 'id', type: 'string', kind: 'primary_key' },
        { name: 'name', type: 'string', kind: 'value' },
        { name: 'color', type: 'string', kind: 'value' },
        { name: 'schemaId', type: 'string', kind: 'value' },
        { name: 'schemaVariantId', type: 'string', arity: 'one', kind: 'foreign_key' },
        { name: 'aEntitySchemaKind.Componente', type: 'object', kind: 'value' },
        {name: 'source', type: 'componentId', kind: 'foreign_key', arity: 'many' },
       // {name: 'target', type: 'componentId', kind: 'foreign_key', arity: 'many' },
      ]
    };
    entities.push(componentEntity);
    // function entity
    const functionEntity: Entity = {
      id: 'functions',
      name: 'Functions',
      entityKind: EntitySchemaKind.Function,
      properties: [
        { name: 'id', type: 'string', kind: 'primary_key' },
        { name: 'state', type: 'string', kind: 'value' },
        { name: 'actor', type: 'string', kind: 'value' },
        { name: 'componentId', type: 'string', arity: 'one', kind: 'foreign_key' },
        { name: 'attributeValueId', type: 'string', kind: 'value' },
        { name: 'componentName', type: 'string', kind: 'value' },
        { name: 'schemaName', type: 'string', kind: 'value' },
        { name: 'actionId', type: 'string', arity: 'one', kind: 'foreign_key' },
        { name: 'actionKind', type: 'string', kind: 'value' },
        { name: 'actionDisplayName', type: 'string', kind: 'value' },
        { name: 'actionOriginatingChangeSetId', type: 'string', kind: 'value' },
        { name: 'actionResultState', type: 'string', kind: 'value' },
        { name: 'functionName', type: 'string', kind: 'value' },
        { name: 'functionDisplayName', type: 'string', kind: 'value' },
        { name: 'functionKind', type: 'string', kind: 'value' },
        { name: 'functionDescription', type: 'string', kind: 'value' },
        { name: 'functionLink', type: 'string', kind: 'value' },
        { name: 'createdAt', type: 'string', kind: 'value' },
        { name: 'updatedAt', type: 'string', kind: 'value' },
        { name: 'functionArgs', type: 'object', kind: 'value' },
        { name: 'functionCodeBase64', type: 'string', kind: 'value' },
        { name: 'resultValue', type: 'object', kind: 'value' },
        { name: 'logs', type: 'object', kind: 'value' },        
      ]
    };
    entities.push(functionEntity);

    // Create relationships based on foreign keys
    const relationships: RelationshipEdge[] = [];
    
    // Action entity relationships
    relationships.push(
      {
        id: 'action-component',
        sourceId: 'actions',
        targetId: 'components',
        arity: 'many',
        sourceProperty: 'componentId',
        targetProperty: 'id',
      },
      {
        id: 'action-schemavariant',
        sourceId: 'actions', 
        targetId: 'schemaVariants',
        arity: 'many',
        sourceProperty: 'schemaVariantId',
        targetProperty: 'schemaVariantId',
      },
      {
        id: 'action-function',
        sourceId: 'actions',
        targetId: 'functions', 
        arity: 'many',
        sourceProperty: 'funcId',
        targetProperty: 'id',
      }
    );

    // Schema entity relationships
    relationships.push(
      {
        id: 'schema-defaultvariant',
        sourceId: 'schemas',
        targetId: 'schemaVariants',
        arity: 'one',
        sourceProperty: 'defaultVariantId',
        targetProperty: 'schemaVariantId',
      },
      {
        id: 'schema-editingvariant', 
        sourceId: 'schemas',
        targetId: 'schemaVariants',
        arity: 'one',
        sourceProperty: 'editingVariantId',
        targetProperty: 'schemaVariantId',
      }
    );

    // Component entity relationships  
    relationships.push(
      {
        id: 'component-schemavariant',
        sourceId: 'components',
        targetId: 'schemaVariants',
        arity: 'many',
        sourceProperty: 'schemaVariantId',
        targetProperty: 'schemaVariantId',
      }
    );

    // Function entity relationships
    relationships.push(
      {
        id: 'function-component',
        sourceId: 'functions',
        targetId: 'components',
        arity: 'many', 
        sourceProperty: 'componentId',
        targetProperty: 'id',
      },
      {
        id: 'function-action',
        sourceId: 'functions',
        targetId: 'actions',
        arity: 'many',
        sourceProperty: 'actionId', 
        targetProperty: 'actionId',
      }
    );

    return { entities, relationships };
  };

  const createGraphData = (
    entities: Entity[] = [],
    providedRelationships: RelationshipEdge[] = []
  ): GraphData => {
    // Transform entities to nodes
    const nodes = transformEntitiesToNodes(entities);
    
    // Use provided relationships or extract from entities
    let edges = providedRelationships;
    if (edges.length === 0) {
      edges = extractRelationshipsFromEntities(entities);
    }
    
    // Filter edges to only include those between visible nodes
    const nodeIds = new Set(nodes.map(node => node.id));
    const filteredEdges = edges.filter(edge => 
      nodeIds.has(edge.sourceId) && nodeIds.has(edge.targetId)
    );

    return { nodes, edges: filteredEdges };
  };

  const getNodeById = (nodeId: string, graphData: GraphData): EntityNode | undefined => {
    return graphData.nodes.find(node => node.id === nodeId);
  };

  const getConnectedNodes = (nodeId: string, graphData: GraphData): EntityNode[] => {
    const connectedIds = new Set<string>();
    
    graphData.edges.forEach(edge => {
      if (edge.sourceId === nodeId) {
        connectedIds.add(edge.targetId);
      } else if (edge.targetId === nodeId) {
        connectedIds.add(edge.sourceId);
      }
    });

    return graphData.nodes.filter(node => connectedIds.has(node.id));
  };

  const selectEntity = (entityId: string) => {
    selectedEntities.value.add(entityId);
  };

  const deselectEntity = (entityId: string) => {
    selectedEntities.value.delete(entityId);
  };

  const clearSelection = () => {
    selectedEntities.value.clear();
  };

  const setHoveredEntity = (entityId: string | null) => {
    hoveredEntity.value = entityId;
  };

  return {
    selectedEntities: computed(() => selectedEntities.value),
    hoveredEntity: computed(() => hoveredEntity.value),
    createGraphData,
    populateGraph,
    getNodeById,
    getConnectedNodes,
    selectEntity,
    deselectEntity,
    clearSelection,
    setHoveredEntity,
    transformEntitiesToNodes,
    extractRelationshipsFromEntities,
    calculateNodeDimensions
  };
}