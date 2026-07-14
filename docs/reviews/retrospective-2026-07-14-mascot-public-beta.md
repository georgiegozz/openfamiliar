# Retrospectiva funcional y técnica — mascota y publicación pública

Fecha: 2026-07-14
Alcance: Perrito Tech, interacción de ventana, personalización de color y
preparación de OpenFamiliar como primer repositorio público.

## Resumen ejecutivo

El MVP demostró su objetivo principal: una pregunta independiente llegó a una
instalación autenticada de Codex CLI y devolvió una respuesta sin historial. El
problema visible no está en Codex sino en la experiencia de la mascota: el idle
programa acciones aleatorias, el spritesheet cambia rasgos entre frames, el
tamaño predeterminado es mayor al deseado y la ventana transparente conserva un
área interactiva de 560×360 px aun cuando la tarjeta está cerrada.

Decisión: **GO condicionado** para implementar el rediseño y publicar el código
como `v0.1.0-beta`. El instalador no debe anunciarse todavía como release estable
o firmado hasta completar la prueba manual en usuario limpio y la firma de
código.

## Evidencia funcional

- Quick Ask encontró Codex CLI, confirmó una sesión ChatGPT y respondió una
  consulta real.
- La interacción es one-shot y no creó historial de conversación.
- El idle actual elige una acción aleatoria cada 4–12 segundos y puede entrar en
  sleeping después de cinco minutos. Esto contradice la preferencia del usuario
  por una mascota inmóvil mientras no haya un evento.
- El frame usado por las miradas cambia ojos y pose de forma inconsistente; el
  defecto reportado del ojo derecho proviene del asset/animación, no del render
  de Windows.
- El frame de 96 px a escala predeterminada 2× produce una mascota de 192 px.
  Reducir el frame a 64 px y conservar 2× produce 128 px: exactamente un tercio
  menos por dimensión y mantiene pixel-perfect integer scaling.
- La ventana transparente cerrada mide 560×360 px. Aunque la zona visible sea
  menor, esa superficie puede interferir con aplicaciones situadas detrás.

## Hallazgos técnicos previos

| Hallazgo                                                                     | Impacto                              | Tratamiento                                                                     |
| ---------------------------------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------- |
| Detector de Codex sólo reconocía `codex.exe` directo en PATH                 | npm quedaba sin detectar             | Integrar el cambio pendiente y sus pruebas                                      |
| `rustfmt` y `clippy` no están instalados localmente                          | Validación incompleta                | Instalar componentes y ejecutar ambos gates                                     |
| Idle y sleeping dependen de timers aleatorios                                | Movimiento sin intención del usuario | Eliminar scheduling ambiental del runtime estable                               |
| Ventana cerrada conserva tamaño expandido                                    | Interfiere con otras aplicaciones    | Compactar a la huella real y expandir sólo para Quick Ask                       |
| El arrastre ocupa todo el sprite                                             | Capturas accidentales                | Limitarlo a un handle pequeño y explícito                                       |
| `SECURITY.md` y `CONTRIBUTING.md` describen funciones futuras como presentes | Expectativas públicas incorrectas    | Alinear documentos con el MVP real                                              |
| `THIRD_PARTY_NOTICES.md` deja el inventario de dependencias pendiente        | Release legal incompleto             | Generar inventario reproducible desde lockfiles                                 |
| Versión raíz `0.1.0-private`                                                 | Mensaje confuso para un repo público | Usar `0.1.0`; conservar `private: true` para impedir publicación npm accidental |
| Instaladores sin firma y sin clean-user smoke final                          | Riesgo SmartScreen/UX                | Mantener release como beta y documentar gate manual                             |

## Rediseño aprobado

- Personaje original basado únicamente en las referencias fotográficas
  proporcionadas por el operador: pelaje atigrado oscuro, franja y hocico
  blancos, orejas caídas, nariz negra y ojos cafés simétricos.
- Conservar la identidad de Perrito Tech mediante pelaje atigrado, franja blanca,
  orejas caídas y expresión feliz; mantenerlo natural y sin collar.
- Spritesheet RGBA 4×4 con celdas de 64×64 px.
- Idle de un solo frame. Animación sólo ante listening, thinking, answering,
  success, error o dragging.
- Variantes deterministas sólo para pequeños props de estado; cuello, pelaje,
  ojos y marcas no cambian. La fuente canónica sigue siendo un único asset.

## Repositorios evaluados

### Referencia aceptada: BongoCat

