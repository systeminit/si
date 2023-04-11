async function question(personName, arg) {
  const millis = Math.floor(Math.random() * 100);
  console.log(`Question is waiting ${millis} millis, arg = ${arg}`);
  await new Promise(resolve => setTimeout(resolve, millis));
  console.log(`What about you, are you a brazilian of median stature ${personName}?`);
  return { status: "ok" }
}
