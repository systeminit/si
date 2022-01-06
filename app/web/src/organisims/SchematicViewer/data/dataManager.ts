import * as Rx from "rxjs";
import _ from "lodash";

import { NodeUpdate, ConnectionCreate, Schematic } from "../model";

import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
import { ApiResponse } from "@/api/sdf";
import { SetNodeResponse } from "@/service/schematic/set_node";
import { schematicDataAfter } from "../model/testDatasetAfter";
import { schematicData$ as schematicDataGlobal$ } from "./observable";

export class SchematicDataManager {
  id: string;
  schematicData$: Rx.ReplaySubject<Schematic | null>;
  nodeUpdate$: Rx.ReplaySubject<NodeUpdate | null>;
  connectionCreate$: Rx.ReplaySubject<ConnectionCreate | null>;
  addNode$: Rx.ReplaySubject<string | null>;

  constructor() {
    this.id = _.uniqueId();

    this.schematicData$ = new Rx.ReplaySubject<Schematic | null>(1);
    this.schematicData$.next(null);

    this.addNode$ = new Rx.ReplaySubject<string | null>(1);
    this.addNode$.next(null);

    this.nodeUpdate$ = new Rx.ReplaySubject<NodeUpdate | null>(1);
    this.nodeUpdate$.next(null);

    this.connectionCreate$ = new Rx.ReplaySubject<ConnectionCreate | null>(1);
    this.connectionCreate$.next(null);

    this.initialize();
  }

  initialize(): void {
    this.nodeUpdate$.subscribe({ next: (d) => this.updateNodePosition(d) });
    this.connectionCreate$.subscribe({ next: (d) => this.createConnection(d) });
    this.addNode$.subscribe({ next: (d) => this.addNode(d) });
  }

  updateNodePosition(nodeUpdate: NodeUpdate | null): void {
    if (nodeUpdate) {
      SchematicService.setNode({ name: "canoe" }).subscribe(
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

  addNode(nodeType: string | null) {
    console.log("dataManager::addNode()", nodeType);
  }
}
