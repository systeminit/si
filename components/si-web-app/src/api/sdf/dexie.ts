import Dexie from "dexie";
import { IBillingAccount } from "@/api/sdf/model/billingAccount";
import { IUser } from "@/api/sdf/model/user";
import { IWorkspace } from "@/api/sdf/model/workspace";

class SiDatabase extends Dexie {
  // Declare implicit table properties.
  // (just to inform Typescript. Instanciated by Dexie in stores() method)
  billingAccounts: Dexie.Table<IBillingAccount, string>;
  users: Dexie.Table<IUser, string>;
  workspaces: Dexie.Table<IWorkspace, string>;

  constructor() {
    super("SiDatabase");
    this.version(1).stores({
      billingAccounts: "id, name",
      users: "id, email",
      workspaces:
        "id, name, [siStorable.billingAccountId+siStorable.organizationId+name]",
    });

    // The following line is needed if your typescript
    // is compiled using babel instead of tsc:
    this.billingAccounts = this.table("billingAccounts");
    this.users = this.table("users");
    this.workspaces = this.table("workspaces");
  }
}

export const db = new SiDatabase();
