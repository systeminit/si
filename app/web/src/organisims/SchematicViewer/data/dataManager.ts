import * as Rx from "rxjs";
import _ from "lodash";

import { NodeUpdate, ConnectionCreate, Schematic } from "../model";

import { SchematicKind } from "@/api/sdf/dal/schematic";
import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
import { ApiResponse } from "@/api/sdf";
import { NodeCreate } from "./event";
import { EditorContext } from "@/api/sdf/dal/schematic";
import { CreateConnectionResponse } from "@/service/schematic/create_connection";
import { SetNodePositionResponse } from "@/service/schematic/set_node_position";

// import { schematicData$ as schematicDataGlobal$ } from "./observable";

export class SchematicDataManager {
  id: string;
  schematicData$: Rx.ReplaySubject<Schematic | null>;
  nodeUpdate$: Rx.ReplaySubject<NodeUpdate | null>;
  connectionCreate$: Rx.ReplaySubject<ConnectionCreate | null>;
  nodeCreate$: Rx.ReplaySubject<NodeCreate | null>;
  editorContext$: Rx.ReplaySubject<EditorContext | null>;

  constructor() {
    this.id = _.uniqueId();

    // TODO: define dataManagerEvent types... and refactor the following observables.

    this.schematicData$ = new Rx.ReplaySubject<Schematic | null>(1);
    this.schematicData$.next(null);

    this.nodeCreate$ = new Rx.ReplaySubject<NodeCreate | null>(1);
    this.nodeCreate$.next(null);

    this.nodeUpdate$ = new Rx.ReplaySubject<NodeUpdate | null>(1);
    this.nodeUpdate$.next(null);

    this.connectionCreate$ = new Rx.ReplaySubject<ConnectionCreate | null>(1);
    this.connectionCreate$.next(null);

    this.editorContext$ = new Rx.ReplaySubject<EditorContext | null>(1);
    this.editorContext$.next(null);

    this.initialize();
  }

  initialize(): void {
    this.nodeUpdate$.subscribe({ next: (d) => this.updateNodePosition(d) });
    this.connectionCreate$.subscribe({ next: (d) => this.createConnection(d) });
    this.nodeCreate$.subscribe({ next: (d) => this.createNode(d) });
  }

  async updateNodePosition(nodeUpdate: NodeUpdate | null): Promise<void> {
    const editorContext: EditorContext | null = await Rx.firstValueFrom(
      this.editorContext$,
    );
    if (nodeUpdate && editorContext) {
      SchematicService.setNodePosition({
        schematicKind: SchematicKind.Component,
        x: `${nodeUpdate.position.x}`,
        y: `${nodeUpdate.position.y}`,
        nodeId: nodeUpdate.nodeId,
        rootNodeId: editorContext.applicationNodeId,
        systemId: editorContext.systemId,
      }).subscribe((response: ApiResponse<SetNodePositionResponse>) => {
        if (response.error) {
          GlobalErrorService.set(response);
        }
        // this.schematicData$.next(d);
        //schematicDataGlobal$.next(nodeUpdate);
      });
    }
  }

  createConnection(nodeUpdate: ConnectionCreate | null): void {
    if (nodeUpdate) {
      // FIXME(nick): these values are temporary.
      SchematicService.createConnection({
        headSocketId: 1,
        headNodeId: 1,
        tailSocketId: 2,
        tailNodeId: 2,
      }).subscribe((response: ApiResponse<CreateConnectionResponse>) => {
        if (response.error) {
          GlobalErrorService.set(response);
        }
        // const d = schematicDataAfter;
        // this.schematicData$.next(d);
        // schematicDataGlobal$.next(d);
      });
    }
  }

  createNode(e: NodeCreate | null): void {
    if (e) {
      SchematicService.createNode({
        schemaId: e.nodeSchemaId,
        rootNodeId: e.rootNodeId,
        systemId: e.systemId,
        x: e.x,
        y: e.y,
      }).subscribe((response) => {
        if (response.error) {
          GlobalErrorService.set(response);
        }
        console.log("Node created on backend", { response });
      });
    }
    // remove temporary node
  }
}
