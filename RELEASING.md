# Releasing FlareDeck

End-to-end flow for shipping a new version of FlareDeck:

1. Generate the updater signing keypair (one time).
2. Wire the public key into the repo and the private key into GitHub Actions secrets.
3. Tag a release. GitHub Actions builds binaries for Windows / macOS / Linux,
   uploads them to GitHub Releases, and pushes a fresh `latest.json` into
   flaredeck-web. Installed users see "Update available" the next time they
   open Settings.

## One-time setup

### 1. Generate the updater signing keypair

Run **locally** (the private key must never leave a machine you control):

```bash
npm run tauri signer generate -- --write-keys ~/.tauri/flaredeck-updater.key
```

Pick a strong password when prompted. Two files appear:

- `~/.tauri/flaredeck-updater.key` — **private key (KEEP SECRET).**
- `~/.tauri/flaredeck-updater.key.pub` — public key, base64. Safe to commit.

### 2. Put the public key into `tauri.conf.json`

Open [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json), find:

```json
"updater": {
  "pubkey": "REPLACE_WITH_BASE64_PUBKEY_AFTER_RUNNING_tauri_signer_generate",
```

Paste the contents of `~/.tauri/flaredeck-updater.key.pub` (a single base64
line, no newlines) over the placeholder. Commit.

> Why this is committed: the public key is the *trust root* for every
> FlareDeck installation. The updater plugin only accepts update bundles
> signed by the matching private key. If the pubkey ever changes,
> already-installed apps will refuse to update — they'd need to be
> uninstalled and reinstalled. Don't rotate it casually.

### 3. Add the GitHub Actions secrets

On the flaredeck GitHub repo: **Settings → Secrets and variables → Actions → New repository secret**. Add:

| Secret | Value |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | Contents of `~/.tauri/flaredeck-updater.key` (the whole file). |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | The password you set when generating the key. |
| `WEB_REPO` | The `owner/repo` of flaredeck-web (e.g. `milzamsz/flaredeck-web`). |
| `WEB_REPO_PAT` | A GitHub PAT (fine-grained, scoped to flaredeck-web with `contents:write`). Used to push `latest.json`. |

The default `GITHUB_TOKEN` already has permission to create releases on the
flaredeck repo itself — no setup needed for that.

### 4. Drop the download page into flaredeck-web

Two reference files are shipped in this repo at:

- [docs/website/Download.tsx](docs/website/Download.tsx) — React component for
  Next/Astro/Remix.
- [docs/website/download.html](docs/website/download.html) — plain HTML +
  vanilla JS, no build step.

Copy whichever matches your stack into `flaredeck-web`. Point a route at it
(e.g. `app/download/page.tsx` for Next App Router, or `public/download.html`
for plain static). Update the `GH_REPO` constant if your GitHub org/repo name
differs from `milzamsz/flaredeck`.

> The page expects `/latest.json` to exist at the site root. The release
> workflow places it there automatically (see below). Until the first release
> succeeds, the version line on the page will be blank — that's expected.

## Each release

### 1. Bump the version

The release workflow uses whatever tag you push as the version. Tauri also
embeds the version from two places — keep them in sync:

```bash
# 1. Bump src-tauri/tauri.conf.json -> "version"
# 2. Bump src-tauri/Cargo.toml -> [package] version
# 3. Bump package.json -> "version"
```

Commit the three-file bump as `release: vX.Y.Z` (no tag yet).

### 2. Tag and push

```bash
git tag -a vX.Y.Z -m "FlareDeck vX.Y.Z"
git push origin vX.Y.Z
```

The push triggers `.github/workflows/release.yml`. It runs three platform
build jobs in parallel (~10-20 min each), then a `manifest` job that:

1. Downloads the per-platform `latest*.json` files tauri-action just uploaded
   to the release.
2. Merges them into a single `latest.json`.
3. Commits that file into `flaredeck-web/public/latest.json` and pushes.
4. Your web host (Vercel/Cloudflare Pages/etc.) auto-deploys from the new
   commit. Installed FlareDeck users see the update on their next Settings
   visit.

### 3. (Optional) Trigger manually

If you didn't tag — say you want to redo a build without bumping — use:

GitHub → Actions → Release → **Run workflow** → enter the tag name.

## Verifying the release worked

1. **GitHub Releases page** at
   `https://github.com/milzamsz/flaredeck/releases` should show the new tag
   with .exe, .dmg, .AppImage, .deb assets attached.
2. **`https://www.flaredeck.dev/latest.json`** should return JSON with
   `"version": "X.Y.Z"` and a `platforms` map.
3. **`https://www.flaredeck.dev/download`** should show the right "Download for…"
   button for your OS and link to the new version's installer.
4. **An installed FlareDeck instance** opened on the OLD version should
   show "Update available" in Settings after a few seconds. Click
   "Download and install"; watch the progress bar; click "Restart now". Open
   Settings again and confirm "Up to date".

## Rolling back

If a release is broken:

```bash
# Option A: yank latest.json (stops auto-updates while you fix it)
# In flaredeck-web:
git revert <commit-that-updated-latest.json>
git push

# Option B: ship a hotfix
# Just tag vX.Y.(Z+1) with the fix and the workflow takes over.
```

GitHub Releases is immutable for assets by default — don't try to overwrite a
shipped binary with the same filename. Bump the patch version and reship.

## Troubleshooting

- **"Updater signature mismatch"** in the in-app log: the pubkey in
  `tauri.conf.json` doesn't match the private key in CI. Re-check the secret
  was pasted in full (it's multi-line — preserve line breaks).
- **macOS build fails on Apple Silicon**: tauri-action needs
  `target=universal-apple-darwin`; verify the matrix entry still has
  `args: --target universal-apple-darwin`.
- **Linux build fails with `libwebkit2gtk-4.1` missing**: Ubuntu 24.04 split
  the package. The workflow installs the right one for 22.04; if you switch
  runners, update the apt-get line.
- **`latest.json` not appearing on the site**: the `manifest` job step
  "Checkout flaredeck-web" requires `WEB_REPO_PAT`. If that secret is missing
  or the PAT lacks `contents:write` on the right repo, the job logs will show
  a 403.
- **Update notification doesn't appear in-app**: the user-facing version
  number must be strictly greater than what's installed. Tauri compares by
  semver — `0.2.0` won't update to `0.2.0`, and `0.10.0` does sort after
  `0.9.0` (it's not lexicographic).
