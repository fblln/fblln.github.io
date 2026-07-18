---
id: TASK-9
title: Add mutation testing for invariants
status: To Do
assignee: []
created_date: '2026-07-18 10:09'
labels:
  - testing
dependencies: []
priority: high
ordinal: 9000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Wire cargo-mutants (Rust) and StrykerJS (design-system TS). Kill all survivors on logic modules or document equivalent mutants. Add 'just mutants'.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 cargo-mutants runs clean (no uncaught survivors) on logic modules
- [ ] #2 StrykerJS configured for design-system with a mutation score threshold
<!-- AC:END -->
