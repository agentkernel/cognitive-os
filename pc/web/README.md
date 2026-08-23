# CognitiveOS Personal Web UI (`pc/web`)

Localhost-only daemon client for CognitiveOS Personal (P7-T05). This is **not**
CognitiveOS Console and must not be implemented under `pc/app/`.

- Stack: React + TypeScript + Vite (ADR-0053)
- Serving: same-origin daemon `GET /ui/` (HashRouter; no BrowserRouter fallback)
- Session: `POST /local/session` per channel; bearers stay in memory
- Cookies, CORS, CDN, AGPL runtime deps, and browser authority writes are forbidden

```text
pnpm install
pnpm test
pnpm build
```

Copy `dist/` to the daemon `data_dir()/ui` (or `{runtime-root}/cognitiveos/ui`)
on the exact kernel revision under review. Vite preview is not the product origin.

Claim ceiling: `hypothesis`. Not a Gate, release, Profile, or B01 claim.
