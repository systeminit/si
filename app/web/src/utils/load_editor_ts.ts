import tsUrl from "@/assets/static/editor_typescript.txt";

const resp = await fetch(tsUrl);
const editor_ts = await resp.text();

export { editor_ts };
