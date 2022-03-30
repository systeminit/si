import * as PIXI from "pixi.js";
import * as Rx from "rxjs";
import { untilUnmounted } from "vuse-rx";

// Should we bypass the datamanager here?
import { editSession$ } from "@/observable/edit_session";
import { SocketType } from "../obj";
import { SceneManager } from "../scene";
import { SchematicDataManager } from "../../data";
import * as ST from "../../state";
import { Renderer } from "../renderer";
import { Interpreter } from "xstate";
import { SelectionManager } from "./selection";
import { DraggingManager } from "./dragging";
import { PanningManager } from "./panning";
import { ConnectingManager } from "./connecting";
import { ZoomingManager } from "./zooming";
import { NodeAddManager } from "./nodeAdd";

// import { PanningInteractionData } from "./interaction/panning";

export interface InteractionState {
  context: {
    mouse: {
      position: {
        x: number;
        y: number;
      };
    };
    transform: {
      offset: {
        x: number;
        y: number;
      };
    };
  };
}

// interface Interaction {
//   data: PIXI.InteractionData | undefined;
//   isDraggingNode: boolean;
//   isPanning: boolean;
//   spaceBarPressed: boolean;
//   socketClicked: boolean;
//   isConnectingNode: boolean;
//   isReadyToConnectNodes: boolean;
//   offset: Position;
//   selection: Array<Node>;
//   sourceSocket: string | undefined;
//   destinationSocket: string | undefined;
// }

export class InteractionManager {
  // interaction: Interaction;
  sceneManager: SceneManager;
  dataManager: SchematicDataManager;
  stateService: Interpreter<unknown>;
  renderer: Renderer;
  selectionManager: SelectionManager;
  draggingManager: DraggingManager;
  panningManager: PanningManager;
  connectingManager: ConnectingManager;
  zoomingManager: ZoomingManager;
  nodeAddManager: NodeAddManager;
  zoomMagnitude$: Rx.ReplaySubject<number | null>;
  zoomFactor$: Rx.ReplaySubject<number | null>;
  zoomMagnitude?: number | null;
  zoomFactor?: number | null;

  constructor(
    sceneManager: SceneManager,
    dataManager: SchematicDataManager,
    stateService: Interpreter<unknown>,
    renderer: Renderer,
  ) {
    this.stateService = stateService;
    this.sceneManager = sceneManager;
    this.dataManager = dataManager;
    this.renderer = renderer;

    this.sceneManager.scene.on("pointerdown", this.onMouseDown, this);
    this.sceneManager.scene.on("pointermove", this.onMouseMove, this);
    this.sceneManager.scene.on("pointerup", this.onMouseUp, this);
    this.sceneManager.scene.on("pointerupoutside", this.onMouseUp, this);

    this.zoomMagnitude$ = new Rx.ReplaySubject<number | null>(1);
    this.zoomMagnitude$.next(1);

    this.zoomMagnitude$.pipe(untilUnmounted).subscribe({
      next: (v) => this.setZoomMagnitude(v),
    });

    this.zoomFactor$ = new Rx.ReplaySubject<number | null>(1);
    this.zoomFactor$.next(1);

    this.zoomFactor$.pipe(untilUnmounted).subscribe({
      next: (v) => this.setZoomFactor(v),
    });

    this.selectionManager = new SelectionManager();
    this.draggingManager = new DraggingManager(sceneManager, dataManager);
    this.panningManager = new PanningManager();
    this.connectingManager = new ConnectingManager(dataManager);
    this.zoomingManager = new ZoomingManager(
      sceneManager.root,
      renderer,
      this.zoomMagnitude$,
      this.zoomFactor$,
    );
    this.nodeAddManager = new NodeAddManager(
      sceneManager,
      dataManager,
      this.selectionManager,
      renderer,
    );
  }

  setZoomMagnitude(zoomMagnitude: number | null): void {
    if (zoomMagnitude) {
      this.zoomMagnitude = zoomMagnitude;
    }
  }

