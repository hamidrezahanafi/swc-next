/*---
description: A regular expression starts a new statement after an assignment ending in bare yield.
---*/
function* g() {
  x = yield
  /foo/.test(s);
}
