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
  schematicKind$: Rx.ReplaySubject<SchematicKind | null>;

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

    this.schematicKind$ = new Rx.ReplaySubject<SchematicKind | null>(1);
    this.schematicKind$.next(null);

    this.initialize();
  }

  initialize(): void {
    this.nodeUpdate$.subscribe({ next: (d) => this.updateNodePosition(d) });
    this.connectionCreate$.subscribe({ next: (d) => this.createConnection(d) });
    this.nodeCreate$.subscribe({ next: (d) => this.createNode(d) });
  }

  async updateNodePosition(nodeUpdate: NodeUpdate | null): Promise<void> {
    const editorContext = await Rx.firstValueFrom(this.editorContext$);
    const schematicKind = await Rx.firstValueFrom(this.schematicKind$);
    if (nodeUpdate && editorContext && schematicKind) {
      SchematicService.setNodePosition({
        schematicKind,
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

  createConnection(connection: ConnectionCreate | null): void {
    if (connection) {
      SchematicService.createConnection({
        headSocketId: connection.destinationSocketId,
        headNodeId: connection.destinationNodeId,
        tailSocketId: connection.sourceSocketId,
        tailNodeId: connection.sourceNodeId,
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
      });
    }
    // remove temporary node
  }
}
