import uuidv4 from "uuid/v4";

import { cdb } from "@/db";

interface GetResultUser {
  value: User;
}

interface UserInterface {
  id: string;
  email: string;
  name?: string;
  __typename: string;
}

export class User implements UserInterface {
  public readonly id!: string;
  public email!: string;
  public name?: string;
  public __typename = "User";

  constructor({
    name,
    email,
    user,
  }: {
    name?: string;
    email?: string;
    user?: UserInterface;
  }) {
    if (user !== undefined) {
      for (const key of Object.keys(user)) {
        this[key] = user[key];
      }
    } else {
      this.id = uuidv4();
      this.name = name;
      this.email = email;
    }
  }

  public get fqId(): string {
    return `user:${this.id}`;
  }

  public static async getByFqId(fqid: string): Promise<User> {
    const col = cdb.bucket.defaultCollection();
    const result = await col.get(fqid);
    const user: UserInterface = result.value;
    return new User({ user: user });
  }

  public static async getById(uuid: string): Promise<User> {
    const col = cdb.bucket.defaultCollection();
    const result = await col.get(`user:${uuid}`);
    const user: UserInterface = result.value;
    return new User({ user: user });
  }

  public static async createOrReturn({
    email,
    name,
  }: {
    email: string;
    name: string;
  }): Promise<User> {
    let userPointer: GetResultUser;
    const col = cdb.bucket.defaultCollection();

    try {
      userPointer = await col.get(`user:email:${email}`);
    } catch (e) {
      const user = new User({ name, email });
      await col.insert(`user:${user.id}`, user);
      await col.insert(`user:email:${email}`, `user:${user.id}`);
      return user;
    }
    const user = await col.get(userPointer.value);
    return new User({ user: user.value });
  }
}
