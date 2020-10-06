import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import { ISimpleStorable } from "@/api/sdf/model/siStorable";
import { IGetRequest, IGetReply } from "@/api/sdf/model";
import store from "@/store";

export interface IBillingAccount {
  id: string;
  name: string;
  description: string;
  siStorable: ISimpleStorable;
}

export interface IBillingAccountCreateRequest {
  billingAccountName: string;
  billingAccountDescription: string;
  userName: string;
  userEmail: string;
  userPassword: string;
}

export interface IBillingAccountCreateReply {
  billingAccount: IBillingAccount;
  user: Record<string, any>;
  group: Record<string, any>;
  organization: Record<string, any>;
  workspace: Record<string, any>;
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
  static async get(
    request: IGetRequest<IBillingAccount["id"]>,
  ): Promise<BillingAccount> {
    const obj = await db.billingAccounts.get(request.id);
    if (obj) {
      return new BillingAccount(obj);
    }
    const reply: IGetReply<IBillingAccount> = await sdf.get(
      `billingAccounts/${request.id}`,
    );
    const fetched: BillingAccount = new BillingAccount(reply.item);
    fetched.save();
    return fetched;
  }

  static async create(
    request: IBillingAccountCreateRequest,
  ): Promise<IBillingAccountCreateReply> {
    const billingAccountReply: IBillingAccountCreateReply = await sdf.post(
      "billingAccounts",
      request,
    );
    const billingAccount = new BillingAccount(
      billingAccountReply.billingAccount,
    );
    await billingAccount.save();
    return billingAccountReply;
  }

  async save(): Promise<string> {
    let result = await db.billingAccounts.put(this);
    await store.dispatch("billingAccount/fromDb", this);
    return result;
  }
}

db.billingAccounts.mapToClass(BillingAccount);
