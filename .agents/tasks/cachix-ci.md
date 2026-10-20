# Cachix CI Setup for sisho-desk

Status: completed. Verified 2026-10-20.

## Objective

Set up GitHub Actions CI with Cachix for `sisho-desk`, matching the setup in `../sisho`:
- Add `.github/workflows/ci.yml` running Nix build (`nix build .#sisho-desk`), Cachix caching (`cachix/cachix-action@v15` with cache name `tomet`), and doc checks (`nix develop --command just docs-check`).
- Verify `flake.nix` configuration matches `../sisho` (`extra-substituters` and `extra-trusted-public-keys` for Cachix).

## Steps

- [x] Verify `flake.nix` `nixConfig` matches Cachix settings from `../sisho`.
- [x] Create `.github/workflows/ci.yml` matching `../sisho/.github/workflows/ci.yml`.
- [x] Verify local Nix package build (`nix build .#sisho-desk`) and test checks (`setsid -w cargo test --workspace`).
- [x] Verify `nix develop --command just docs-check`.
- [x] Await user confirmation for commit and description.
