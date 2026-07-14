import * as vscode from "vscode";

async function postToCore(path: string, body: unknown): Promise<void> {
  const cfg = vscode.workspace.getConfiguration("openfamiliar");
  const base = cfg.get<string>("coreEndpoint") ?? "http://127.0.0.1:17321";
  // Keys never leave the Core; extension only sends editor context.
  try {
    await fetch(`${base}${path}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
  } catch (e) {
    vscode.window.showWarningMessage(
      `OpenFamiliar Core no responde en ${base}. ¿Está abierta la app de escritorio?`,
    );
  }
}

export function activate(context: vscode.ExtensionContext) {
  const reg = (cmd: string, fn: () => Promise<void>) =>
    context.subscriptions.push(vscode.commands.registerCommand(cmd, fn));

  reg("openfamiliar.askSelection", async () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) return;
    const text = editor.document.getText(editor.selection);
    await postToCore("/v1/context/selection", {
      path: editor.document.uri.fsPath,
      text,
    });
    vscode.window.showInformationMessage(
      "Selección enviada a OpenFamiliar (local).",
    );
  });

  reg("openfamiliar.explainActive", async () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) return;
    await postToCore("/v1/context/active-file", {
      path: editor.document.uri.fsPath,
    });
  });

  reg("openfamiliar.reviewDiff", async () => {
    await postToCore("/v1/context/git-diff", {
      workspace: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
    });
  });

  reg("openfamiliar.addToContext", async () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) return;
    await postToCore("/v1/context/add-file", {
      path: editor.document.uri.fsPath,
    });
  });

  reg("openfamiliar.startAgent", async () => {
    const task = await vscode.window.showInputBox({
      prompt: "Tarea para el agente",
    });
    if (!task) return;
    await postToCore("/v1/agent/start", { task });
  });

  reg("openfamiliar.showSession", async () => {
    await postToCore("/v1/session/show", {});
  });
}

export function deactivate() {}
