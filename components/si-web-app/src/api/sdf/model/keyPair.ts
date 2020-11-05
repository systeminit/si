import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import _ from "lodash";
import store from "@/store";

export interface IPublicKey {
  id: string;
  name: string;
  publicKey: number[] | Uint8Array;
  siStorable: ISiStorable;
}

export class PublicKey implements IPublicKey {
  id: IPublicKey["id"];
  name: IPublicKey["name"];
  publicKey: Uint8Array;
  siStorable: IPublicKey["siStorable"];

  constructor(args: IPublicKey) {
    this.id = args.id;
    this.name = args.name;
    this.publicKey = Uint8Array.from(args.publicKey);
    this.siStorable = args.siStorable;
  }

  async save(): Promise<void> {
    const currentObj = await db.keyPairs.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.keyPairs.put(this);
      await this.dispatch();
    }
  }

  async dispatch(): Promise<void> {
    await store.dispatch("secret/fromPublicKey", this);
  }

  static async restore(): Promise<void> {
    let iObjects = await db.keyPairs.toArray();
    for (const iobj of iObjects) {
      let obj = new PublicKey(iobj);
      await obj.dispatch();
    }
  }
}

db.keyPairs.mapToClass(PublicKey);
