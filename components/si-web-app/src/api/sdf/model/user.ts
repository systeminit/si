import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import { ISimpleStorable } from "@/api/sdf/model/siStorable";
import { IGetRequest, IGetReply } from "@/api/sdf/model";
import { wipe } from "@/api/sdf/dexie";
import store from "@/store";
import _ from "lodash";

export interface IUser {
  id: string;
  name: string;
  email: string;
  siStorable: ISimpleStorable;
}

export interface IUserLoginRequest {
  billingAccountName: string;
  email: string;
  password: string;
}

export interface IUserLoginReply {
  user: IUser;
  jwt: string;
}

export class User implements IUser {
  id: IUser["id"];
  name: IUser["name"];
  email: IUser["email"];
  siStorable: IUser["siStorable"];

  constructor(args: IUser) {
    this.id = args.id;
    this.name = args.name;
    this.email = args.email;
    this.siStorable = args.siStorable;
  }

  static upgrade(obj: User | IUser): User {
    if (obj instanceof User) {
      return obj;
    } else {
      return new User(obj);
    }
  }

  static async login(request: IUserLoginRequest): Promise<User> {
    let userLoginReply: IUserLoginReply = await sdf.post(
      "users/login",
      request,
    );
    const user = new User(userLoginReply.user);
    await wipe();
    await user.save();

    // Store the token, so all requests in the future are authenticated
    sdf.token = userLoginReply.jwt;

    return user;
  }

  static async get(request: IGetRequest<IUser["id"]>): Promise<User> {
    const user = await db.users.get(request.id);
    if (user) {
      return new User(user);
    }
    const reply: IGetReply<IUser> = await sdf.get(`users/${request.id}`);
    const fetched: User = new User(reply.item);
    await fetched.save();
    return fetched;
  }

  async logout() {
    sdf.token = undefined;
    if (sdf.update) {
      sdf.update.socket.close();
    }
    await wipe();
  }

  async save(): Promise<void> {
    const currentObj = await db.users.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.users.put(this);
      await store.dispatch("users/fromDb", this);
    }
  }
}

db.users.mapToClass(User);
