import {
  from,
  Observable,
  ReplaySubject,
  BehaviorSubject,
  combineLatest,
  Subject,
} from "rxjs";
import { switchMap, multicast, tap, map, refCount, take } from "rxjs/operators";
import {
  AttributeDal,
  IGetEntityReply,
  getEntityList,
  IGetEntityListReply,
  IUpdateEntityRequest,
  IUpdateEntityReply,
  IGetEntityRequest,
  IGetEntityListRequest,
  IGetConnectionsRequest,
  IGetConnectionsReply,
} from "@/api/sdf/dal/attributeDal";
import { IWorkspace } from "@/api/sdf/model/workspace";
import { IChangeSet } from "@/api/sdf/model/changeSet";
import { IEditSession } from "@/api/sdf/model/editSession";
import { Entity, IEntity } from "@/api/sdf/model/entity";
import { Diff } from "@/api/sdf/model/diff";
import {
  Qualification,
  QualificationStart,
} from "@/api/sdf/model/qualification";
import {
  IRunActionReply,
  IRunActionRequest,
  WorkflowDal,
} from "./api/sdf/dal/workflowDal";
import {
  WorkflowRun,
  WorkflowRunStep,
  WorkflowRunStepEntity,
} from "./api/sdf/model/workflow";
import {
  ISchematic,
  ISchematicNode,
  SchematicKind,
} from "./api/sdf/model/schematic";

export const workspace$ = new ReplaySubject<IWorkspace | null>(1);
workspace$.next(null);
export const changeSet$ = new ReplaySubject<IChangeSet | null>(1);
changeSet$.next(null);
export const editSession$ = new ReplaySubject<IEditSession | null>(1);
editSession$.next(null);
export const applicationId$ = new ReplaySubject<string | null>(1);
applicationId$.next(null);
export const system$ = new ReplaySubject<IEntity | null>(1);
system$.next(null);

export const editMode$ = new BehaviorSubject<boolean>(false);

new BehaviorSubject(false);

export interface AttributePanelEntityUpdate {
  entity: Entity;
  diff: Diff;
  qualifications: Qualification[];
}

export const attributePanelEntityUpdates$ = new Subject<
  AttributePanelEntityUpdate
>();

export function getEntity(
  entityId: string,
  workspace: IWorkspace | null,
  changeSet: IChangeSet | null,
  editSession: IEditSession | null,
): Observable<IGetEntityReply> {
  if (workspace && entityId) {
    const request: IGetEntityRequest = {
      entityId,
      workspaceId: workspace.id,
    };
    if (changeSet) {
      request["changeSetId"] = changeSet.id;
    }
    if (editSession) {
      request["editSessionId"] = editSession.id;
    }
    return from(AttributeDal.getEntity(request)).pipe(
      map(reply => {
        if (!reply.error) {
          reply.entity = Entity.fromJson(reply.entity);
        }
        return reply;
      }),
    );
  } else {
    let reply: IGetEntityReply = {
      error: {
        code: 42,
        message: "cannot get an entity without a workspace or entity id",
      },
    };
    return from([reply]);
  }
}

export function loadEntityForEdit(
  entityId: string,
): Observable<IGetEntityReply> {
  let observable = combineLatest(workspace$, changeSet$, editSession$).pipe(
    switchMap(args => getEntity(entityId, ...args)),
    tap(r => {
      if (!r.error) {
        r.entity = Entity.fromJson(r.entity);
      }
    }),
  );
  return observable;
}

export function getConnections(
  entityId: string,
  workspace: IWorkspace | null,
  changeSet: IChangeSet | null,
  editSession: IEditSession | null,
  editMode: Boolean,
): Observable<IGetConnectionsReply> {
  if (workspace && entityId) {
    const request: IGetConnectionsRequest = {
      entityId,
      workspaceId: workspace.id,
    };
    if (changeSet) {
      request["changeSetId"] = changeSet.id;
    }
    if (editMode && editSession) {
      request["editSessionId"] = editSession.id;
    }
    return from(AttributeDal.getConnections(request)).pipe(
      map(reply => {
        if (!reply.error) {
        }
        return reply;
      }),
    );
  } else {
    let reply: IGetConnectionsReply = {
      error: {
        code: 42,
        message: "cannot get direct edges without a workspace or entity id",
      },
    };
    return from([reply]);
  }
}