  setZoomFactor(zoomFactor: number | null): void {
    if (zoomFactor) {
      this.zoomFactor = zoomFactor;
    }
  }

  async onMouseDown(
    this: InteractionManager,
    e: PIXI.InteractionEvent,
  ): Promise<void> {
    const editSession = await Rx.firstValueFrom(editSession$);
    const schematicKind = await Rx.firstValueFrom(
      this.dataManager.schematicKind$,
    );

    const target = this.renderer.plugins.interaction.hitTest(e.data.global);
    const isFakeNode = target.id === -1;
    const canEdit = editSession && !isFakeNode;

    if (target.name === "scene") {
      if (ST.isPanningActivated(this.stateService)) {
        ST.initiatePanning(this.stateService);
        const root = this.renderer.stage.getChildByName("root", true);
        const offset = {
          x: e.data.global.x - root.worldTransform.tx,
          y: e.data.global.y - root.worldTransform.ty,
        };
        this.panningManager.beforePan(e.data, offset);
        this.renderer.renderStage();
      } else {
        ST.readySelecting(this.stateService);
        ST.deSelecting(this.stateService);

        if (schematicKind) {
          console.debug("Deselecting node");
          const selectionObserver = this.selectionManager.selectionObserver(
            schematicKind,
          );
          this.selectionManager.clearSelection(selectionObserver);
          this.renderer.renderStage();
        }
      }
    }

    if (target.kind === "node") {
      if (ST.isPanningActivated(this.stateService)) {
        ST.initiatePanning(this.stateService);
        const sceneGeo = this.renderer.stage.getChildByName("root", true);
        const offset = {
          x: e.data.global.x - sceneGeo.worldTransform.tx,
          y: e.data.global.y - sceneGeo.worldTransform.ty,
        };
        this.panningManager.beforePan(e.data, offset);
        this.renderer.renderStage();
      } else {
        ST.readySelecting(this.stateService);
        ST.selecting(this.stateService);

        if (!isFakeNode && schematicKind) {
          console.debug("Selecting real node");
          const selectionObserver = this.selectionManager.selectionObserver(
            schematicKind,
          );
          this.selectionManager.select(target, selectionObserver);
        }

        if (canEdit) {
          ST.activateDragging(this.stateService);

          let zoomFactor = 1;
          if (this.zoomFactor) {
            zoomFactor = this.zoomFactor;
          }
          const offset = {
            x: (e.data.global.x - target.worldTransform.tx) * (1 / zoomFactor),
            y: (e.data.global.y - target.worldTransform.ty) * (1 / zoomFactor),
          };
          this.draggingManager.beforeDrag(e.data, offset);
        }
        this.renderer.renderStage();
      }
    }

    if (canEdit) {
      if (target.kind === "socket") {
        if (ST.isPanningActivated(this.stateService)) {
          ST.initiatePanning(this.stateService);
          const sceneGeo = this.renderer.stage.getChildByName("root", true);
          const offset = {
            x: sceneGeo.worldTransform.tx,
            y: sceneGeo.worldTransform.ty,
          };
          this.panningManager.beforePan(e.data, offset);
          this.renderer.renderStage();
        } else {
          if (target.type === SocketType.output) {
            ST.activateConnecting(this.stateService);
            const sceneGeo = this.renderer.stage.getChildByName("root", true);
            const offset = {
              x: sceneGeo.worldTransform.tx,
              y: sceneGeo.worldTransform.ty,
            };

            let zoomFactor = 1;
            if (this.zoomFactor) {
              zoomFactor = this.zoomFactor;
            }

            this.connectingManager.beforeConnect(
              e.data,
              target,
              this.sceneManager,
              offset,
              zoomFactor,
            );
            this.renderer.renderStage();
          }
        }
      }
    }

    // Adding a node
    const canAdd = !!editSession;
    if (canAdd && ST.isAddingNode(this.stateService)) {
      this.nodeAddManager.afterAddNode();
      ST.deactivateNodeAdd(this.stateService);
    }
  }

