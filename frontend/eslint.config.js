// eslint.config.js

import tsParser from "@typescript-eslint/parser";
import eslintPluginReact from "eslint-plugin-react";
import reactRefresh from "eslint-plugin-react-refresh";
import typescriptEslintPlugin from "@typescript-eslint/eslint-plugin";
import importPlugin from "eslint-plugin-import";

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
      import: importPlugin,
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
    },
    settings: {
      "import/resolver": {
        typescript: {
          alwaysTryTypes: true,
        },
      },
      react: { version: "detect" },
    },
  },
];
