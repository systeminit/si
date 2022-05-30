import * as Rx from "rxjs";
import _ from "lodash";

import { Schematic, SchematicKind } from "@/api/sdf/dal/schematic";
import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
import { NodeCreate, ConnectionCreate } from "./event";
import { EditorContext } from "@/api/sdf/dal/schematic";

export class SchematicDataManager {
  id: string;
  schematicData$: Rx.ReplaySubject<Schematic | null>;
  editorContext$: Rx.ReplaySubject<EditorContext | null>;
  schematicKind$: Rx.ReplaySubject<SchematicKind | null>;
  selectedDeploymentNodeId?: number;

  constructor() {
    this.id = _.uniqueId();

    this.selectedDeploymentNodeId = undefined;

    this.schematicData$ = new Rx.ReplaySubject<Schematic | null>(1);
    this.schematicData$.next(null);

    this.editorContext$ = new Rx.ReplaySubject<EditorContext | null>(1);
    this.editorContext$.next(null);

    this.schematicKind$ = new Rx.ReplaySubject<SchematicKind | null>(1);
    this.schematicKind$.next(null);
  }

  async updateNodePosition(
    nodeId: number,
    x: number,
    y: number,
  ): Promise<void> {
    const editorContext = await Rx.firstValueFrom(this.editorContext$);
    const schematicKind = await Rx.firstValueFrom(this.schematicKind$);
    if (editorContext && schematicKind && nodeId !== -1) {
      SchematicService.setNodePosition({
        deploymentNodeId: this.selectedDeploymentNodeId,
        schematicKind,
        x: `${x}`,
        y: `${y}`,
        nodeId: nodeId,
        systemId: editorContext.systemId,
      }).subscribe((response) => {
        if (response.error) {
          GlobalErrorService.set(response);
        }
      });
    }
  }

  createConnection(connection: ConnectionCreate): void {
    SchematicService.createConnection({
      headSocketId: connection.destinationSocketId,
      headNodeId: connection.destinationNodeId,
      tailSocketId: connection.sourceSocketId,
      tailNodeId: connection.sourceNodeId,
    }).subscribe((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
      }
    });
  }

  createNode(e: NodeCreate): void {
    SchematicService.createNode({
      schemaId: e.nodeSchemaId,
      systemId: e.systemId,
      x: e.x,
      y: e.y,
      parentNodeId: e.parentNodeId,
    }).subscribe((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
      }
    });
  }
}
