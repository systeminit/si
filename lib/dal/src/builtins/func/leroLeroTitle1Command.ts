async function title() {
  console.log(
    "Fallible Title: Lero Lero / Bullshit - https://www.youtube.com/watch?v=UuA8WnZMIDc\n",
  );
  throw new Error("title failed");
}
