exports.title = async () => {
  console.log("Fallible Title: Lero Lero / Bullshit - https://www.youtube.com/watch?v=UuA8WnZMIDc\n");
  throw new Error("title failed");
}

exports.title2 = async (err) => {
  console.error("Title 1 threw:", err);
  console.log("Lero Lero / Bullshit - https://www.youtube.com/watch?v=UuA8WnZMIDc\n");
}

exports.firstStanza = async () => {
  const secs = Math.floor(Math.random() * 10);
  console.log(`First stanza is waiting ${secs} seconds\n`);
  await new Promise(resolve => setTimeout(resolve, secs * 1000));
  console.log("I'm Brazilian, of median stature");
  console.log("I really like Fulana, but Sicrana is the one who wants me");
  console.log("Because in love, whoever loses almost always wins");
  console.log("Such a strange thing, avoid it if you can\n");
  return "first stanza return value";
}

exports.secondStanza = async (value) => {
  console.log(`First stanza returned: ${value}\n`);
  console.log("I don't hold a grudge, I don't blaspheme, I don't ponder");
  console.log("I don't tolerate bullshit, I don't owe anyone anything");
  console.log("I'm well rested, I carry my life with strength");
  console.log("From the beat to heavy manual labor, I do as I please\n");
}

exports.thirdStanza = async () => {
  console.log("I am a poet and I don't deny my kind");
  console.log("I make verses out of precision and spite");
  console.log("With a broken foot, a blank verse and a rich rhyme");
  console.log("I deny, I give a hint, I have my own way\n");
}

exports.fourthStanza = async () => {
  console.log("I'm Brazilian, Six-banded Armadillo, Taturana");
  console.log("I play soccer well, I'm bad at budgeting, I know the multiplications table by heart");
  console.log("Four times seven twenty eight nine's out");
  console.log("Either the Jaguar eats me or I will have a great laught\n");
}

exports.fifthStanza = async () => {
  console.log("I don't enter raffles, I don't sweeten, I don't season");
  console.log("I don't reschedule, ground-zero, if I said something I don't take it back");
  console.log("I leave a trail, I lay fame");
  console.log("I mess up the whole plot, I defy satan\n");
}

exports.sixthStanza = async () => {
  console.log("I'm Brazilian, of median stature");
  console.log("I really like Fulana, but Sicrana is the one who wants me\n");
}

exports.seventhStanza = async () => {
  console.log("Because in love, whoever loses almost always wins");
  console.log("Such a strange thing, avoid it if you can");
  console.log("As a proverb from my region says");
  console.log("A good goatling is the one that screams the most wherever the thrush sings");
  console.log("I don't believe in my fate's bad luck");
  console.log("Preying Tico-Tico, nobody takes my cornmeal\n");
  return "seventh stanza return value";
}

exports.question = async (personName, arg) => {
  const secs = Math.floor(Math.random() * 10);
  console.log(`Question is waiting ${secs} seconds, arg = ${arg}`);
  await new Promise(resolve => setTimeout(resolve, secs * 1000));
  console.log(`What about you, are you a brazilian of median stature ${personName}?`);
}

exports.bye = async (arg) => {
  const secs = Math.floor(Math.random() * 10);
  console.log(`Bye is waiting ${secs} seconds, arg = ${arg}`);
  await new Promise(resolve => setTimeout(resolve, secs * 1000));
  console.log("bye");
}
