import Dexie from "dexie";
import { IBillingAccount } from "@/api/sdf/model/billingAccount";
import { IUser } from "@/api/sdf/model/user";
import { IOrganization } from "@/api/sdf/model/organization";
import { IWorkspace } from "@/api/sdf/model/workspace";
import { IChangeSet, IChangeSetParticipant } from "@/api/sdf/model/changeSet";
import { IEditSession } from "@/api/sdf/model/editSession";
import { INode } from "@/api/sdf/model/node";
import { IEntity } from "@/api/sdf/model/entity";
import { ISystem } from "@/api/sdf/model/system";
import { IEdge } from "@/api/sdf/model/edge";
import { IUpdateClockGlobal } from "@/api/sdf/model/update";
import { IEntityOps } from "@/api/sdf/model/ops";
import { IEventLog } from "@/api/sdf/model/eventLog";
import { IResource } from "@/api/sdf/model/resource";

class SiDatabase extends Dexie {
  // Declare implicit table properties.
  // (just to inform Typescript. Instanciated by Dexie in stores() method)
  billingAccounts: Dexie.Table<IBillingAccount, string>;
  users: Dexie.Table<IUser, string>;
  organizations: Dexie.Table<IOrganization, string>;
  workspaces: Dexie.Table<IWorkspace, string>;
  changeSets: Dexie.Table<IChangeSet, string>;
  changeSetParticipants: Dexie.Table<IChangeSetParticipant, string>;
  editSessions: Dexie.Table<IEditSession, string>;
  nodes: Dexie.Table<INode, string>;
  headEntities: Dexie.Table<IEntity, string>;
  projectionEntities: Dexie.Table<IEntity, string>;
  systems: Dexie.Table<ISystem, string>;
  edges: Dexie.Table<IEdge, string>;
  globalUpdateClock: Dexie.Table<IUpdateClockGlobal, string>;
  entityOps: Dexie.Table<IEntityOps, string>;
  eventLog: Dexie.Table<IEventLog, string>;
  resources: Dexie.Table<IResource, string>;

  constructor() {
    super("SiDatabase");
    this.version(1).stores({
      billingAccounts: "id, name",
      users: "id, email",
      organizations: "id, name",
      workspaces:
        "id, name, [siStorable.billingAccountId+siStorable.organizationId+name]",
      changeSets: "id, name",
      changeSetParticipants: "id, changeSetId, objectId",
      nodes: "id",
      headEntities: "id, nodeId, objectType",
      projectionEntities:
        "[id+siChangeSet.changeSetId], [nodeId+siChangeSet.changeSetId], id, nodeId, objectType",
      editSessions: "id, name, changeSetId",
      systems: "id, name, nodeId",
      edges:
        "id, [kind+headVertex.nodeId], [kind+tailVertex.nodeId], [kind+tailVertex.objectId], [kind+headVertex.objectId], [kind+tailVertex.typeName+headVertex.typeName], [kind+tailVertex.typeName+headVertex.objectId]",
      globalUpdateClock: "id, [epoch+updateClock]",
      entityOps:
        "id, toId, siChangeSet.changeSetId, [siChangeSet.changeSetId+toId]",
      eventLog: "id",
      resources: "id, [systemId+entityId], [systemId+nodeId]",
    });

    // The following line is needed if your typescript
    // is compiled using babel instead of tsc:
    this.billingAccounts = this.table("billingAccounts");
    this.users = this.table("users");
    this.workspaces = this.table("workspaces");
    this.organizations = this.table("organizations");
    this.changeSets = this.table("changeSets");
    this.changeSetParticipants = this.table("changeSetParticipants");
    this.nodes = this.table("nodes");
    this.headEntities = this.table("headEntities");
    this.projectionEntities = this.table("projectionEntities");
    this.editSessions = this.table("editSessions");
    this.systems = this.table("systems");
    this.edges = this.table("edges");
    this.globalUpdateClock = this.table("globalUpdateClock");
    this.entityOps = this.table("entityOps");
    this.eventLog = this.table("eventLog");
    this.resources = this.table("resources");
  }
}

export let db = new SiDatabase();

export async function wipe() {
  await db.delete();
  db = new SiDatabase();
}
