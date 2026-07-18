---
id: TASK-8
title: Enforce 100% unit-test coverage gate
status: To Do
assignee: []
created_date: '2026-07-18 10:09'
labels:
  - testing
dependencies: []
priority: high
ordinal: 8000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Wire cargo-llvm-cov for the Rust site + blog tool and vitest --coverage for the design system. Fail CI below 100% line+branch on non-main/non-view logic. Add 'just coverage'.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 cargo-llvm-cov produces a coverage report locally and in CI
- [ ] #2 vitest coverage threshold set to 100 for design-system
- [ ] #3 CI fails when coverage drops below threshold
<!-- AC:END -->
