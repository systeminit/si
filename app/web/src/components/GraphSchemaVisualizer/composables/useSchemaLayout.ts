import { ref, computed } from "vue";
import ELK from "elkjs/lib/elk.bundled.js";
import type { 
  GraphData, 
  EntityNode, 
  RelationshipEdge, 
  LayoutOptions,
  GraphViewportState 
} from "../types/schema-graph";

// ELK layout interfaces based on Map.vue patterns
interface ElkNode {
  id: string;
  width: number;
  height: number;
  x?: number;
  y?: number;
  children?: ElkNode[];
  [key: string]: any;
}

interface ElkEdge {
  id: string;
  sources: string[];
  targets: string[];
  sections?: any[];
  [key: string]: any;
}

interface ElkGraph {
  id: string;
  children: ElkNode[];
  edges: ElkEdge[];
  layoutOptions: Record<string, string>;
}

interface LayoutResult {
  nodes: EntityNode[];
  edges: RelationshipEdge[];
  bounds: {
    minX: number;
    minY: number;
    maxX: number;
    maxY: number;
  };
}

export function useSchemaLayout() {
  const isLayouting = ref(false);
  const layoutError = ref<string | null>(null);
  const elk = new ELK();

  const defaultLayoutOptions: LayoutOptions = {
    algorithm: 'layered',
    spacing: {
      nodeNode: 50,
      layerSeparation: 100,
      portPort: 10
    },
    direction: 'DOWN'
  };

  const convertToElkGraph = (
    graphData: GraphData, 
    options: LayoutOptions = defaultLayoutOptions
  ): ElkGraph => {
    const elkNodes: ElkNode[] = graphData.nodes.map(node => ({
      id: node.id,
      width: node.dimensions.width,
      height: node.dimensions.height,
      // Store the original entity data for later retrieval
      entityKind: node.entityKind,
      name: node.name,
      properties: node.properties
    }));
    console.log('elkEdges', graphData.edges);
    const elkEdges: ElkEdge[] = graphData.edges.map(edge => ({
      id: edge.id,
      sources: [edge.sourceId],
      targets: [edge.targetId],
      // Store relationship metadata
      sourceProperty: edge.sourceProperty,
      targetProperty: edge.targetProperty
    }));

    return {
      id: "schema-graph-root",
      children: elkNodes,
      edges: elkEdges,
      layoutOptions: {
        "elk.algorithm": options.algorithm,
        "elk.direction": options.direction,
        "elk.spacing.nodeNode": options.spacing.nodeNode.toString(),
        "elk.layered.spacing.nodeNodeBetweenLayers": options.spacing.layerSeparation.toString(),
        "elk.spacing.portPort": options.spacing.portPort.toString(),
        "elk.padding": "[top=25,left=25,bottom=25,right=25]"
      }
    };
  };

  const convertFromElkGraph = (elkResult: any): LayoutResult => {
    const nodes: EntityNode[] = elkResult.children.map((elkNode: any) => ({
      id: elkNode.id,
      name: elkNode.name,
      entityKind: elkNode.entityKind,
      properties: elkNode.properties,
      position: { 
        x: elkNode.x || 0, 
        y: elkNode.y || 0 
      },
      dimensions: { 
        width: elkNode.width, 
        height: elkNode.height 
      }
    }));

    const edges: RelationshipEdge[] = elkResult.edges.map((elkEdge: any) => ({
      id: elkEdge.id,
      source: elkEdge.sources[0],
      target: elkEdge.targets[0],
      sourceProperty: elkEdge.sourceProperty,
      targetProperty: elkEdge.targetProperty
    }));

    // Calculate bounds for viewport management
    const bounds = calculateBounds(nodes);
    console.log('Layout bounds:', bounds);
    console.log('Layout nodes:', nodes);
    console.log('Layout edges:', edges);
    return { nodes, edges, bounds };
  };

  const calculateBounds = (nodes: EntityNode[]) => {
    if (nodes.length === 0) {
      return { minX: 0, minY: 0, maxX: 0, maxY: 0 };
    }

    const bounds = nodes.reduce(
      (acc, node) => ({
        minX: Math.min(acc.minX, node.position.x),
        minY: Math.min(acc.minY, node.position.y),
        maxX: Math.max(acc.maxX, node.position.x + node.dimensions.width),
        maxY: Math.max(acc.maxY, node.position.y + node.dimensions.height)
      }),
      {
        minX: Infinity,
        minY: Infinity,
        maxX: -Infinity,
        maxY: -Infinity
      }
    );

    return bounds;
  };

  const layoutGraph = async (
    graphData: GraphData, 
    options: LayoutOptions = defaultLayoutOptions
  ): Promise<LayoutResult> => {
    if (isLayouting.value) {
      throw new Error("Layout operation already in progress");
    }

    try {
      isLayouting.value = true;
      layoutError.value = null;

      if (graphData.nodes.length === 0) {
        return { 
          nodes: [], 
          edges: [], 
          bounds: { minX: 0, minY: 0, maxX: 0, maxY: 0 } 
        };
      }

      const elkGraph = convertToElkGraph(graphData, options);
      const elkResult = await elk.layout(elkGraph);
      console.log('ELK layout result:', elkResult);
      return convertFromElkGraph(elkResult);

    } catch (error) {
      layoutError.value = error instanceof Error ? error.message : 'Unknown layout error';
      console.error('Layout error:', error);
      
      // Return original positions as fallback
      return {
        nodes: graphData.nodes,
        edges: graphData.edges,
        bounds: calculateBounds(graphData.nodes)
      };
    } finally {
      isLayouting.value = false;
    }
  };

  const getNodeAtPosition = (
    nodes: EntityNode[], 
    x: number, 
    y: number
  ): EntityNode | null => {
    return nodes.find(node => 
      x >= node.position.x && 
      x <= node.position.x + node.dimensions.width &&
      y >= node.position.y && 
      y <= node.position.y + node.dimensions.height
    ) || null;
  };

  const getViewportForBounds = (
    bounds: LayoutResult['bounds'], 
    containerWidth: number, 
    containerHeight: number,
    padding = 50
  ): GraphViewportState => {
    const contentWidth = bounds.maxX - bounds.minX;
    const contentHeight = bounds.maxY - bounds.minY;
    
    if (contentWidth === 0 || contentHeight === 0) {
      return { x: 0, y: 0, scale: 1 };
    }

    const scaleX = (containerWidth - padding * 2) / contentWidth;
    const scaleY = (containerHeight - padding * 2) / contentHeight;
    const scale = Math.min(scaleX, scaleY, 1); // Don't scale up beyond 100%

    const x = (containerWidth - contentWidth * scale) / 2 - bounds.minX * scale;
    const y = (containerHeight - contentHeight * scale) / 2 - bounds.minY * scale;

    return { x, y, scale };
  };

  return {
    isLayouting: computed(() => isLayouting.value),
    layoutError: computed(() => layoutError.value),
    layoutGraph,
    getNodeAtPosition,
    getViewportForBounds,
    calculateBounds
  };
}