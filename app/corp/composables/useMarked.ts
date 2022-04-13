import { marked } from "marked";
import hljs from "highlight.js";

export const useMarked = () => {
  marked.setOptions({
    highlight: (code: string, lang: string) => {
      const language = hljs.getLanguage(lang) ? lang : "plaintext";
      return hljs.highlight(code, { language }).value;
    },
    langPrefix: "hljs language-",
    smartLists: true,
    smartypants: true,
  });
  return marked;
};
