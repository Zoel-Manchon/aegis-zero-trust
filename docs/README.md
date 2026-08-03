# Demo assets

| File | What it is |
|------|------------|
| `demo.mp4` | ~60s walkthrough: sign in → MFA → console → attack range |
| `poster.png` | First frame, used as the clickable thumbnail in the root README |

## Recording checklist

Record at **1280×720**, browser maximized, on `https://localhost` — with the CA
trusted, so the padlock is visible. A zero-trust demo showing "Not secure" in the
address bar argues against itself.

| Seconds | Shot |
|---------|------|
| 0-6   | `/login` — the split screen. Sign in as the seeded admin. |
| 6-14  | MFA challenge, then the console. Let the KPI strip and live feed settle. |
| 14-24 | Scroll the three bands: Telemetry → Geo intelligence → Operations. |
| 24-34 | **Attack range**. Madrid + brute force, launch. Watch the feed and the launch log. |
| 34-46 | Switch origin to Singapore, launch again — impossible travel fires. Arc on the map, accent toast, tinted row. |
| 46-60 | **Account**: MFA panel, backup codes, passkeys. End on the console. |

## Keep it small

Target **under 5 MB** — a committed video lives in the history forever:

```bash
ffmpeg -i raw.mp4 -vf "scale=1280:-2,fps=24" -c:v libx264 -crf 30 -preset slow -an \
  -movflags +faststart demo.mp4

ffmpeg -i demo.mp4 -vframes 1 -q:v 3 poster.png
```

> **GitHub will not play a repo-relative `.mp4` inline.** The README shows
> `poster.png` as a clickable thumbnail that opens the file. For an actual inline
> player, drag the `.mp4` into a new issue comment on this repo and paste the
> `user-attachments` URL it returns, alone on its own line in the README — that
> also keeps the binary out of the repository entirely.
