import couchbase from "couchbase";

import { environment } from "@/environment";

const cluster = new couchbase.Cluster(environment.couchbase.cluster, {
  username: environment.couchbase.username,
  password: environment.couchbase.password,
});
const bucket = cluster.bucket(environment.couchbase.bucket);

export const cdb = {
  cluster,
  bucket,
};
