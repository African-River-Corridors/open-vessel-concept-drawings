// Unit tests for src/lib/loadBasis.ts (issue #668). Run: npm run test:unit
import test from "node:test";
import assert from "node:assert/strict";
import { consumablesInLoad } from "../src/lib/loadBasis.ts";

test("consumablesInLoad reads the consumables of the IS Code loading conditions", () => {
  assert.equal(consumablesInLoad({ loading_conditions: { consumables_t: 34.84 } }), 34.84);
});

test("consumablesInLoad is null when the results carry no consumables", () => {
  assert.equal(consumablesInLoad(null), null);
  assert.equal(consumablesInLoad({}), null);
  assert.equal(consumablesInLoad({ loading_conditions: null }), null);
  assert.equal(consumablesInLoad({ loading_conditions: { consumables_t: 0 } }), null, "none carried: nothing to label");
});

