import tsUrl from "@/assets/static/editor_typescript.txt";

let editor_ts: string | null = null;

const loadEditorTs = async (): Promise<string> => {
  const resp = await fetch(tsUrl);
  editor_ts = await resp.text();
  return editor_ts;
};

export { editor_ts, loadEditorTs };
