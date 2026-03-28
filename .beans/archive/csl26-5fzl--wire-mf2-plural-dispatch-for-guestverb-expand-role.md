---
# csl26-5fzl
title: Wire MF2 plural dispatch for guest.verb + expand role_message_id
status: completed
type: task
priority: normal
created_at: 2026-03-28T11:18:01Z
updated_at: 2026-03-28T11:23:42Z
---

Add Guest to ContributorRole enum; extend role_message_id() to route Guest through MF2; add MF2 messages with plural dispatch for role.guest.verb in all 4 locale files; wire guest into engine contributor resolution via reference.guest().

## Summary of Changes

Added Guest to ContributorRole enum; added guest field to Monograph + InputReference.guest() accessor; extended role_message_id() for Guest with MF2 message IDs; added role.guest.{label,label-long,verb} to en-US, de-DE, fr-FR, tr-TR; wired ContributorRole::Guest in engine contributor resolution. Guest.verb now dispatches plural correctly via MF2 count.
