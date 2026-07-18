---
id: TASK-7
title: Add justfile as the command runner
status: To Do
assignee: []
created_date: '2026-07-18 10:09'
labels:
  - tooling
dependencies: []
priority: high
ordinal: 7000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Introduce a justfile so all workflows run through 'just' (build, serve, test, coverage, mutants, e2e, fmt, ds-build, ds-sync-check). Single entry point for site + design-system + blog tool. Update AGENTS.md and CI to invoke just recipes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 justfile at repo root with recipes for build, serve, test, coverage, mutation, e2e, fmt
- [ ] #2 Recipes cover both the site and design-system/
- [ ] #3 AGENTS.md and CI reference just recipes
<!-- AC:END -->
