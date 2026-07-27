---
name: Bug report
about: Something in zxcvbn-wasm isn't working as expected
title: ''
labels: bug
assignees: ''
---

**Describe the bug**
A clear and concise description of what's wrong.

**To reproduce**
```ts
import init, { zxcvbn } from "@lycheeorg/zxcvbn-wasm";

await init();
const result = zxcvbn("...", []);
```
What did you expect `result` to look like, and what did you actually get?

**Environment**
- `@lycheeorg/zxcvbn-wasm` version:
- Runtime: (browser + version / Node version)
- Bundler (if any):

**Additional context**
Add any other context about the problem here.
