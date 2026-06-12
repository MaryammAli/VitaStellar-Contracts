# Resolved Issues

## Issue #11: escrow_claim() Has Missing Reentrancy Protection (CRITICAL)

**Status**: Fixed

The `escrow_claim()` function was vulnerable to reentrancy attacks because it did not use
the `acquire_lock()` / `release_lock()` mechanism used by `process_payment()` and
`batch_process_payments()`.

### Changes
- Added `Self::acquire_lock(&env)?` at function entry
- Added `Self::release_lock(&env)` on all error paths and normal exit
- Applied CEI (Checks-Effects-Interactions) pattern: state update before external calls
- Added diagnostic event for escrow payment completion
- Added reentrancy guard tests

### Risk
**Critical** — Without this fix, a malicious escrow contract could call back into
`escrow_claim()` before the claim state is updated, potentially allowing double-claiming.

---

## Issue #13: Panic-Based Error Handling in Legacy identity_registry Functions

**Status**: Fixed

Three legacy functions used `panic!()` instead of returning `Result<(), Error>`:
- `register_identity_hash()` — panicked on invalid metadata
- `attest()` — panicked when caller is not a verifier
- `revoke_attestation()` — panicked on missing attestation or non-verifier

### Changes
- `register_identity_hash()` now returns `Result<(), Error>` with `Error::InvalidInput`
- `attest()` now returns `Result<(), Error>` with `Error::NotVerifier`
- `revoke_attestation()` now returns `Result<(), Error>` with `Error::NotVerifier` or `Error::AttestationNotFound`
- Removed `#[allow(clippy::panic)]` lint suppression (no longer needed)
- Updated tests to use `try_*` methods

### Breaking Change
These functions previously returned `()` and panicked on error. They now return
`Result<(), Error>`. Callers must handle the `Result`.

---

## Issue #14: Inconsistent Health Check Implementations Across Contracts

**Status**: Fixed

Contracts implemented `health_check()` inconsistently:
- `medical_records` returned `(Symbol, u32, u64)` — status, version, timestamp
- `patient_consent_management` returned `bool` — just whether initialized
- `identity_registry` returned `(Symbol, u32, u64)` — but didn't check paused state

### Changes
- Standardized return type to `(Symbol, u32, u64)` across all contracts
- Added paused state check to all health check implementations
- Status values: `"OK"`, `"PAUSED"`, `"NOT_INIT"`, `"DEGRADED"`
- Added health check to `medical_record_hash_registry`
- Added health check event publishing to `patient_consent_management`

### Breaking Change
`patient_consent_management::health_check()` previously returned `bool`. It now returns
`(Symbol, u32, u64)`.

---

## Issue #7: No Input Validation in Batch Consent Granting

**Status**: Fixed

The `batch_grant_consent()` function accepted any size list without validation:
- Empty lists were silently accepted
- Oversized batches could cause gas exhaustion
- No explicit size limits

### Changes
- Added `MAX_BATCH_SIZE = 50` constant
- Reject empty grantees list with `Error::InvalidInput`
- Reject batches exceeding `MAX_BATCH_SIZE` with `Error::BatchTooLarge`
- Added `BatchTooLarge` (471) and `InvalidInput` (472) error variants

---

## Issue #5: Missing Pause/Unpause in Multiple Contracts

**Status**: Fixed (medical_record_hash_registry)

`medical_record_hash_registry` lacked any pause/unpause mechanism. If a security
vulnerability was found, there was no way for admins to halt operations.

### Changes
- Added `Paused` storage key
- Added `pause()` and `unpause()` functions (admin-only)
- Added `is_paused()` getter
- `store_record()` now rejects new records when paused
- Added standardized `health_check()` returning pause state
- Added `require_admin()` helper for access control
- Added pause/unpause event publishing
