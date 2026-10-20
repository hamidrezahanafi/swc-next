/*---
description: A conditional ending in an arrow cannot become a logical operand.
negative:
  phase: parse
  type: SyntaxError
---*/
a ? b : () => {} || 1;
