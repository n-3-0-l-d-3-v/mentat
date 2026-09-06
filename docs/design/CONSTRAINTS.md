# Constraints — THE MACHINE

## Primary constraint

The machine does not use a conventional instruction-pointer-driven execution model. Instructions execute when their operand dependencies become ready, not in program-counter order.

## What it forces

Fixed-size instruction encoding, bounded physical registers, explicit operand dependency tracking, deterministic scheduling, and deterministic replay.

## Research question

How much of conventional instruction sequencing is actually necessary for useful general-purpose computation?

## What is explicitly out of scope

See the root [SCOPE.md](../../SCOPE.md) for the CORE / EXTENSION / EXPERIMENT
classification that applies to this repo.
