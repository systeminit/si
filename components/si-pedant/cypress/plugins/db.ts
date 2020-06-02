import couchbase from "couchbase";
const cluster = new couchbase.Cluster("couchbase://localhost", {
  username: "si",
  password: "bugbear",
});
const bucket = cluster.bucket("si");
const collection = bucket.defaultCollection();

export async function dbDeleteByTypeName(
  typeName: string,
  billingAccountId: string,
): Promise<boolean> {
  const query = `
    DELETE FROM si as s 
      WHERE s.siStorable.typeName = "${typeName}" 
        AND ARRAY_CONTAINS(s.siStorable.tenantIds, "${billingAccountId}")
      RETURNING s
  `;
  try {
    const result = await cluster.query(query);
    console.log("delete result", result);
    for (const delObj of result.rows) {
      try {
        const delObjName = delObj["s"]["name"];
        const lookupResult = await collection.remove(
          `${billingAccountId}:${typeName}:${delObjName}`,
        );
        console.log("delete lookupResult", lookupResult);
      } catch (err) {
        console.log(err);
      }
    }
  } catch (err) {
    console.log(err);
  }
  return true;
}

export async function dbDeleteByName(
  typeName: string,
  name: string,
  billingAccountId: string,
): Promise<boolean> {
  const query = `
    DELETE FROM si as s 
      WHERE s.siStorable.typeName = "${typeName}" 
        AND s.name = "${name}" 
        AND ARRAY_CONTAINS(s.siStorable.tenantIds, "${billingAccountId}")
      RETURNING s
  `;
  try {
    const result = await cluster.query(query);
    console.log("delete result", result);
    const lookupResult = await collection.remove(
      `${billingAccountId}:${typeName}:${name}`,
    );
    console.log("delete lookupResult", lookupResult);
  } catch (err) {
    console.log(err);
  }
  return true;
}
