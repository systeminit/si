# app/web

## Project setup
```
npm install
```

### Compiles and hot-reloads for development
```
npm run start
```

### Compiles and minifies for production
```
npm run build
```

### Lint check (no fix)
```
npm run lint
```

### Type check (no fix)
```
npm run build:check
```

### Code formatting check (no fix)
```
npm run fmt:check
```

### Format code (WILL FIX)
```
npm run fmt
```

# Vue 3 + Typescript + Vite

This template should help get you started developing with Vue 3 and Typescript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## IDE Setup Instructions
### [VSCode](https://code.visualstudio.com/) (preferred)
  - Install [Volar](https://marketplace.visualstudio.com/items?itemName=johnsoncodehk.volar) plugin
    - and disable Vetur if installed
  - Enable TS "takeover mode" (see [here](https://github.com/johnsoncodehk/volar/discussions/471))
    - run "Extensions: Show built-in extensions" from command pallete
    - search for "@builtin typescript" in extensions panel
    - find "Typescript + Javascript Language Features", click gear and "Disable (Workspace)"
  - Use workspace's typescript version
    - run "Volar: Select Typescript Version" from command pallete
    - select "Use workspace version"
  - Install [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) plugin
  - Enable format on save (recommended)
    - add `"editor.formatOnSave": true` to `.vscode/settings.json` file
    - add `"[vue][typescript][javascript]": { "editor.defaultFormatter": "esbenp.prettier-vscode" }` to `.vscode/settings.json` file
