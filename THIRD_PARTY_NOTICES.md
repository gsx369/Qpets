# Third-party notices

Qpets uses open-source projects including Tauri, Vue, Vite, Vitest, Rust crates and their transitive dependencies. Their exact versions are recorded in `pnpm-lock.yaml` and `src-tauri/Cargo.lock`; each dependency remains governed by its own license.

The Qpets character artwork is not an open-source dependency and is covered by `ASSETS_LICENSE.md`.

## codex-pets reference

The character-asset workflow and the basic `pet.json`, `spriteVersionNumber: 2`, and `spritesheet.webp` conventions used by Qpets were informed by [YaKun9/codex-pets](https://github.com/YaKun9/codex-pets).

This is a design and workflow acknowledgement, not a declaration that `codex-pets` is a runtime dependency. Qpets does not copy or redistribute the upstream website code or any existing upstream pet asset directory. Qpets defines additional application-specific package metadata, dialogue files, validation rules, safe-import constraints, and local character-management behavior; an upstream pet directory is therefore not guaranteed to be directly compatible with Qpets.

The upstream repository states that its website code and repository-level documentation are licensed under its [MIT License](https://github.com/YaKun9/codex-pets/blob/main/LICENSE). Pet-specific creative assets are excluded from that repository-wide MIT license and are instead governed by the upstream [asset licensing policy](https://github.com/YaKun9/codex-pets/blob/main/ASSETS_LICENSE.md) and each pet directory's own `LICENSE.md`. If a future Qpets package incorporates upstream pet assets, the applicable directory-level notice and attribution must accompany that package.
