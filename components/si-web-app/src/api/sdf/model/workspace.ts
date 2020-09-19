import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import { ISimpleStorable } from "@/api/sdf/model/siStorable";
import { IGetRequest, IGetReply } from "@/api/sdf/model";
import { User } from "@/api/sdf/model/user";
import store from "@/store";

export interface IWorkspace {
  id: string;
  name: string;
  siStorable: ISimpleStorable;
}

export interface IWorkspaceDefaultForUserRequest {
  user_id: string;
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

  static async get(request: IGetRequest<IWorkspace["id"]>): Promise<Workspace> {
    const workspace = await db.workspaces.get(request.id);
    if (workspace) {
      return new Workspace(workspace);
    }
    const reply: IGetReply<IWorkspace> = await sdf.get(
      `workspaces/${request.id}`,
    );
    const fetched: Workspace = new Workspace(reply.item);
    fetched.save();
    return fetched;
  }

  // ... Actually, you need to implement list for things like organization, and then workspace! that will
  // make this logic way easier.

  static async default_for_user(
    request: IWorkspaceDefaultForUserRequest,
  ): Promise<Workspace> {
    let user = await User.get({ id: request.user_id });
    let workspaceResult = db.workspaces.where({
      siStorable: { billingAccountId: user.siStorable.billingAccountId },
      name: "default",
    });
    let workspace: Workspace | null = (await workspaceResult.toArray()).pop();
    if (workspace) {
      return workspace;
    }
  }

  async save(): Promise<string> {
    let result = await db.workspaces.put(this);
    await store.dispatch("workspace/fromDb", this);
    return result;
  }
}

db.users.mapToClass(Workspace);
