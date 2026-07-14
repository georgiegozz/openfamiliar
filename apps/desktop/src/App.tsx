import { MascotWindow } from "./windows/MascotWindow";
import { SettingsWindow } from "./windows/SettingsWindow";

export function App() {
  const windowKind = new URLSearchParams(window.location.search).get("window");
  return windowKind === "settings" ? <SettingsWindow /> : <MascotWindow />;
}
