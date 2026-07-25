# agent

Rust binary that scans a local library, syncs its catalog to the server, serves
ranged audio over Tailscale HTTPS, and writes rating/notes/play-count tags into
source files (per `docs/M0-FINDINGS.md`). Implemented in **M3**. Current contents
are a compiling placeholder so the monorepo and CI are wired end-to-end.
