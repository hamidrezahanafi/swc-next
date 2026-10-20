/*---
description: A sequence ending in an arrow cannot become a binary operand.
negative:
  phase: parse
  type: SyntaxError
---*/
a, () => {} + 1;
