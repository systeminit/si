import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import _ from "lodash";
import { Base64 } from "js-base64";
import Bottle from "bottlejs";

export interface IPublicKey {
  id: string;
  name: string;
  publicKey: Uint8Array | string;
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
    if (_.isString(args.publicKey)) {
      this.publicKey = Base64.toUint8Array(args.publicKey);
    } else {
      this.publicKey = args.publicKey;
    }
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
    const bottle = Bottle.pop("default");
    const store = bottle.container.Store;
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
