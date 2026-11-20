export const UPDATER_UNAVAILABLE_MESSAGE =
  "Las actualizaciones firmadas no están habilitadas todavía. Los instaladores actuales no se actualizan por este canal.";

export const UPDATER_UP_TO_DATE_MESSAGE =
  "No hay una actualización firmada disponible.";

export function mapUpdaterError(message: string): string {
  const text = message.toLowerCase();
  if (
    text.includes("endpoints") ||
    text.includes("public key") ||
    text.includes("pubkey") ||
    text.includes("signature")
  ) {
    return UPDATER_UNAVAILABLE_MESSAGE;
  }
  return `No se pudo buscar actualizaciones: ${message}`;
}
