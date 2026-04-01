---
# csl26-3gye
title: Introduce numbering trait polymorphism
status: completed
type: task
priority: deferred
created_at: 2026-04-01T15:27:01Z
updated_at: 2026-04-01T16:30:00Z
---

# Completed: Numbering Trait Polymorphism

## Context
During the PR review for the sequences/numbering refactor, it was pointed out that the `find_numbering()` method is a large monolithic `match self` statement. It currently enumerates multiple concrete variants with identical match arms.

## Task
Introduce a `HasNumbering` trait to eliminate this repetitive match statement entirely. By making reference types self-registering and polymorphic through this trait, new reference types added in the future will automatically inherit numbering functionality without requiring a manual update to `find_numbering()`.

## Summary of Changes

- Added a private `HasNumbering` trait for numbering-bearing reference structs.
- Replaced the repetitive `find_numbering()` match arms with a single trait-based helper on `InputReference`.
