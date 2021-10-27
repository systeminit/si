import { ISimpleStorable } from "@/api/sdf/model/siStorable";
// import { IGetRequest, IGetReply } from "@/api/sdf/model";
// import Bottle from "bottlejs";

export interface IBillingAccount {
  id: string;
  name: string;
  description: string;
  siStorable: ISimpleStorable;
}

export class BillingAccount implements IBillingAccount {
  id: IBillingAccount["id"];
  name: IBillingAccount["name"];
  description: IBillingAccount["description"];
  siStorable: IBillingAccount["siStorable"];

  constructor(args: IBillingAccount) {
    this.id = args.id;
    this.name = args.name;
    this.description = args.description;
    this.siStorable = args.siStorable;
  }

  static upgrade(obj: BillingAccount | IBillingAccount): BillingAccount {
    if (obj instanceof BillingAccount) {
      return obj;
    } else {
      return new BillingAccount(obj);
    }
  }
}
