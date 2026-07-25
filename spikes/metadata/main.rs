// M0 metadata compatibility spike.
// Writes rating / notes / play-count / last-played into MP3, FLAC, M4A(ALAC)
// using `lofty` (the agent's real tag library), reloads from disk, reads back,
// and prints a round-trip PASS/FAIL matrix.

use std::fs::File;
use lofty::{AudioFile, TagExt, TextEncoding, ParseOptions};
use lofty::mpeg::MpegFile;
use lofty::flac::FlacFile;
use lofty::mp4::{Mp4File, Ilst, Atom, AtomIdent, AtomData};
use lofty::ogg::VorbisComments;
use lofty::id3::v2::{Id3v2Tag, Frame, FrameFlags, FrameId, FrameValue,
                     Popularimeter, CommentFrame};

// ---- shared test values ----
const STARS: u8 = 4;
const POPM_BYTE: u8 = 196;      // WMP de-facto: 5=255,4=196,3=128,2=64,1=1
const RATING_0_100: &str = "80"; // stars*20
const FMPS_RATING: &str = "0.8"; // FMPS float 0.0-1.0
const NOTES: &str = "M0 round-trip — café ☕ ✓"; // non-ASCII on purpose
const PLAY_COUNT: u64 = 7;
const LAST_PLAYED: &str = "2026-07-24T22:00:00Z";

struct Row { fmt: &'static str, field: &'static str, want: String, got: String, pass: bool }
fn rec(rows: &mut Vec<Row>, fmt: &'static str, field: &'static str, want: String, got: Option<String>) {
    let got_s = got.clone().unwrap_or_else(|| "<none>".into());
    let pass = got.as_deref() == Some(want.as_str());
    rows.push(Row { fmt, field, want, got: got_s, pass });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rows: Vec<Row> = Vec::new();
    write_mp3("files/test.mp3")?;   read_mp3("files/test.mp3", &mut rows)?;
    write_flac("files/test.flac")?; read_flac("files/test.flac", &mut rows)?;
    write_m4a("files/test.m4a")?;   read_m4a("files/test.m4a", &mut rows)?;

    println!("\n{:<6} {:<12} {:<30} {:<30} {}", "FMT","FIELD","EXPECTED","GOT","");
    println!("{}", "-".repeat(88));
    let (mut ok, mut total) = (0, 0);
    for r in &rows {
        total += 1; if r.pass { ok += 1; }
        println!("{:<6} {:<12} {:<30} {:<30} {}", r.fmt, r.field, r.want, r.got,
                 if r.pass { "PASS" } else { "FAIL" });
    }
    println!("{}", "-".repeat(88));
    println!("{ok}/{total} round-trips passed");
    Ok(())
}

// ---------------- MP3 (ID3v2) ----------------
fn write_mp3(p: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut tag = Id3v2Tag::new();
    // rating -> POPM (rating byte + counter doubles as play count)
    let popm = Popularimeter { email: "m0-spike@music.local".into(), rating: POPM_BYTE, counter: PLAY_COUNT };
    tag.insert(Frame::new("POPM", FrameValue::Popularimeter(popm), FrameFlags::default())?);
    // dedicated PCNT play counter (raw binary experiment)
    let pcnt = (PLAY_COUNT as u32).to_be_bytes().to_vec();
    tag.insert(Frame::new("PCNT", FrameValue::Binary(pcnt), FrameFlags::default())?);
    // notes -> COMM
    let comm = CommentFrame { encoding: TextEncoding::UTF8, language: *b"eng", description: String::new(), content: NOTES.into() };
    tag.insert(Frame::new("COMM", FrameValue::Comment(comm), FrameFlags::default())?);
    // last played -> TXXX:LAST_PLAYED
    tag.insert_user_text("LAST_PLAYED".into(), LAST_PLAYED.into());
    tag.save_to_path(p)?;
    Ok(())
}
fn read_mp3(p: &str, rows: &mut Vec<Row>) -> Result<(), Box<dyn std::error::Error>> {
    let f = MpegFile::read_from(&mut File::open(p)?, ParseOptions::default())?;
    let tag = f.id3v2().ok_or("no id3v2 after write")?;
    // POPM
    let (mut rate, mut cnt) = (None, None);
    if let Some(fr) = tag.get(&FrameId::new("POPM")?) {
        if let FrameValue::Popularimeter(p) = fr.content() { rate = Some(p.rating.to_string()); cnt = Some(p.counter.to_string()); }
    }
    rec(rows, "mp3", "rating", POPM_BYTE.to_string(), rate);
    rec(rows, "mp3", "playcount", PLAY_COUNT.to_string(), cnt); // via POPM counter
    // PCNT (raw)
    let mut pcnt = None;
    if let Some(fr) = tag.get(&FrameId::new("PCNT")?) {
        if let FrameValue::Binary(b) = fr.content() {
            let mut arr = [0u8;8]; let s = 8 - b.len().min(8); arr[s..].copy_from_slice(&b[b.len().saturating_sub(8)..]);
            pcnt = Some(u64::from_be_bytes(arr).to_string());
        }
    }
    rec(rows, "mp3", "PCNT(raw)", PLAY_COUNT.to_string(), pcnt);
    // COMM
    let note = tag.comments().next().map(|c| c.content.clone());
    rec(rows, "mp3", "notes", NOTES.to_string(), note);
    // TXXX
    rec(rows, "mp3", "lastplayed", LAST_PLAYED.to_string(), tag.get_user_text("LAST_PLAYED").map(|s| s.to_string()));
    Ok(())
}