export function loadConnections(
  entityId: string,
): Observable<IGetConnectionsReply> {
  let observable = combineLatest(
    workspace$,
    changeSet$,
    editSession$,
    editMode$,
  ).pipe(switchMap(args => getConnections(entityId, ...args)));
  return observable;
}

export function updateEntity(entity: Entity): Observable<IUpdateEntityReply> {
  return combineLatest(workspace$, changeSet$, editSession$, system$).pipe(
    switchMap(([workspace, changeSet, editSession, system]) => {
      if (workspace?.id && changeSet?.id && editSession?.id) {
        let request: IUpdateEntityRequest = {
          workspaceId: workspace.id,
          changeSetId: changeSet.id,
          editSessionId: editSession.id,
          entity,
        };
        if (system && system.id) {
          request.systemId = system.id;
        }
        return from(AttributeDal.updateEntity(request)).pipe(
          map(reply => {
            if (!reply.error) {
              reply.entity = Entity.fromJson(reply.entity);
            }
            return reply;
          }),
        );
      } else {
        return from([
          {
            error: {
              message: "cannot save entity; missing required data! bug!",
              code: 42,
            },
          },
        ]);
      }
    }),
    tap(reply => {
      if (!reply.error) {
        attributePanelEntityUpdates$.next(reply);
        refreshEntityLabelList$.next(true);
      }
    }),
    take(1),
  );
}

export const refreshEntityLabelList$ = new BehaviorSubject<boolean>(true);

export const entityLabelList$: Observable<IGetEntityListReply> = combineLatest(
  applicationId$,
  workspace$,
  changeSet$,
  editSession$,
  refreshEntityLabelList$,
).pipe(
  map(([applicationId, workspace, changeSet, editSession]) => [
    applicationId,
    workspace?.id,
    changeSet?.id,
    editSession?.id,
  ]),
  switchMap(([applicationId, workspaceId, changeSetId, editSessionId]) => {
    if (applicationId && workspaceId) {
      let request: IGetEntityListRequest = {
        applicationId,
        workspaceId,
      };
      if (changeSetId) {
        request.changeSetId = changeSetId;
      }
      if (editSessionId) {
        request.editSessionId = editSessionId;
      }
      return from(getEntityList(request));
    } else {
      return from([
        {
          error: {
            code: 42,
            message:
              "cannot get list of entities for attribute panel, because a required bit of data is missing",
          },
        },
      ]);
    }
  }),
  multicast(new ReplaySubject(1)),
  refCount(),
);

export const entityQualifications$: Subject<Qualification> = new Subject();
export const entityQualificationStart$: Subject<QualificationStart> = new Subject();

export const workflowRuns$: Subject<WorkflowRun> = new Subject();
export const workflowRunSteps$: Subject<WorkflowRunStep> = new Subject();
export const workflowRunStepEntities$: Subject<WorkflowRunStepEntity> = new Subject();

// Schematic
export const deploymentSchematicSelectNode$ = new ReplaySubject<ISchematicNode | null>(
  1,
);
deploymentSchematicSelectNode$.next(null);

export const schematicSelectNode$ = new ReplaySubject<ISchematicNode | null>(1);
schematicSelectNode$.next(null);

export interface SchematicUpdated {
  schematicKind: SchematicKind;
  schematic: ISchematic;
}
export const schematicUpdated$ = new Subject<SchematicUpdated>();

export interface NodePositionUpdated {
  positionCtx: string;
}
export const nodePositionUpdated$ = new ReplaySubject<NodePositionUpdated | null>(
  1,
);
nodePositionUpdated$.next(null);

export interface EdgeCreating {
  entityType: string;
  schematicKind: string;
  entityId: string;
}
export const edgeCreating$ = new Subject<EdgeCreating | null>();

export interface EdgeDeleted {
  edgeId: string;
}
export const edgeDeleted$ = new BehaviorSubject<EdgeDeleted | null>(null);
