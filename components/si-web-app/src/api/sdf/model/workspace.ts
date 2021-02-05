import { ISimpleStorable } from "@/api/sdf/model/siStorable";

export interface IWorkspace {
  id: string;
  name: string;
  siStorable: ISimpleStorable;
}

export class Workspace implements IWorkspace {
  id: IWorkspace["id"];
  name: IWorkspace["name"];
  siStorable: IWorkspace["siStorable"];

  constructor(args: IWorkspace) {
    this.id = args.id;
    this.name = args.name;
    this.siStorable = args.siStorable;
  }

  static upgrade(obj: Workspace | IWorkspace): Workspace {
    if (obj instanceof Workspace) {
      return obj;
    } else {
      return new Workspace(obj);
    }
  }

  async updateStores() {
    //const bottle = Bottle.pop("default");
    //const store = bottle.container.Store;
    //    await store.dispatch("billingAccount/fromDb", this);
  }
}