[`ayangweb/BongoCat`](https://github.com/ayangweb/BongoCat) usa Tauri, publica
su código bajo MIT y soporta importación de modelos personalizados. Se toma sólo
como validación del patrón: separar runtime y modelo visual. No se copiarán
código, formatos ni assets.

### Referencia aceptada: Petdex

[`crafter-station/petdex`](https://github.com/crafter-station/petdex) valida el
patrón de paquete autocontenido `manifest + spritesheet` y separa la licencia del
código de la licencia de cada asset. OpenFamiliar ya tiene ese patrón mediante
`familiar.json`, por lo que no necesita adoptar su formato.

### Referencia rechazada para reutilización

[`rullerzhou-afk/clawd-on-desk`](https://github.com/rullerzhou-afk/clawd-on-desk)
es útil como comparación de producto, pero su código es AGPL-3.0 y varios assets
son de terceros o están reservados. Se mantiene como referencia conceptual; no
se copiará ninguna implementación o imagen.

## Gate de publicación pública

GitHub recomienda README, LICENSE, CONTRIBUTING y CODE_OF_CONDUCT para el perfil
de comunidad, además de SECURITY, secret scanning, push protection y code
scanning. El repositorio ya contiene los documentos base, CI y gitleaks. Antes
de cambiar la visibilidad deben quedar cerrados estos gates:

1. CI JavaScript/Rust en verde y pack validator sin warnings.
2. Inventario de licencias generado desde lockfiles.
3. Ningún secreto, path personal, log de diagnóstico o instalador trackeado.
4. MSI/NSIS generados y checksums registrados para el release candidato.
5. Smoke manual en Windows 11 con DPI 100/125/150 %, instalación y
   desinstalación.
6. Activar Dependabot, secret scanning, push protection y branch protection en
   GitHub.
7. Mantener el primer release como pre-release mientras el binario esté sin
   firma.

Referencias:

- [GitHub community profile](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/about-community-profiles-for-public-repositories)
- [GitHub repository security](https://docs.github.com/en/repositories/creating-and-managing-repositories/best-practices-for-repositories)

## Criterio de cierre

La implementación es aceptable cuando el idle permanece visualmente idéntico
durante al menos 60 segundos, la mascota predeterminada mide 128×128 px, la
ventana compacta no excede la huella del sprite más 24 px, el handle de arrastre
no excede 48×24 px, las pupilas se mantienen alineadas en todos los frames, las
variantes se regeneran con un comando reproducible y el release set completo
queda en verde.

## Cierre de implementación

Resultado del 2026-07-14: el código cumple el gate automatizado y es viable como
`v0.1.0-beta` de fuente. No cumple todavía el gate de binario estable firmado.

| Evidencia                              | Resultado                                                                |
| -------------------------------------- | ------------------------------------------------------------------------ |
| Arte original, feliz y sin collar      | Sheet 4×4 normalizado y validador automático: aprobado                   |
| Tamaño predeterminado                  | Sprite 128×128; ventana compacta 152×152: aprobado                       |
| Idle sin timers                        | Timer ambiental eliminado; prueba determinista de 60 s: aprobada         |
| Observación visual desatendida de 60 s | `idle` estable; captura transparente tardía requirió reintento inmediato |
| Handle de arrastre                     | 48×24, separado del click principal: aprobado por código y captura       |
| Variantes                              | 57 píxeles, todos dentro del prop del frame de arrastre                  |
| JS/TS                                  | formato, typecheck, tests, build, assets y packs: aprobado               |
| Rust                                   | `fmt`, `clippy -D warnings` y 68 pruebas: aprobado                       |
| Inventario legal                       | reproducible; SHA-256 `39040D3D…2568`; sin rutas personales              |
| Build Tauri                            | MSI y NSIS generados: aprobado                                           |
| Firma Authenticode                     | `NotSigned`: bloquea afirmar instalador estable                          |
| Clean-user/DPI/install/uninstall       | pendiente; requiere usuario limpio o VM                                  |

Artefactos del candidato local:

- EXE: `7CEFA4FEB177F8C6C3F2F3A9441A9ACC916DB41CC6883D9FFAB2BABA260D2A42`
- MSI: `B06E22B1467AD618F6D0436D2E87E763FCE12D75F6BA50369C598EC9D6417A47`
- NSIS: `CDF933D8064158D4436A75AF97572D3F73DA43283175F2F94C2E6BDB4F15413F`

La deuda previa resuelta incluye el guardado idempotente cuando la entrada de
inicio automático ya está ausente, creación del directorio de preferencias,
preservación del estado en memoria ante fallo de escritura, detección de Codex
instalado por npm, deserialización camelCase de packs, validación de variantes,
instalación de `rustfmt`/`clippy`, limpieza de un warning de prueba, documentación
de privacidad y seguridad alineada al MVP e inventario de licencias desde
lockfiles. El PNG legado `apps/desktop/public/perrito.png` sigue trackeado y sin
referencias; no se eliminó porque borrar un archivo existente requiere
autorización explícita.

La estrategia y el gate reproducible de Authenticode quedan documentados en
`docs/guides/windows-code-signing.md`; `pnpm release:verify` confirma que el
candidato actual sigue `NotSigned` y el modo `-RequireSignature` lo rechaza.
