import js from "@eslint/js";
import globals from "globals";
import json from "@eslint/json";
import markdown from "@eslint/markdown";
import css from "@eslint/css";
import { defineConfig } from "eslint/config";

export default defineConfig([
  js.configs.recommended,  
  { files: ["**/*.{js,mjs,cjs}"], languageOptions: { globals: globals.browser } },  
  json.configs.recommended,  
  markdown.configs.recommended,  
  css.configs.recommended,  
]);
