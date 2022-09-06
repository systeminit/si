/*
add the following line to .vscode/settings.json
"tailwindCSS.experimental.classRegex": [["tw`([^`]*)", // tw`...`]]
*/

/** noop string template tag used to help IDE enable autocomplete on tailwind classes */
export const tw = String.raw;
