import Dexie from "dexie";
import { IBillingAccount } from "@/api/sdf/model/billingAccount";
import { IUser } from "@/api/sdf/model/user";
import { IOrganization } from "@/api/sdf/model/organization";
import { IWorkspace } from "@/api/sdf/model/workspace";
import { IChangeSet } from "@/api/sdf/model/changeSet";
import { IEditSession } from "@/api/sdf/model/editSession";
import { INode } from "@/api/sdf/model/node";
import { IEntity } from "@/api/sdf/model/entity";
import { ISystem } from "@/api/sdf/model/system";
import { IEdge } from "@/api/sdf/model/edge";

class SiDatabase extends Dexie {
  // Declare implicit table properties.
  // (just to inform Typescript. Instanciated by Dexie in stores() method)
  billingAccounts: Dexie.Table<IBillingAccount, string>;
  users: Dexie.Table<IUser, string>;
  organizations: Dexie.Table<IOrganization, string>;
  workspaces: Dexie.Table<IWorkspace, string>;
  changeSets: Dexie.Table<IChangeSet, string>;
  editSessions: Dexie.Table<IEditSession, string>;
  nodes: Dexie.Table<INode, string>;
  entities: Dexie.Table<IEntity, string>;
  systems: Dexie.Table<ISystem, string>;
  edges: Dexie.Table<IEdge, string>;

  constructor() {
    super("SiDatabase");
    this.version(1).stores({
      billingAccounts: "id, name",
      users: "id, email",
      organizations: "id, name",
      workspaces:
        "id, name, [siStorable.billingAccountId+siStorable.organizationId+name]",
      changeSets: "id, name",
      nodes: "id",
      entities: "id, nodeId, objectType",
      editSessions: "id, name, changeSetId",
      systems: "id, name, nodeId",
      edges: "id",
    });

    // The following line is needed if your typescript
    // is compiled using babel instead of tsc:
    this.billingAccounts = this.table("billingAccounts");
    this.users = this.table("users");
    this.workspaces = this.table("workspaces");
    this.organizations = this.table("organizations");
    this.changeSets = this.table("changeSets");
    this.nodes = this.table("nodes");
    this.entities = this.table("entities");
    this.editSessions = this.table("editSessions");
    this.systems = this.table("systems");
    this.edges = this.table("edges");
  }
}

export let db = new SiDatabase();

export async function wipe() {
  await db.delete();
  db = new SiDatabase();
}
