---
id: TASK-1
title: Adopt self-hosted type system (Archivo + JetBrains Mono)
status: Done
assignee: []
created_date: '2026-07-18 09:37'
updated_date: '2026-07-18 09:49'
labels:
  - design
  - typography
dependencies: []
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace Arial/Helvetica display and Courier New mono with self-hosted, Latin-subset woff2 fonts. Archivo 800 for display, JetBrains Mono 500 for labels. Add @font-face to styles.css, copy woff2 into assets/fonts, wire Trunk copy-dir, add font vars.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Display uses Archivo, mono uses JetBrains Mono site-wide
- [ ] #2 Fonts self-hosted as subset woff2 (no external CDN)
- [ ] #3 Total added font weight under ~40KB
<!-- AC:END -->