// ---------------- FLAC (Vorbis comments) ----------------
fn write_flac(p: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut vc = VorbisComments::new();
    vc.insert("RATING".into(), RATING_0_100.into());
    vc.insert("FMPS_RATING".into(), FMPS_RATING.into());
    vc.insert("COMMENT".into(), NOTES.into());
    vc.insert("FMPS_PLAYCOUNT".into(), PLAY_COUNT.to_string());
    vc.insert("LAST_PLAYED".into(), LAST_PLAYED.into());
    vc.save_to_path(p)?;
    Ok(())
}
fn read_flac(p: &str, rows: &mut Vec<Row>) -> Result<(), Box<dyn std::error::Error>> {
    let f = FlacFile::read_from(&mut File::open(p)?, ParseOptions::default())?;
    let vc = f.vorbis_comments().ok_or("no vorbis comments after write")?;
    rec(rows, "flac", "rating", RATING_0_100.into(), vc.get("RATING").map(str::to_string));
    rec(rows, "flac", "fmps_rate", FMPS_RATING.into(), vc.get("FMPS_RATING").map(str::to_string));
    rec(rows, "flac", "notes", NOTES.into(), vc.get("COMMENT").map(str::to_string));
    rec(rows, "flac", "playcount", PLAY_COUNT.to_string(), vc.get("FMPS_PLAYCOUNT").map(str::to_string));
    rec(rows, "flac", "lastplayed", LAST_PLAYED.into(), vc.get("LAST_PLAYED").map(str::to_string));
    Ok(())
}

// ---------------- M4A / ALAC (MP4 atoms) ----------------
fn freeform(name: &str) -> AtomIdent<'static> {
    AtomIdent::Freeform { mean: "com.apple.iTunes".into(), name: name.to_string().into() }
}
fn write_m4a(p: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut ilst = Ilst::new();
    ilst.insert(Atom::new(freeform("RATING"), AtomData::UTF8(RATING_0_100.into())));
    ilst.insert(Atom::new(freeform("PLAYCOUNT"), AtomData::UTF8(PLAY_COUNT.to_string())));
    ilst.insert(Atom::new(freeform("LAST_PLAYED"), AtomData::UTF8(LAST_PLAYED.into())));
    // notes -> standard ©cmt atom
    ilst.insert(Atom::new(AtomIdent::Fourcc(*b"\xa9cmt"), AtomData::UTF8(NOTES.into())));
    ilst.save_to_path(p)?;
    Ok(())
}
fn read_m4a(p: &str, rows: &mut Vec<Row>) -> Result<(), Box<dyn std::error::Error>> {
    let f = Mp4File::read_from(&mut File::open(p)?, ParseOptions::default())?;
    let ilst = f.ilst().ok_or("no ilst after write")?;
    let get = |id: &AtomIdent| -> Option<String> {
        ilst.get(id).and_then(|a| a.data().next()).and_then(|d| match d { AtomData::UTF8(s) => Some(s.clone()), _ => None })
    };
    rec(rows, "m4a", "rating", RATING_0_100.into(), get(&freeform("RATING")));
    rec(rows, "m4a", "playcount", PLAY_COUNT.to_string(), get(&freeform("PLAYCOUNT")));
    rec(rows, "m4a", "lastplayed", LAST_PLAYED.into(), get(&freeform("LAST_PLAYED")));
    rec(rows, "m4a", "notes", NOTES.into(), get(&AtomIdent::Fourcc(*b"\xa9cmt")));
    Ok(())
}
