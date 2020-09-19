import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import { ISimpleStorable } from "@/api/sdf/model/siStorable";
import { IGetRequest, IGetReply } from "@/api/sdf/model";
import store from "@/store";

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

  static async login(request: IUserLoginRequest): Promise<IUserLoginReply> {
    let userLoginReply: IUserLoginReply = await sdf.post(
      "users/login",
      request,
    );
    const user = new User(userLoginReply.user);
    await user.save();

    // Store the token, so all requests in the future are authenticated
    sdf.token = userLoginReply.jwt;

    return userLoginReply;
  }

  static async get(request: IGetRequest<IUser["id"]>): Promise<User> {
    const user = await db.users.get(request.id);
    if (user) {
      return new User(user);
    }
    const reply: IGetReply<IUser> = await sdf.get(`users/${request.id}`);
    const fetched: User = new User(reply.item);
    fetched.save();
    return fetched;
  }

  logout() {
    sdf.token = undefined;
  }

  async save(): Promise<string> {
    let result = await db.users.put(this);
    await store.dispatch("users/fromDb", this);
    return result;
  }
}

db.users.mapToClass(User);
