async function firstStanza() {
  const millis = Math.floor(Math.random() * 100);
  console.log(`First stanza is waiting ${millis} millis\n`);
  await new Promise(resolve => setTimeout(resolve, millis));
  console.log("I'm Brazilian, of median stature");
  console.log("I really like Fulana, but Sicrana is the one who wants me");
  console.log("Because in love, whoever loses almost always wins");
  console.log("Such a strange thing, avoid it if you can\n");
  return { payload: "first stanza return value", status: "ok" };
}
