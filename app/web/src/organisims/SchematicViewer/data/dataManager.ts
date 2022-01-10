import * as Rx from "rxjs";
import _ from "lodash";

import { NodeUpdate, ConnectionCreate, Schematic } from "../model";

import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
import { ApiResponse } from "@/api/sdf";
import { SetNodeResponse } from "@/service/schematic/set_node";
import { NodeCreate } from "./event";

import { schematicData$ as schematicDataGlobal$ } from "./observable";

export class SchematicDataManager {
  id: string;
  schematicData$: Rx.ReplaySubject<Schematic | null>;
  nodeUpdate$: Rx.ReplaySubject<NodeUpdate | null>;
  connectionCreate$: Rx.ReplaySubject<ConnectionCreate | null>;
  nodeCreate$: Rx.ReplaySubject<NodeCreate | null>;

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

    this.initialize();
  }

  initialize(): void {
    this.nodeUpdate$.subscribe({ next: (d) => this.updateNodePosition(d) });
    this.connectionCreate$.subscribe({ next: (d) => this.createConnection(d) });
    this.nodeCreate$.subscribe({ next: (d) => this.createNode(d) });
  }

  updateNodePosition(nodeUpdate: NodeUpdate | null): void {
    if (nodeUpdate) {
      SchematicService.setNode({ name: "canoe" }).subscribe(
        (response: ApiResponse<SetNodeResponse>) => {
          if (response.error) {
            GlobalErrorService.set(response);
          }
	  // TODO: fetch schematic when position is set

          //const d = schematicDataAfter;
          // this.schematicData$.next(d);
          //schematicDataGlobal$.next(nodeUpdate);
        },
      );
    }
  }

  createConnection(nodeUpdate: ConnectionCreate | null): void {
    if (nodeUpdate) {
      SchematicService.createConnection({ name: "canoe" }).subscribe(
        (response: ApiResponse<SetNodeResponse>) => {
          if (response.error) {
            GlobalErrorService.set(response);
          }
          const d = schematicDataAfter;
          // this.schematicData$.next(d);
          schematicDataGlobal$.next(d);
        },
      );
    }
  }

  createNode(e: NodeCreate | null): void {
    if (e && e.nodeSchemaId) {
      SchematicService.createNode({ schemaId: e.nodeSchemaId }).subscribe(
        (response) => {
          if (response.error) {
            GlobalErrorService.set(response);
          }
          console.log("Node created on backend", { response });
        },
      );
    }
    // remove temporary node
  }
}
