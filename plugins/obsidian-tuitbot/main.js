"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/main.ts
var main_exports = {};
__export(main_exports, {
  default: () => TuitBotPlugin,
  extractBlock: () => extractBlock,
  resolveHeadingPath: () => resolveHeadingPath
});
module.exports = __toCommonJS(main_exports);
var import_obsidian = require("obsidian");

// src/context.ts
function resolveHeadingPath(headings, cursorLine) {
  if (headings.length === 0) return null;
  const preceding = headings.filter((h) => h.line <= cursorLine);
  if (preceding.length === 0) return null;
  const chain = [];
  let maxLevel = Infinity;
  for (let i = preceding.length - 1; i >= 0; i--) {
    const h = preceding[i];
    if (h.level < maxLevel) {
      chain.unshift(h);
      maxLevel = h.level;
      if (maxLevel === 1) break;
    }
  }
  if (chain.length === 0) return null;
  return chain.map((h) => `${"#".repeat(h.level)} ${h.heading}`).join(" > ");
}
function extractBlock(lines, cursorLine) {
  if (lines[cursorLine].trim() === "") {
    return { text: "", startLine: cursorLine, endLine: cursorLine };
  }
  let start = cursorLine;
  let end = cursorLine;
  while (start > 0 && lines[start - 1].trim() !== "") {
    start--;
  }
  while (end < lines.length - 1 && lines[end + 1].trim() !== "") {
    end++;
  }
  const text = lines.slice(start, end + 1).join("\n");
  return { text, startLine: start, endLine: end };
}

