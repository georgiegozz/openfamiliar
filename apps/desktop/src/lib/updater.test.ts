import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { UPDATER_UNAVAILABLE_MESSAGE, mapUpdaterError } from "./updater";

describe("mapUpdaterError", () => {
  it("treats a missing endpoint or public key as an inert channel", () => {
    assert.equal(
      mapUpdaterError("Updater does not have any endpoints set."),
      UPDATER_UNAVAILABLE_MESSAGE,
    );
    assert.equal(
      mapUpdaterError("Could not decode public key"),
      UPDATER_UNAVAILABLE_MESSAGE,
    );
  });

  it("keeps other errors visible without claiming a signed update exists", () => {
    assert.equal(
      mapUpdaterError("network failure"),
      "No se pudo buscar actualizaciones: network failure",
    );
  });
});
