import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
export const familiarV1Schema = JSON.parse(
  readFileSync(join(__dirname, "familiar-v1.json"), "utf8"),
);

export function validateManifestShape(manifest) {
  const errors = [];
  for (const key of ["id", "name", "version", "engine", "author", "license", "personality", "states"]) {
    if (manifest == null || manifest[key] == null || manifest[key] === "") {
      errors.push(`missing ${key}`);
    }
  }
  if (manifest?.states) {
    for (const s of ["idle", "thinking", "working", "approval", "success", "error"]) {
      if (!manifest.states[s]) errors.push(`missing state ${s}`);
    }
  }
  return { ok: errors.length === 0, errors };
}