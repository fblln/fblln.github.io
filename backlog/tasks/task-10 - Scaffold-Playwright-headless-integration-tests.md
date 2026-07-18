---
id: TASK-10
title: Scaffold Playwright headless integration tests
status: To Do
assignee: []
created_date: '2026-07-18 10:09'
labels:
  - testing
dependencies: []
priority: high
ordinal: 10000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Real browser tests against served dist/ or trunk serve: boot/mount, keyboard model (J/K/Enter/S///Esc), project filtering + search, case-study and system panels, runtime-facts readout, mobile nav (<=680px) and <=900px breakpoints. Also cover design-system via Storybook/mounted page. Add 'just e2e' and run headless in CI.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Playwright suite drives the real WASM app and asserts observed behavior
- [ ] #2 Responsive breakpoints tested as real viewports
- [ ] #3 Suite runs headless in CI
<!-- AC:END -->
