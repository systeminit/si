# app/web

## Project setup
```
pnpm install
```

### Compiles and hot-reloads for development
```
pnpm run start
```

### Compiles and minifies for production
```
pnpm run build
```

### Lint check (no fix)
```
pnpm run lint
```

### Type check (no fix)
```
pnpm run build:check
```

### Code formatting check (no fix)
```
pnpm run fmt:check
```

### Format code (WILL FIX)
```
pnpm run fmt
```

# Vue 3 + Typescript + Vite

This template should help get you started developing with Vue 3 and Typescript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## IDE Setup Instructions
### [VSCode](https://code.visualstudio.com/) (preferred)
  - Install [Volar](https://marketplace.visualstudio.com/items?itemName=johnsoncodehk.volar) plugin and [Typescript Volar](https://marketplace.visualstudio.com/items?itemName=Vue.vscode-typescript-vue-plugin) plugin
    - and disable Vetur if installed
  - Enable TS "takeover mode" (see [here](https://github.com/johnsoncodehk/volar/discussions/471))
    - run "Extensions: Show built-in extensions" from command pallete
    - search for "@builtin typescript" in extensions panel
    - find "Typescript + Javascript Language Features", click gear and "Disable (Workspace)"
  - Use workspace's typescript version
    - run "Volar: Select Typescript Version" from command pallete
    - select "Use workspace version"
  - Install [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint) plugin
    - NOTE - you don't need the prettier plugin, as it will be running via the eslint plugin
  - Enable format on save (recommended)
    - add `"editor.formatOnSave": true` to `.vscode/settings.json` file
    - add `"[vue][typescript][javascript]": { "editor.defaultFormatter": "dbaeumer.vscode-eslint" }` to `.vscode/settings.json` file
    - add `"eslint.workingDirectories": [{"mode": "auto"}]` to `.vscode/settings.json` file
    - add `"vue.codeActions.enabled": false` to `.vscode/settings.json` file

### .vscode/settings.json
```json
{
  "editor.formatOnSave": true,
  "[vue]": {
    "editor.defaultFormatter": "dbaeumer.vscode-eslint"
  },
  "[typescript]": {
    "editor.defaultFormatter": "dbaeumer.vscode-eslint"
  },
  "[javascript]": {
    "editor.defaultFormatter": "dbaeumer.vscode-eslint"
  },
  // enable tailwind class autocomplete / tooling outside of just "class" in templates
  "tailwindCSS.experimental.classRegex": [
    [
        "tw`([^`]*)", // tw`...`
    ]
  ],
  "eslint.workingDirectories": [
    {
        "mode": "auto"
    }
  ],
  "vue.codeActions.enabled": false,
}
```
