import { Query, OrderByDirection } from "@/api/sdf/model/query";
import { db } from "@/api/sdf/dexie";

export interface IGetReply<T> {
  item: T;
}

export interface IGetRequest<T> {
  id: T;
}

export interface ICreateReply<T> {
  item: T;
}

export interface IListRequest {
  query?: Query;
  pageSize?: number;
  orderBy?: string;
  orderByDirection?: OrderByDirection;
  pageToken?: string;
}

export interface IListReply<T> {
  items: T[];
  totalCount: number;
  pageToken?: string;
}
