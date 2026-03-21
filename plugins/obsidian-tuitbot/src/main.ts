import {
  type CachedMetadata,
  type Editor,
  type MarkdownView,
  Notice,
  Plugin,
  type TFile,
  requestUrl,
} from "obsidian";

import {
  type GhostwriterPayload,
  type HeadingInfo,
  type SendSelectionResponse,
  resolveHeadingPath,
  extractBlock,
} from "./context.js";

export type { GhostwriterPayload, SendSelectionResponse };
export { resolveHeadingPath, extractBlock };

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

export interface TuitBotSettings {
  serverUrl: string;
  apiTokenPath: string;
}

const DEFAULT_SETTINGS: TuitBotSettings = {
  serverUrl: "http://127.0.0.1:3001",
  apiTokenPath: "~/.tuitbot/api_token",
};

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

export default class TuitBotPlugin extends Plugin {
  settings: TuitBotSettings = DEFAULT_SETTINGS;
  private cachedToken: string | null = null;

  override async onload(): Promise<void> {
    await this.loadSettings();

    this.addCommand({
      id: "tuitbot:send-selection",
      name: "Send selection to TuitBot",
      editorCallback: (editor: Editor, view: MarkdownView) => {
        this.handleSendSelection(editor, view);
      },
    });

    this.addCommand({
      id: "tuitbot:send-block",
      name: "Send current block to TuitBot",
      editorCallback: (editor: Editor, view: MarkdownView) => {
        this.handleSendBlock(editor, view);
      },
    });

    // Attempt to read the API token early so we can warn the user.
    this.readApiToken().catch(() => {
      new Notice(
        `TuitBot: API token not found at ${this.resolveTokenPath()}. ` +
          "Start TuitBot once to generate it.",
      );
    });
  }

  override onunload(): void {
    this.cachedToken = null;
  }

  // -- Settings -------------------------------------------------------------

  async loadSettings(): Promise<void> {
    const saved = (await this.loadData()) as Partial<TuitBotSettings> | null;
    this.settings = { ...DEFAULT_SETTINGS, ...(saved ?? {}) };
  }

  async saveSettings(): Promise<void> {
    await this.saveData(this.settings);
  }

  // -- Token ----------------------------------------------------------------

  private resolveTokenPath(): string {
    const raw = this.settings.apiTokenPath;
    if (raw.startsWith("~/")) {
      const home =
        typeof process !== "undefined"
          ? process.env.HOME || process.env.USERPROFILE || ""
          : "";
      return home + raw.slice(1);
    }
    return raw;
  }

  async readApiToken(): Promise<string> {
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

  buildPayload(
    file: TFile,
    selectedText: string,
    startLine: number,
    endLine: number,
    headingContext: string | null,
    cache: CachedMetadata | null,
  ): GhostwriterPayload {
    const title = cache?.frontmatter?.title ?? file.basename;
    const tags: string[] | null = cache?.frontmatter?.tags ?? null;

    return {
      vault_name: this.app.vault.getName(),
      file_path: file.path,
      selected_text: selectedText,
      heading_context: headingContext,
      selection_start_line: startLine,
      selection_end_line: endLine,
      note_title: title,
      frontmatter_tags: Array.isArray(tags) ? tags : null,
    };
  }

  // -- Transport ------------------------------------------------------------

  private isLocalTransport(): boolean {
    try {
      const url = new URL(this.settings.serverUrl);
      return url.hostname === "localhost" || url.hostname === "127.0.0.1";
    } catch {
      return false;
    }
  }

  async sendToTuitBot(
    payload: GhostwriterPayload,
  ): Promise<SendSelectionResponse> {
    const token = await this.readApiToken();
    const url = `${this.settings.serverUrl}/api/vault/send-selection`;

    const response = await requestUrl({
      url,
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(payload),
    });

    return response.json as SendSelectionResponse;
  }

  // -- Command handlers -----------------------------------------------------

  private async handleSendSelection(
    editor: Editor,
    view: MarkdownView,
  ): Promise<void> {
    const file = view.file;
    if (!file) {
      new Notice("TuitBot: No active file.");
      return;
    }

    const selectedText = editor.getSelection().trim();
    if (!selectedText) {
      new Notice(
        "TuitBot: No text selected. Use 'Send current block' for the block at cursor.",
      );
      return;
    }

    const cache = this.app.metadataCache.getFileCache(file);
    const headings = cache?.headings ?? [];
    const fromLine = editor.getCursor("from").line;
    const toLine = editor.getCursor("to").line;

    const headingInfos: HeadingInfo[] = headings.map((h) => ({
      level: h.level,
      heading: h.heading,
      line: h.position.start.line,
    }));
    const headingContext = resolveHeadingPath(headingInfos, fromLine);

    const payload = this.buildPayload(
      file,
      selectedText,
      fromLine,
      toLine,
      headingContext,
      cache,
    );

    await this.send(payload);
  }

  private async handleSendBlock(
    editor: Editor,
    view: MarkdownView,
  ): Promise<void> {
    const file = view.file;
    if (!file) {
      new Notice("TuitBot: No active file.");
      return;
    }

    const cursorLine = editor.getCursor().line;
    const lineCount = editor.lineCount();
    const lines: string[] = [];
    for (let i = 0; i < lineCount; i++) {
      lines.push(editor.getLine(i));
    }

    const block = extractBlock(lines, cursorLine);
    if (!block.text.trim()) {
      new Notice("TuitBot: Cursor is on an empty line. Move to a paragraph.");
      return;
    }

    const cache = this.app.metadataCache.getFileCache(file);
    const headings = cache?.headings ?? [];
    const headingInfos: HeadingInfo[] = headings.map((h) => ({
      level: h.level,
      heading: h.heading,
      line: h.position.start.line,
    }));
    const headingContext = resolveHeadingPath(headingInfos, cursorLine);

    const payload = this.buildPayload(
      file,
      block.text,
      block.startLine,
      block.endLine,
      headingContext,
      cache,
    );

    await this.send(payload);
  }

  private async send(payload: GhostwriterPayload): Promise<void> {
    if (!this.isLocalTransport()) {
      new Notice(
        "TuitBot: Sending selection to remote server \u2014 text will cross the network.",
        3000,
      );
    }
    try {
      const result = await this.sendToTuitBot(payload);
      new Notice(
        `TuitBot: Selection received. Open the composer at ${result.composer_url}`,
      );
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);

      if (message.includes("401") || message.includes("Unauthorized")) {
        new Notice(
          `TuitBot: Authentication failed. Check your API token at ${this.resolveTokenPath()}.`,
        );
      } else if (
        message.includes("ECONNREFUSED") ||
        message.includes("net::ERR")
      ) {
        new Notice(
          `TuitBot: Server not reachable at ${this.settings.serverUrl}. Is TuitBot running?`,
        );
      } else {
        new Notice(`TuitBot: Failed to send selection — ${message}`);
      }
    }
  }
}
