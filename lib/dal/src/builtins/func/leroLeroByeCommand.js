async function bye(arg) {
  const millis = Math.floor(Math.random() * 100);
  console.log(`Bye is waiting ${millis} millis, arg = ${arg}`);
  await new Promise(resolve => setTimeout(resolve, millis));
  console.log("bye");
  return { status: "ok" }
}
