# Telemetry dashboard and offline debug build

The dashboard observes a running VERA20k match: populations, production, AI
teams, map positions, selected paths, frame timing and process logs. The dashboard
design and initial implementation were contributed by
[1372704087](https://github.com/1372704087) in
[PR #2083](https://github.com/YuriPlanet/vera20k/pull/2083).

## Run

From the checkout root, with the ordinary retail data setup:

```sh
python -m tools.cargo_run -- build --locked --profile fast -p vera20k --features telemetry-dashboard --bin vera20k
VERA_TELEMETRY_ADDR=127.0.0.1:9741 "$(python -m tools.cargo_run --resolve vera20k --profile fast)"
```

Open the local URL printed to stderr. Its fragment contains a fresh session
credential. The page consumes and removes that fragment; reloading needs the
original printed URL again. Treat the URL as access to the current match and its
logs. The server requires the matching local Host, Origin and credential, serves
only fixed dashboard assets, and the dashboard-only build accepts no game commands. Non-loopback binding
is rejected. Close the game to revoke the session.

The feature is off by default and absent from ordinary release builds. Without
`VERA_TELEMETRY_ADDR`, it starts no server. `VERA_TELEMETRY_WEB` can select the
dashboard asset directory when running outside the checkout root. Live snapshot
publishing and the render/ledger meters start with the first authenticated
client; the whole-match summary ring (1 Hz) and the report-grade ring (5 s) are
recorded whether or not one is attached, and arrive with the first publish after
a client connects. The newest report sample rides every snapshot.
The existing `dev-overlays` feature and build profile are independent choices;
an optimized build can include these diagnostics without a separate debug branch.
The runner supports building and resolving the executable separately; it does not
accept `cargo run`.

On PowerShell, after the same build command, launch with:

```powershell
$env:VERA_TELEMETRY_ADDR = "127.0.0.1:9741"
$game = python -m tools.cargo_run --resolve vera20k --profile fast
& $game
```

## Offline trainer and event hooks

Use the same current `main` checkout, with the explicit extra feature:

```sh
python -m tools.cargo_run -- build --locked --profile fast -p vera20k --features debug-tools --bin vera20k
```

Then launch as above. `debug-tools` includes `telemetry-dashboard` and the existing
developer overlays. It enables the **Offline debug tools** panel only for a live
offline campaign or skirmish with an authenticated dashboard viewer. LAN,
Internet, unknown session modes and replay playback do not admit trainer writes.
Closing the last dashboard disconnects its controls and clears active switches.
New worlds and loads also reset the switches and invalidate queued commands.

The trainer restores the original contributor's money, power, instant-build,
invincibility, damage, technology, placement, reveal, heal, veteran and superweapon
controls. The inspector records at most 32 selected entities, with 64 bounded
events per entity; logging is off by default. History observes changes around
each object turn, not every internal transition. Browser commands are limited in
size, rate and queue length, and refer to the current world session.

These controls intentionally change gameplay. Affected health, money, completed
production and other results can persist in saved games even though the control
switches do not. Before the first trainer mutation, the app closes its ordinary
diagnostic replay segment and stops recording that modified world. Such a run
cannot serve as a normal replay or parity/performance baseline. Turning the
switches off does not undo previous actions. Save files do not retain a trainer
provenance marker: loading one resets the controls and starts a new diagnostic
segment, but does not make the saved state a clean gameplay baseline.

| Build | Features | Trainer and event hooks |
| --- | --- | --- |
| Shipped game | `--no-default-features` | Absent |
| Local development | Default `dev-overlays` | Absent |
| Read-only dashboard | `telemetry-dashboard` | Absent |
| Offline debug tools | `debug-tools` | Explicitly included |

The existing `fast` profile keeps release optimizations for interactive use;
these are optional development features, not a permanent fork of the engine.

For a synthetic demonstration without retail data:

```sh
python -m tools.cargo_run -- build --locked --profile fast -p vera20k --features telemetry-dashboard --bin telemetry-serve
"$(python -m tools.cargo_run --resolve telemetry-serve --profile fast)" --ticks 300
```

This fixture animates artificial entities. Its timing is not a gameplay benchmark.

## Startup parameters and usage

### Command line

| Switch | Effect |
| --- | --- |
| `-CD` | Use the wildcard media-archive branch. |
| `-NOAUDIO` | Disable music and sound output. |
| `-?`, `-h`, `/?`, `/h` | Print usage and exit. |
| `-WIN` | Accepted; no effect. Full screen is a choice on the Options page. |
| `-<W>X<H>`, `-480`, `-16`, `-RECORD`, `-PLAY`, `-SOCKET`, `-DESTNET`, other net/debug flags | Accepted so a launch never fails, not applied yet. |

### Environment

| Variable | Value | Effect |
| --- | --- | --- |
| `RA2_DIR` | retail install path | Where the engine reads `.mix` files. `config.toml` `[paths] ra2_dir` is the alternative; `RA2_DIR` wins when set. |
| `RA2_QUICKPLAY` | map name or path (`.map`/`.mpr`/`.mmx`), or a `.sed` seed | Load straight into that map through the authored-map sandbox: one local house, no generated opponents and no starting forces. |
| `RA2_DEBUG_SPAWN_UNITS` | `TYPE[:COUNT],...`, or `1` | Spawn units at load on the nearest free cells. A bare type is one unit; `=1` keeps the default list `HTNK,MTNK,E1`. |
| `RA2_DEBUG_SPAWN_MARCH` | `1` or `rx,ry` | Order every spawned object to attack-move across the map (`1`) or to one cell. |
| `VERA_TELEMETRY_ADDR` | `127.0.0.1:9741` | Start the read-only dashboard server on that loopback address. |
| `VERA_TELEMETRY_WEB` | directory | Dashboard asset root when running outside the checkout. |
| `RA2_DEBUG_DEPTH_VIEW`, `RA2_DEBUG_BRIDGE_RENDER_BUCKETS`, `RA2_DEBUG_CAMEO_PALETTES`, `RA2_DEBUG_MOUSE_CURSOR_SHEET`, `RA2_FORCE_TIB3_TO_TIB01`, `RA2_TIB_ID_OFFSET`, `RA2_FORCE_TIB_IMAGE`, `RA2_NORMALS`, `RA2_NORMAL_COUNT`, `RA2_QUEUE_FRAME_MS`, `RA2_DUMP_SHROUD`, `RA2_ENABLE_LAT`, `RA2_DISABLE_LAT` | — | Render and diagnostic toggles; not needed for ordinary runs. |

A scale run on a base-less map needs one `[General] BaseUnit=` type in the spawn
list (stock YR: `AMCV`, `SMCV`, `PCV`), or the defeat gate ends the match about
half a minute in.

### Examples

20,000 units marching on the flat 500x500 scale map:

```sh
python -m tools.scale_map target/scale/flat500.map --starts 8
python -m tools.cargo_run -- build --locked --profile fast -p vera20k   --features telemetry-dashboard --bin vera20k
game=$(python -m tools.cargo_run --resolve vera20k --profile fast)
RA2_QUICKPLAY="target/scale/flat500.map" RA2_DEBUG_SPAWN_UNITS=AMCV:1,HTNK:10000,MTNK:10000 RA2_DEBUG_SPAWN_MARCH=1 VERA_TELEMETRY_ADDR=127.0.0.1:9741 "$game"
```

Headless simulation benchmark (no window, no dashboard):

```sh
cargo build --release -p vera20k-core --bin sim-bench
target/release/sim-bench --scenario march --units 20000 --houses 30
```

### Recording semantics

The timeline on the match report page is tick-based, not wall-clock based.
Whole-match summaries close every 15 ticks (one second at the retail 15 Hz
logic rate); report-grade samples close every 75 ticks (five seconds) and each
one renders the same report as the live view. A report sample closes the
ledger window, so the live phase means cover at most the last five seconds and
briefly fall back to the window the sample just closed. Both rings are
app-owned, never read by the simulation, and are not persisted in saves.

## Limits and interpretation

Snapshots update at most ten times per second, including while paused. Population
totals cover the complete world; displayed detail is capped at 32 houses, 20,000
map objects, 16 roster types per house, 8 production lines, 16 AI teams and 32 base
nodes per house. Selected-object and cursor details each show at most 32 objects.
Names, log records, connections and network buffers are bounded. A slow client
may skip snapshots. Logs are a shared incremental stream; new tabs receive future
batches, not a complete historical replay.

Timing describes the current instrumented run and includes instrumentation cost.
Some frame work remains grouped under its existing owner rather than split into
new hooks. CPU share is normalized to all logical cores. The GPU memory field
counts the engine's palette-index atlas pages, not total driver allocation.
PNG and HTML exports are generated locally in the browser. No external services
are used. The dashboard-only feature includes no trainer, remote pause/step
controls or gameplay mutations.

The full original contribution from
[PR #2083](https://github.com/YuriPlanet/vera20k/pull/2083) is preserved unchanged
in a local maintainer Git archive for future integration. The optional debug
build adapts that work to current state owners and the session, replay and
transport rules above.

## Frontend changes

```sh
cd tools/telemetry-dashboard
npm ci --ignore-scripts --no-audit --no-fund
npm run build
```

Commit the TypeScript and generated JavaScript together. The manual Dashboard
workflow checks they agree. Vendored Three.js r180 is unchanged upstream code;
its license is in `vendor/THREE-LICENSE.txt`.

## Inspect a retained save without playing it

The existing `telemetry-serve` tool also has a local, read-only inspection mode:

```sh
python -m tools.cargo_run -- build --profile fast -p vera20k --bin telemetry-serve --no-default-features --features telemetry-dashboard,npatch
```

Resolve `telemetry-serve` with `tools.cargo_run --resolve telemetry-serve --profile fast`,
then run that executable with `--inspect-save <path>`. It prints JSON to stdout and
exits without a server, asset loading, simulation steps or save writes. Use the
save's actual gameplay capability (omit `npatch` for a stock save). The production
`GameSnapshot::load_with_resources` gate must accept the exact saved layout and
capability; never replace a header or bypass this check to inspect an old save.
Use a source checkout matching the writing build when layouts differ.

The report contains all retained houses, factories, team definitions and teams,
entities and the ID-to-string table, not the dashboard's capped lists. Normal
player-load planning cleanup still applies; resources and derived caches are not
rebound. It is a point-in-time observation, not proof of progress or strength, and
does not recompute provider availability. Reports can contain player names and
all match positions: keep them local (for example under ignored `target/`) unless
the player explicitly authorizes sharing. Do not commit user saves or reports.

## Trace selected saved objects locally

`--trace-save` is a separate, explicit continuation mode, not read-only inspection.
Using the same `telemetry-serve` build, set the following variables to the resolved
executable, a compatible save, its game-data root and IDs from the inspection
report. For example, in PowerShell:

```powershell
& $tool --trace-save $save --ra2-dir $assets --ticks 300 --observe-entity $entity_id --observe-team $team_id
```

Repeat the ID observer flags for additional IDs (at most 32 entities and 32 teams).
`--observe-cell x,y` adds a requested cell (at most 32 cells, unsigned 16-bit
coordinates). A cell-only trace is allowed. It reports retained terrain fields,
occupancy, the existing derived Normal/Water/WaterBeach connectivity labels
on ground and bridge layers, and native non-bridge raw IDs through the existing
GetZoneID owner. A raw native ID of 1 is distinct from derived invalid label 0.
These observations are **not** a full Map56D100 admission result, dynamic
cell-entry answer, or path search; missing terrain or a missing graph is `null`,
and an invalid derived zone label is zero. Native IDs are omitted for absent
terrain cells; no bridge walk or shared Dummy mutation is performed.
At least one observer is required; missing/retired objects produce `null`, not
invented state. Trace is limited to 300 ordinary frames and samples frame 0, 1,
10, 120, 300 and the requested final frame. It prepares and commits saved resources
through the same owner as the interactive loader, enables normal batched paths,
consumes due saved commands and issues no new player orders. It opens no window
or server and never writes the input save.

The continuation uses normal cold-load Main RNG seed zero and Scenario RNG reset,
not the original running process's streams. It is a short diagnostic continuation,
not an exact original-session replay, native comparison or whole-match AI verdict.
The JSON labels this scope and records the admitted envelope/layout and content
hashes. Keep reports local under ignored `target/`; selected object states can
reveal private match positions. Layout and resource checks are not optional.

## Inspect initial map cells locally

```powershell
& $tool --inspect-map $map --ra2-dir $assets --observe-cell $cell_xy
```

This uses the production headless initial Battle local-only loader and the same
cell observation owner, then exits without advancing frames, starting a server,
or writing saves. It is not the player's 1v1 roster, native execution or a match
replay. Only cell observers are allowed (at most 32); entity/team observers,
nonzero ticks, missing cells, and combinations with save modes are refused.
The JSON records the selected map, normal parsed-INI map content hash, effective
rules hash, seed, dimensions and tick0. When comparing with a save, first check
map/rules identity and compare whether endpoints share a zone within each run;
zone numbers alone need not stay identical across sessions or graph changes.
