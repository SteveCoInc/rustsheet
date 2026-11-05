# Changelog: `rustsheet` (public API)

Changes to the facade crate's public API that a caller must act on. The API is pre-v1 (docs/milestones.md,
"API v1 freeze"), so a breaking change can land without a deprecation release; it is listed here.

## Unreleased

- **Breaking:** `LoadOptions` gains `password: Option<Password>` and `max_spin_count: u32` (SPEC-0031 D4, T-4266).
  `LoadOptions` is not `#[non_exhaustive]`, so a struct literal without `..LoadOptions::default()` no longer
  compiles: add it. The defaults (`None`, 1,000,000) keep every other load unchanged.
- New: password-protected xlsx (ECMA-376 Agile and Standard encryption) open with `LoadOptions::password`.
  `LoadError` gains `PasswordRequired`, `WrongPassword` and `Unsupported(UnsupportedKind)`; a legacy `.xls` or another
  OLE file, formerly `LoadError::Format`, is now `LoadError::Unsupported`.
- Python: `Workbook.open`, `Workbook.from_bytes` and `rustsheet.open` take `password=`, and raise the new
  `PasswordRequiredError` and `WrongPasswordError` (subclasses of `LoadError`).
