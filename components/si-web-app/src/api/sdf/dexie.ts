import Dexie from "dexie";
import { IBillingAccount } from "@/api/sdf/model/billingAccount";

class SiDatabase extends Dexie {
  // Declare implicit table properties.
  // (just to inform Typescript. Instanciated by Dexie in stores() method)
  billingAccounts: Dexie.Table<IBillingAccount, string>; // number = type of the primkey

  constructor() {
    super("SiDatabase");
    this.version(1).stores({
      billingAccounts: "id, name",
    });

    // The following line is needed if your typescript
    // is compiled using babel instead of tsc:
    this.billingAccounts = this.table("billingAccounts");
  }
}

export const db = new SiDatabase();

export interface ISimpleStorable {
  typeName: String;
  objectId: String;
  billingAccountId: String;
  tenantIds: String[];
  deleted: boolean;
}