  onMouseMove(this: InteractionManager, e: PIXI.InteractionEvent) {
    // Panning
    if (this.stateService.state.value === ST.ViewerState.PANNING_INITIATED) {
      this.stateService.send({ type: ST.ViewerEventKind.PANNING });
    }
    if (ST.isPanningInitiated(this.stateService)) {
      ST.panning(this.stateService);
    }

    if (ST.isPanning(this.stateService)) {
      this.panningManager.pan(e.data, this.sceneManager.root);
      this.renderer.renderStage();
    }

    // Dragging
    if (ST.isDraggingActivated(this.stateService)) {
      ST.initiateDragging(this.stateService);
    }
    if (ST.isDraggingInitiated(this.stateService)) {
      ST.dragging(this.stateService);
    }
    if (ST.isDragging(this.stateService)) {
      const node = this.selectionManager.selection[0];
      this.draggingManager.drag(node);
      this.sceneManager.refreshConnections();
      this.renderer.renderStage();
    }

    // Connecting
    if (ST.isConnectingActivated(this.stateService)) {
      ST.initiateConnecting(this.stateService);
    }
    if (ST.isConnectingInitiated(this.stateService)) {
      ST.connecting(this.stateService);
      this.connectingManager.drag(e.data, this.sceneManager);
      this.renderer.renderStage();
    }
    if (ST.isConnecting(this.stateService)) {
      const target = this.renderer.plugins.interaction.hitTest(e.data.global);
      if (target && target.kind === "socket") {
        if (target.type === SocketType.input) {
          this.connectingManager.connect(target.name);
          this.renderer.renderStage();
        }
      } else {
        ST.connecting(this.stateService);
        this.connectingManager.drag(e.data, this.sceneManager);
        this.renderer.renderStage();
      }
    }

    // Adding node
    if (ST.isNodeAddActivated(this.stateService)) {
      this.renderer.renderStage();
      ST.initiateNodeAdd(this.stateService);
    }
    if (ST.isNodeAddInitiated(this.stateService)) {
      this.nodeAddManager.beforeAddNode(e.data);
      ST.addingNode(this.stateService);
    }
    if (ST.isAddingNode(this.stateService)) {
      this.nodeAddManager.drag();
    }
  }

  onMouseUp(this: InteractionManager) {
    // Panning
    if (
      ST.isPanning(this.stateService) ||
      ST.isPanningActivated(this.stateService) ||
      ST.isPanningInitiated(this.stateService)
    ) {
      ST.deactivatePanning(this.stateService);
      this.renderer.renderStage();
    }

    // Selecting
    if (
      ST.isSelecting(this.stateService) ||
      ST.isDeselecting(this.stateService)
    ) {
      ST.deactivateSelecting(this.stateService);
      this.renderer.renderStage();
    }

    // Connecting
    if (ST.isConnecting(this.stateService)) {
      ST.connectingToSocket(this.stateService);
      this.connectingManager.afterConnect(this.sceneManager);
      this.renderer.renderStage();
    }
    if (
      ST.isConnectingToSocket(this.stateService) ||
      ST.isConnectingActivated(this.stateService) ||
      ST.isConnectingInitiated(this.stateService)
    ) {
      ST.deactivateConnecting(this.stateService);
      this.connectingManager.clearInteractiveConnection(this.sceneManager);
      this.renderer.renderStage();
    }

    // Dragging
    if (
      ST.isDragging(this.stateService) ||
      ST.isDraggingActivated(this.stateService) ||
      ST.isDraggingInitiated(this.stateService)
    ) {
      const node = this.selectionManager.selection[0];
      this.draggingManager.afterDrag(node);
      ST.deactivateDragging(this.stateService);
      this.renderer.renderStage();
    }
  }
}
