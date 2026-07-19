+++
title = "Systems That Survive Contact With Production"
date = "2026-07-17"
description = "Why the smallest solid thing beats the broad demo — and how to make evidence queryable instead of theatrical."
tags = ["Systems", "Reliability", "Rust"]
+++

Most architecture diagrams are fiction. They describe a system that would exist
if traffic were uniform, dependencies never failed, and every engineer read the
same wiki page. Production disagrees.

This is a placeholder post that exists to exercise the article pipeline —
headings, code, quotes, tables, and attachments all render from a single
Markdown file.

## Measure the real system

Production distributions, traces, and representative data beat architectural
theatre. Before optimizing, look at what actually happens:

```rust
fn p99(latencies: &mut [u64]) -> u64 {
    latencies.sort_unstable();
    let idx = (latencies.len() as f64 * 0.99) as usize;
    latencies[idx.min(latencies.len() - 1)]
}
```

A single `p99` on real data will reshape more decisions than a week of
whiteboarding. Numbers are cheap; opinions are expensive.

## Make evidence queryable

Graphs, typed contracts, and deterministic tests turn understanding into
infrastructure. The goal is to make the system *answer questions about itself*.

> A narrow production-grade loop creates more leverage than a broad demo that
> cannot be trusted.

### A quick comparison

| Approach        | Trust | Leverage |
| --------------- | ----- | -------- |
| Broad demo      | Low   | Low      |
| Narrow, solid   | High  | High     |

### Attachments

Images and files live in `assets/articles/` and are linked with a normal
Markdown link, so they flow through the existing asset pipeline.

## Ship the smallest solid thing

Then question whether the next thing needs to exist at all.
