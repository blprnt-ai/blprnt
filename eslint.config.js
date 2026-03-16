// eslint.config.js

import tsParser from "@typescript-eslint/parser";
import eslintPluginReact from "eslint-plugin-react";
import reactRefresh from "eslint-plugin-react-refresh";
import boundaries from "eslint-plugin-boundaries";
import typescriptEslintPlugin from "@typescript-eslint/eslint-plugin";
import importPlugin from "eslint-plugin-import";
import sonarjs from "eslint-plugin-sonarjs";

const ignores = [
  "dist/**",
  "build/**",
  "target/**",
  "./src/bindings.ts",
  "./eslint.config.js",
  "./vite.config.ts",
];

/** @type {import("eslint").Linter.Config[]} */
export default [
  {
    ignores,
  },
  {
    ...reactRefresh.configs.vite,
    ...sonarjs.configs.recommended,
    files: ["src/**/*.{ts,tsx,js,jsx}"],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaFeatures: { jsx: true },
        ecmaVersion: "latest",
        project: "./tsconfig.json",
        sourceType: "module",
      },
    },
    plugins: {
      "@typescript-eslint": typescriptEslintPlugin,
      react: eslintPluginReact,
      boundaries: boundaries,
      import: importPlugin,
      sonarjs: sonarjs,
    },
    rules: {
      "react/jsx-sort-props": [
        "error",
        {
          reservedFirst: true,
          callbacksLast: true,
          ignoreCase: true,
          multiline: "last",
          shorthandFirst: true,
        },
      ],
      "boundaries/extrnal": [0],
      "boundaries/element-types": [
        "error",
        {
          default: "disallow",
          rules: [
            {
              from: ["api"],
              allow: ["bindings", "api", "events"],
            },
            {
              from: ["*"],
              allow: ["bindings"],
              importKind: "type",
            },
            {
              from: ["api"],
              allow: ["stores"],
              importKind: "type",
            },
            {
              from: ["utils"],
              allow: ["utils", "api"],
            },
            {
              from: ["events"],
              allow: ["events", "utils", "components", "stores"],
            },
            {
              from: ["stores"],
              allow: ["api", "stores", "utils", "components", "events"],
            },
            {
              from: ["hooks"],
              allow: ["hooks", "stores", "context", "utils"],
            },
            {
              from: ["context"],
              allow: ["components", "stores", "hooks", "context", "utils"],
            },
            {
              from: ["components"],
              allow: [
                "components",
                "stores",
                "hooks",
                "context",
                "utils",
                "events",
              ],
            },
            {
              from: ["*"],
              allow: ["*"],
              importKind: "type",
            },
          ],
        },
      ],
    },
    settings: {
      "import/resolver": {
        typescript: {
          alwaysTryTypes: true,
        },
      },
      react: { version: "detect" },
      "boundaries/elements": [
        { type: "api", pattern: "src/lib/api/**/*", mode: "file" },
        { type: "bindings", pattern: "src/bindings.ts", mode: "file" },
        { type: "components", pattern: "src/components/**/*", mode: "file" },
        { type: "context", pattern: "src/context/**/*", mode: "file" },
        { type: "events", pattern: "src/lib/events/**/*", mode: "file" },
        { type: "hooks", pattern: "src/hooks/**/*", mode: "file" },
        { type: "stores", pattern: "src/lib/stores/**/*", mode: "file" },
        { type: "utils", pattern: "src/lib/utils/**/*", mode: "file" },
      ],
      "boundaries/include": ["src/**/*"],
    },
  },
];
