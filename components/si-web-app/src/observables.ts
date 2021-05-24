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
  ResourceDal,
  IGetResourceRequest,
  IGetResourceReply,
} from "./api/sdf/dal/resourceDal";
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
import { Resource, SiEntity } from "si-entity";
import { IUser } from "./api/sdf/model/user";
import { IBillingAccount } from "./api/sdf/model/billingAccount";
import { IOrganization } from "./api/sdf/model/organization";
import { IApplicationCreateReplySuccess } from "./api/sdf/dal/applicationDal";

export const user$ = new ReplaySubject<IUser | null>(1);
user$.next(null);
export const billingAccount$ = new ReplaySubject<IBillingAccount | null>(1);
billingAccount$.next(null);
export const organization$ = new ReplaySubject<IOrganization | null>(1);
organization$.next(null);
export const workspace$ = new ReplaySubject<IWorkspace | null>(1);
workspace$.next(null);
export const changeSet$ = new ReplaySubject<IChangeSet | null>(1);
changeSet$.next(null);
export const revision$ = new ReplaySubject<IChangeSet | null>(1);
revision$.next(null);
export const editSession$ = new ReplaySubject<IEditSession | null>(1);
editSession$.next(null);
export const applicationId$ = new ReplaySubject<string | null>(1);
applicationId$.next(null);
export const applicaton$ = new ReplaySubject<IEntity | null>(1);
applicaton$.next(null);
export const system$ = new ReplaySubject<IEntity | null>(1);
system$.next(null);

export const editMode$ = new BehaviorSubject<boolean>(false);

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

export function getResource(
  entityId: string,
  system: IEntity | null,
  workspace: IWorkspace | null,
): Observable<IGetResourceReply> {
  if (entityId && system && workspace) {
    const request: IGetResourceRequest = {
      entityId,
      systemId: system.id,
      workspaceId: workspace.id,
    };
    return from(ResourceDal.getResource(request));
  } else {
    let reply: IGetResourceReply = {
      error: {
        code: 42,
        message:
          "cannot get a resource without an entity id, system, and workspace",
      },
    };
    return from([reply]);
  }
}

export function loadResource(entityId: string): Observable<IGetResourceReply> {
  let observable = combineLatest(system$, workspace$).pipe(
    switchMap(args => getResource(entityId, ...args)),
  );
  return observable;
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

export interface NameAttributeChanged {
  nodeId: string;
  entityId: string;
  entityType: string;
  oldValue: string;
  newValue: string;
}
export const nameAttributeChanged$ = new ReplaySubject<NameAttributeChanged | null>(
  1,
);
nameAttributeChanged$.next(null);

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

export const resources$: Subject<Resource> = new Subject();

export const applicationCreated$ = new ReplaySubject<IApplicationCreateReplySuccess | null>(
  1,
);
applicationCreated$.next(null);

export interface PanelTypeChange {
  panelRef: string;
  applicationId: string;
  panelType: string;
}

export const panelTypeChanges$ = new Subject<PanelTypeChange | null>();
export const restorePanelTypeChanges$ = new ReplaySubject<PanelTypeChange | null>();

export interface SchematicPanelState {
  panelRef: string;
  applicationId: string;
  schematicKind: string;
}

export const schematicPanelKind$ = new Subject<SchematicPanelState>();
export const restoreSchematicPanelKind$ = new ReplaySubject<
  SchematicPanelState
>();

export interface AttributePanelState {
  panelRef: string;
  applicationId: string;
  selectionIsLocked: boolean;
  selectedEntityId: string;
  activeView: string;
}

export const attributePanelState$ = new Subject<AttributePanelState>();
export const restoreAttributePanelState$ = new ReplaySubject<
  AttributePanelState
>();

export const refreshSecretList$ = new BehaviorSubject<true>(true);
