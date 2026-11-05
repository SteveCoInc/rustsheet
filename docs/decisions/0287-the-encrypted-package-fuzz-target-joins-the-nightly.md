# 0287 — The encrypted_package fuzz target joins the nightly fuzz job

- **Status:** accepted (2026-11-05, T-4265)
- **Context:** SPEC-0031 D2 and D3 have the importer decrypt ECMA-376 encrypted xlsx packages:
  an untrusted descriptor (Agile XML or the Standard binary header), a key derivation whose cost
  the file chooses, and an untrusted ciphertext. The spec's test plan and T-4265's acceptance
  criteria require a new fuzz target, `encrypted_package`, run in the fuzz job beside
  `cfb_container` (decision 0283). The job's target list is the matrix in
  `.github/workflows/fuzz.yml`, a protected path.
- **Decision:** the fuzz matrix gains `encrypted_package` (`-max_len=65536`), with the same time,
  timeout, memory and permission settings as the other targets (decision 0035). The target hands
  `rustsheet_xlsx::import::decrypt_ole_package` the input as an OLE container, tried with both
  fixture passwords, under a 16 MiB byte budget and a spin-count limit of 10,000 so a mutated
  `spinCount` is refused rather than hashed. Its seeds are the encrypted fixtures of
  crates/rustsheet-xlsx/tests/encrypted, copied by `fuzz/seed_corpus.py`. The fuzz crate gains a
  path dependency on `rustsheet_xlsx`, because decryption has no facade entry point before T-4266.
- **Consequences:** one more 20-minute nightly job. A crash it finds is filed by the steward like
  any fuzz finding (decision 0035). No PR gate changes. When T-4266 puts decryption behind
  `Workbook::open`, the target may move to the facade.
