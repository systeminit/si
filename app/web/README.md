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

- Install [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint) plugin
- Install [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) plugin

### .vscode/settings.json

```json
{
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": "always"
  },
  "eslint.workingDirectories": [{ "mode": "auto" }],
  "eslint.format.enable": true,
  "eslint.lintTask.enable": true,
  "typescript.preferences.preferTypeOnlyAutoImports": false,
  "eslint.run": "onSave",
  "eslint.lintTask.options": "--ext .ts,.js,.cjs,.vue",
  "[vue]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[javascript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  // enable tailwind class autocomplete / tooling outside of just "class" in templates
  "tailwindCSS.experimental.classRegex": [
    [
      "tw`([^`]*)" // tw`...`
    ]
  ]
}
```
