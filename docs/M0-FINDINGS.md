# M0 — Metadata Compatibility Spike: Findings

**Status: PASSED. 14/14 round-trips.** Risk retired: rating, notes, play count,
and last-played all write durably into MP3, FLAC, and M4A/ALAC using standard
conventions, and the bytes are readable by parsers that aren't `lofty`.

## What was tested

Real 1-second audio files were generated with ffmpeg in each format (confirmed
codecs: `mp3`, `flac`, `alac`). The spike (Rust, using **`lofty` — the agent's
actual tag library**) wrote all four fields into each file, saved to disk,
reloaded, and read them back. Independent readers (**`mutagen`** and
**`ffprobe`**, neither of which is lofty) then confirmed the written bytes are
spec-correct and cross-reader durable. Notes deliberately included non-ASCII
(`café ☕ ✓`) to test UTF-8 handling.

## Verified tag map (this is the contract for M2/M3)

| Field        | MP3 (ID3v2)                    | FLAC (Vorbis)      | M4A/ALAC (MP4 atom)                 |
|--------------|--------------------------------|--------------------|-------------------------------------|
| Rating       | `POPM` rating byte             | `RATING` (0–100)   | `----:com.apple.iTunes:RATING`      |
|              |                                | + `FMPS_RATING`    |                                     |
| Notes        | `COMM` (UTF-8, lang `eng`)     | `COMMENT`          | `©cmt`                              |
| Play count   | `POPM` counter *(canonical)*   | `FMPS_PLAYCOUNT`   | `----:com.apple.iTunes:PLAYCOUNT`   |
|              | + raw `PCNT` *(also works)*    |                    |                                     |
| Last played  | `TXXX:LAST_PLAYED` (ISO-8601)  | `LAST_PLAYED`      | `----:com.apple.iTunes:LAST_PLAYED` |

### Rating scale — the one place formats diverge, handle it in a single mapping layer

The canonical internal value is **0–5 stars**. Each format encodes it differently,
so the agent needs one encode/decode layer, not scattered per-format logic:

- **MP3 `POPM`** stores a 0–255 *byte*. Write these anchor values:
  `5★=255, 4★=196, 3★=128, 2★=64, 1★=1, unrated=0`.
  When *reading* files rated by other apps, map ranges → stars:
  `224–255→5, 160–223→4, 96–159→3, 32–95→2, 1–31→1, 0→unrated`.
- **FLAC / M4A** store a 0–100 string (`stars × 20`, so `4★ = "80"`). FLAC
  additionally gets `FMPS_RATING` as a 0.0–1.0 float (`"0.8"`) for broad
  third-party-player interop.

All of these were confirmed round-tripping and independently readable.

## lofty-specific findings (carry into the agent build)

- **lofty 0.18.2 compiles cleanly on rustc 1.75** with no transitive-dependency
  MSRV conflicts. Pin the agent's lofty + rustc versions deliberately; newer
  lofty majors change the API and raise the MSRV.
- **lofty 0.18.2 has no dedicated `PCNT` frame type.** Play count was written two
  ways, both of which round-trip and both of which `mutagen` reads as `7`:
  1. the **`POPM` counter field** (first-class in lofty — *recommended canonical*),
  2. a **raw `PCNT` binary frame** (works, for players that specifically read PCNT).
  Recommendation: treat the POPM counter as canonical; optionally also emit PCNT.
- Notes/UTF-8 (including emoji) survive intact in all three formats.
- The MP4 comment used the standard `©cmt` atom; `ffprobe` surfaces it as
  `comment`, confirming it's the conventional field, not a custom one.

## Caveats and required behaviors for the real agent (NOT covered by the spike)

1. **Read-modify-write, never write-fresh.** The spike wrote *fresh* tags onto
   minimal files. The agent must **read existing tags, modify only the target
   field, and write back**, or it will drop the file's existing metadata
   (artist/album/art/etc.). This is the single most important implementation note.
2. **Safe in-place mutation.** Writing tags rewrites the file. The agent should
   write to a temp file and atomically rename, verify the re-read, and only then
   replace — consistent with the "verify before destructive action" principle in
   the architecture. Tag writes change `mtime` (expected; the audio-stream hash is
   recomputed and, because only tags changed, resolves to the *same* identity).
3. **Target readers.** With Music.app dropped, "durable/portable" means our own
   client + standard players. `mutagen` and `ffprobe` are strong stand-ins for
   "standard players." A final playback check on a real device is still worth
   doing once the client exists, but the *format* risk is retired.
4. **Environment artifact.** cargo was installed from Ubuntu repos (1.75) because
   `rust-lang.org` egress was blocked in the sandbox. Real dev/CI should use a
   proper pinned rustup toolchain — not a blocker, just noted.

## Files

- `m0-metadata-spike/main.rs` — the spike (throwaway, ~180 lines).
- `m0-metadata-spike/Cargo.toml` — `lofty = "0.18"`.
- `m0-metadata-spike/run-output.txt` — captured 14/14 result matrix.

## Acceptance

M0 acceptance criterion — *all four fields round-trip correctly in all three
formats; documented* — **met**, with independent cross-reader verification beyond
the original bar.