// src/main.ts
var DEFAULT_SETTINGS = {
  serverUrl: "http://127.0.0.1:3001",
  apiTokenPath: "~/.tuitbot/api_token"
};
var TuitBotPlugin = class extends import_obsidian.Plugin {
  settings = DEFAULT_SETTINGS;
  cachedToken = null;
  async onload() {
    await this.loadSettings();
    this.addCommand({
      id: "tuitbot:send-selection",
      name: "Send selection to TuitBot",
      editorCallback: (editor, view) => {
        this.handleSendSelection(editor, view);
      }
    });
    this.addCommand({
      id: "tuitbot:send-block",
      name: "Send current block to TuitBot",
      editorCallback: (editor, view) => {
        this.handleSendBlock(editor, view);
      }
    });
    this.readApiToken().catch(() => {
      new import_obsidian.Notice(
        `TuitBot: API token not found at ${this.resolveTokenPath()}. Start TuitBot once to generate it.`
      );
    });
  }
  onunload() {
    this.cachedToken = null;
  }
  // -- Settings -------------------------------------------------------------
  async loadSettings() {
    const saved = await this.loadData();
    this.settings = { ...DEFAULT_SETTINGS, ...saved ?? {} };
  }
  async saveSettings() {
    await this.saveData(this.settings);
  }
  // -- Token ----------------------------------------------------------------
  resolveTokenPath() {
    const raw = this.settings.apiTokenPath;
    if (raw.startsWith("~/")) {
      const home = typeof process !== "undefined" ? process.env.HOME || process.env.USERPROFILE || "" : "";
      return home + raw.slice(1);
    }
    return raw;
  }
  async readApiToken() {
    if (this.cachedToken) return this.cachedToken;
    const fs = await import("fs/promises");
    const tokenPath = this.resolveTokenPath();
    const token = (await fs.readFile(tokenPath, "utf-8")).trim();
    if (!token) {
      throw new Error(`API token file is empty: ${tokenPath}`);
    }
    this.cachedToken = token;
    return token;
  }
  // -- Payload construction -------------------------------------------------
  buildPayload(file, selectedText, startLine, endLine, headingContext, cache) {
    const title = cache?.frontmatter?.title ?? file.basename;
    const tags = cache?.frontmatter?.tags ?? null;
    return {
      vault_name: this.app.vault.getName(),
      file_path: file.path,
      selected_text: selectedText,
      heading_context: headingContext,
      selection_start_line: startLine,
      selection_end_line: endLine,
      note_title: title,
      frontmatter_tags: Array.isArray(tags) ? tags : null
    };
  }
  // -- Transport ------------------------------------------------------------
  isLocalTransport() {
    try {
      const url = new URL(this.settings.serverUrl);
      return url.hostname === "localhost" || url.hostname === "127.0.0.1";
    } catch {
      return false;
    }
  }
  async sendToTuitBot(payload) {
    const token = await this.readApiToken();
    const url = `${this.settings.serverUrl}/api/vault/send-selection`;
    const response = await (0, import_obsidian.requestUrl)({
      url,
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`
      },
      body: JSON.stringify(payload)
    });
    return response.json;
  }
  // -- Command handlers -----------------------------------------------------
  async handleSendSelection(editor, view) {
    const file = view.file;
    if (!file) {
      new import_obsidian.Notice("TuitBot: No active file.");
      return;
    }
    const selectedText = editor.getSelection().trim();
    if (!selectedText) {
      new import_obsidian.Notice(
        "TuitBot: No text selected. Use 'Send current block' for the block at cursor."
      );
      return;
    }
    const cache = this.app.metadataCache.getFileCache(file);
    const headings = cache?.headings ?? [];
    const fromLine = editor.getCursor("from").line;
    const toLine = editor.getCursor("to").line;
    const headingInfos = headings.map((h) => ({
      level: h.level,
      heading: h.heading,
      line: h.position.start.line
    }));
    const headingContext = resolveHeadingPath(headingInfos, fromLine);
    const payload = this.buildPayload(
      file,
      selectedText,
      fromLine,
      toLine,
      headingContext,
      cache
    );
    await this.send(payload);
  }
  async handleSendBlock(editor, view) {
    const file = view.file;
    if (!file) {
      new import_obsidian.Notice("TuitBot: No active file.");
      return;
    }
    const cursorLine = editor.getCursor().line;
    const lineCount = editor.lineCount();
    const lines = [];
    for (let i = 0; i < lineCount; i++) {
      lines.push(editor.getLine(i));
    }
    const block = extractBlock(lines, cursorLine);
    if (!block.text.trim()) {
      new import_obsidian.Notice("TuitBot: Cursor is on an empty line. Move to a paragraph.");
      return;
    }
    const cache = this.app.metadataCache.getFileCache(file);
    const headings = cache?.headings ?? [];
    const headingInfos = headings.map((h) => ({
      level: h.level,
      heading: h.heading,
      line: h.position.start.line
    }));
    const headingContext = resolveHeadingPath(headingInfos, cursorLine);
    const payload = this.buildPayload(
      file,
      block.text,
      block.startLine,
      block.endLine,
      headingContext,
      cache
    );
    await this.send(payload);
  }
  async send(payload) {
    if (!this.isLocalTransport()) {
      new import_obsidian.Notice(
        "TuitBot: Sending selection to remote server \u2014 text will cross the network.",
        3e3
      );
    }
    try {
      const result = await this.sendToTuitBot(payload);
      new import_obsidian.Notice(
        `TuitBot: Selection received. Open the composer at ${result.composer_url}`
      );
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      if (message.includes("401") || message.includes("Unauthorized")) {
        new import_obsidian.Notice(
          `TuitBot: Authentication failed. Check your API token at ${this.resolveTokenPath()}.`
        );
      } else if (message.includes("ECONNREFUSED") || message.includes("net::ERR")) {
        new import_obsidian.Notice(
          `TuitBot: Server not reachable at ${this.settings.serverUrl}. Is TuitBot running?`
        );
      } else {
        new import_obsidian.Notice(`TuitBot: Failed to send selection \u2014 ${message}`);
      }
    }
  }
};
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  extractBlock,
  resolveHeadingPath
});
