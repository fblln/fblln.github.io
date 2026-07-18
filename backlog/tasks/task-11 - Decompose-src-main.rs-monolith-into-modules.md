---
id: TASK-11
title: Decompose src/main.rs monolith into modules
status: To Do
assignee: []
created_date: '2026-07-18 10:09'
labels:
  - refactor
dependencies: []
priority: medium
ordinal: 11000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Split the ~500-line single-file app into cohesive modules: projects data, keyboard handling, runtime diagnostics, and each view component. One reason to change per file. Preserve behavior (guarded by the test suite) and keep wide why/what block comments.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 main.rs reduced to app wiring; logic lives in focused modules
- [ ] #2 All existing tests still pass; no behavior change
<!-- AC:END -->
