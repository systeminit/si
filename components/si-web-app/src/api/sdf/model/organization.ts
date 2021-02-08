import { ISimpleStorable } from "@/api/sdf/model/siStorable";

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

  static upgrade(obj: Organization | IOrganization): Organization {
    if (obj instanceof Organization) {
      return obj;
    } else {
      return new Organization(obj);
    }
  }

  async updateStores() {
    //const bottle = Bottle.pop("default");
    //const store = bottle.container.Store;
    //    await store.dispatch("billingAccount/fromDb", this);
  }
}
