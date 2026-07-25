# st0w's Music Streamer (sms)

Self-hosted, multi-source music streaming for personal use. In-file metadata is
the source of truth; a thin serverless control plane never touches audio bytes;
clients stream directly from agents over Tailscale. See
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

I wanted an app to be able to stream my own music privately, across Tailscale, from multiple sources. So I made this.

It's just a side/hobby project, so I have really made it mostly focused to my own needs. If that works for you, great! If you'd like to change something to add new features, I'd welcome PRs.

## Layout
```
server/           FastAPI control plane (Cloud Run)          — M1 skeleton, M2 core
agent/            Rust: scan, sync, stream, tag writeback    — M3
client/           React + TS PWA                             — M4
shared/           API + capability-token contracts           — M2
infra/            Terraform (Cloud Run, AR, Secret Mgr, WIF) — M1
spikes/metadata/  M0 tag round-trip spike (done)
docs/             ARCHITECTURE.md, M0-FINDINGS.md
```

## Status
- **M0 — metadata spike:** ✅ done (`docs/M0-FINDINGS.md`). Rating/notes/play-count/
  last-played round-trip in MP3/FLAC/M4A, independently verified.
- **M1 — repo + IaC + CI/CD:** this skeleton. Server runs and its lint/test go
  green; `terraform apply` stands up a public HTTPS endpoint; GitHub Actions
  deploys keylessly via Workload Identity Federation.
- **M2+ :** control-plane core, agent, client.

## Quick start
```bash
make server-check     # lint + test the server
make dev              # run it locally on :8080
# infra: see infra/README.md (terraform apply, then wire GitHub Actions vars)
```

## Security posture (M1)
- Keyless CI→GCP auth (Workload Identity Federation); no service-account keys.
- Least-privilege service accounts; runtime SA separate from deployer SA.
- Secrets in Secret Manager (containers here; values added out-of-band).
- Non-root, multi-stage container image.
- Deny-by-default CORS; app-layer auth is the access gate (M2).
