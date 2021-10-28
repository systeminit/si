// import Bottle from "bottlejs";
import { ISimpleStorable } from "@/api/sdf/model/siStorable";
// import { IGetRequest, IGetReply } from "@/api/sdf/model";
import _ from "lodash";

export interface IUser {
  id: string;
  name: string;
  email: string;
  siStorable: ISimpleStorable;
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
}
