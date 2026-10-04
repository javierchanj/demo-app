# Constitution

- Every task ships a test, written before the implementation.
- Every Gherkin scenario is implemented in this order: write its step
  definitions wired to real behavior and see them fail (BDD red), then a TDD
  inner loop (failing unit test = TDD red, minimal implementation = TDD green),
  then re-run Cucumber to see the scenario pass (BDD green).
- No breaking changes to existing endpoints.
- Spec changes only through a human-reviewed commit.
- One task per prompt; never run past a failing test.
- Answer in the repo's existing stack and conventions (Loco, SeaORM, Axum).