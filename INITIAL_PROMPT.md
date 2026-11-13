# Initial Prompt for New Antigravity Chat

Copy and paste the exact command and prompt below into your new Antigravity session:

---

### Skill & Prompt to Copy:

```text
/cs-backend-engineer Read AGENTS.md in /home/sarthak/Desktop/Projects/Softify-Release.

We are building Softify Next-Gen from scratch using Rust Core + Kotlin Compose Multiplatform.

CRITICAL INSTRUCTIONS:
1. STRICT ISOLATION: Do NOT inspect or reference the legacy /softify directory under any circumstance. All specifications, endpoints, DES keys, and algorithms are self-contained in AGENTS.md.
2. BACKEND-FIRST ONLY: Focus 100% on the Rust Core Engine (core/softify-core) and CLI runner (core/softify-cli). Do NOT write any Kotlin or frontend UI code at this stage.
3. 320 KBPS AUDIO FIDELITY: Ensure JioSaavn DES decryptor and stream URL rewriter strictly output CD-quality 320 Kbps AAC streams (_320.mp4).
4. OFFLOAD BUILDS & TESTS TO GITHUB CI: The user's machine has limited resources. Do NOT run heavy local release builds or continuous local test loops. Commit changes and push to GitHub so GitHub Actions runs the tests and checks on the cloud runners.
5. SCOPE FOR THIS SESSION:
   - Scaffold the Rust workspace in core/ with softify-core and softify-cli.
   - Implement domain models (TrackMetadata, StreamResult, SearchResult).
   - Implement clean DES-ECB decryptor (key: 38346591) and JioSaavn 320 Kbps stream resolver.
   - Implement Spotify Pathfinder anonymous session and search client.
   - Implement YouTube Music InnerTube (WEB_REMIX) fallback with Topic channel scoring.
   - Setup UniFFI scaffolding for Kotlin bindings.
   - Commit and push to GitHub, verifying CI test runs via `gh run list`.
```
