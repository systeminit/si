import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import { ISimpleStorable } from "@/api/sdf/model/siStorable";
import {
  IGetRequest,
  IGetReply,
  IListRequest,
  IListReply,
} from "@/api/sdf/model";
import { Query, Comparison } from "@/api/sdf/model/query";
import store from "@/store";
import _ from "lodash";

export interface IOrganization {
  id: string;
  name: string;
  siStorable: ISimpleStorable;
}

export class Organization implements IOrganization {
  id: IOrganization["id"];
  name: IOrganization["name"];
  siStorable: IOrganization["siStorable"];

  constructor(args: IOrganization) {
    this.id = args.id;
    this.name = args.name;
    this.siStorable = args.siStorable;
  }

  static async get(
    request: IGetRequest<IOrganization["id"]>,
  ): Promise<Organization> {
    const obj = await db.organizations.get(request.id);
    if (obj) {
      return new Organization(obj);
    }
    const reply: IGetReply<IOrganization> = await sdf.get(
      `organizations/${request.id}`,
    );
    const fetched: Organization = new Organization(reply.item);
    fetched.save();
    return fetched;
  }

  static async find(
    index: "id" | "name",
    value: string,
  ): Promise<Organization[]> {
    let organizations = await db.organizations
      .where(index)
      .equals(value)
      .toArray();
    if (!organizations.length) {
      const results = await Organization.list({
        query: Query.for_simple_string(index, value, Comparison.Equals),
      });
      return results.items;
    } else {
      return organizations.map(obj => new Organization(obj));
    }
  }

  static async list(request?: IListRequest): Promise<IListReply<Organization>> {
    const items: Organization[] = [];
    let totalCount = 0;

    db.organizations.each(obj => {
      items.push(new Organization(obj));
      totalCount++;
    });

    if (!totalCount) {
      let finished = false;
      while (!finished) {
        const reply: IListReply<IOrganization> = await sdf.list(
          "organizations",
          request,
        );
        if (reply.items.length) {
          for (let item of reply.items) {
            let objItem = new Organization(item);
            objItem.save();
            items.push(objItem);
          }
        }
        if (reply.pageToken) {
          request = {
            pageToken: reply.pageToken,
          };
        } else {
          totalCount = reply.totalCount;
          finished = true;
        }
      }
    }

    return {
      items,
      totalCount,
    };
  }

  async save(): Promise<void> {
    const currentObj = await db.organizations.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.organizations.put(this);
      await this.dispatch();
    }
  }

  async dispatch(): Promise<void> {
    // await store.dispatch("organizations/fromDb", this);
  }

  static async restore(): Promise<void> {
    let iObjects = await db.organizations.toArray();
    for (const iobj of iObjects) {
      let obj = new Organization(iobj);
      await obj.dispatch();
    }
  }
}

db.organizations.mapToClass(Organization);
