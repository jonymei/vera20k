//! Local save/map inspection and headless telemetry demo.
//!
//! `--inspect-save` reads a layout-compatible save through production admission
//! and prints retained owners locally without running the simulation. The
//! explicit `--trace-save` mode cold-loads and advances selected observers.
//! `--inspect-map` instead observes selected cells after initial map construction,
//! without advancing frames.
//! Otherwise, this runs a live synthetic simulation with the browser dashboard,
//! no retail assets required.
//!
//! Builds the same synthetic two-house world `parity-digest` validates
//! against (passive houses, lifecycle-free entities, empty bound resources),
//! then ticks it at the retail cadence while serving
//! `sim::telemetry::TelemetrySnapshot` JSON over the telemetry server. Open
//! the printed URL in a browser to watch the dashboard.
//!
//! This is a pipeline demo, not a benchmark: the synthetic world runs almost
//! no rules-driven work, so its tick costs say nothing about a real match's.
//! What it does prove is the full path — capture, serialize, serve, render —
//! and that a match can be watched without a window. The stroll that animates
//! the map view is demo animation, not simulation (see `stroll`).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use vera20k::app::telemetry::{TelemetryServer, TickTimingRing};
use vera20k_core::headless_scenario::SIM_TICK_MS;
use vera20k_core::map::entities::EntityCategory;
use vera20k_core::sim::combat::Health;
use vera20k_core::sim::components::DriveCoord;
use vera20k_core::sim::house_state::HouseState;
use vera20k_core::sim::runtime::{SimResources, SimRuntime};
use vera20k_core::sim::telemetry::{SessionContext, TerrainGrid};
use vera20k_core::sim::world::Simulation;

/// Entities per house per category.
const DEFAULT_PER_CATEGORY: u16 = 120;
/// Structures per house.
const DEFAULT_STRUCTURES: u16 = 6;
/// Stroll speed in cells per tick (7.5 cells/s at the headless 15 Hz).
const STROLL_STEP: f32 = 0.5;
/// The one-shot movement probe fires at this tick (3 s in at 15 Hz).
const PROBE_AT: u64 = 45;
/// Publish rate: one snapshot every second tick (7.5 Hz at 15 Hz).
const PUBLISH_EVERY: u64 = 2;
/// Map side for the synthetic world when the simulation carries no map.
const DEFAULT_MAP_SIDE: u16 = 128;

struct Args {
    addr: SocketAddr,
    web_root: PathBuf,
    per_category: u16,
    structures: u16,
    ticks: u64,
    seed: u32,
    inspect_save: Option<PathBuf>,
    inspect_map: Option<String>,
    trace_save: Option<PathBuf>,
    ra2_dir: Option<PathBuf>,
    observe_entities: Vec<u64>,
    observe_teams: Vec<u64>,
    observe_cells: Vec<(u16, u16)>,
}

fn parse_args() -> Result<Args, String> {
    let mut addr: SocketAddr = "127.0.0.1:9741".parse().unwrap();
    let mut web_root = PathBuf::from("tools/telemetry-dashboard");
    let mut per_category = DEFAULT_PER_CATEGORY;
    let mut structures = DEFAULT_STRUCTURES;
    let mut ticks = 0u64;
    let mut seed = 0x5EED_0001u32;
    let mut inspect_save = None;
    let mut inspect_map = None;
    let mut trace_save = None;
    let mut ra2_dir = None;
    let mut observe_entities = Vec::new();
    let mut observe_teams = Vec::new();
    let mut observe_cells = Vec::new();
    let mut argv = std::env::args().skip(1);
    while let Some(flag) = argv.next() {
        let mut next = |what: &str| argv.next().ok_or(format!("{flag} needs {what}"));
        match flag.as_str() {
            "--observe-cell" => {
                let raw = next("x,y cell coordinates")?;
                let (x, y) = raw.split_once(',').ok_or("--observe-cell takes x,y")?;
                let x = x
                    .parse::<u16>()
                    .map_err(|_| "--observe-cell takes unsigned u16 x,y")?;
                let y = y
                    .parse::<u16>()
                    .map_err(|_| "--observe-cell takes unsigned u16 x,y")?;
                if observe_cells.len() == 32 {
                    return Err("--observe-cell accepts at most 32 cells".into());
                }
                observe_cells.push((x, y));
            }
            "--observe-entity" | "--observe-team" => {
                let id = next("an unsigned stable ID")?
                    .parse::<u64>()
                    .map_err(|_| format!("{flag} takes an unsigned stable ID"))?;
                let ids = if flag == "--observe-entity" {
                    &mut observe_entities
                } else {
                    &mut observe_teams
                };
                if ids.len() == 32 {
                    return Err(format!("{flag} accepts at most 32 IDs"));
                }
                ids.push(id);
            }
            "--trace-save" => trace_save = Some(PathBuf::from(next("a save path")?)),
            "--ra2-dir" => ra2_dir = Some(PathBuf::from(next("an asset root")?)),
            "--inspect-save" => inspect_save = Some(PathBuf::from(next("a save path")?)),
            "--inspect-map" => inspect_map = Some(next("a map file name")?),
            "--addr" => {
                let raw = next("an address")?;
                addr = raw
                    .parse()
                    .map_err(|error| format!("--addr takes an address: {error}"))?;
            }
            "--web" => web_root = PathBuf::from(next("a directory")?),
            "--units" => {
                per_category = next("a count")?
                    .parse()
                    .map_err(|_| "--units takes a count")?
            }
            "--structures" => {
                structures = next("a count")?
                    .parse()
                    .map_err(|_| "--structures takes a count")?
            }
            "--ticks" => {
                ticks = next("a count")?
                    .parse()
                    .map_err(|_| "--ticks takes a count")?;
            }
            "--seed" => {
                let raw = next("a 32-bit value")?;
                seed = raw
                    .strip_prefix("0x")
                    .map(|hex| u32::from_str_radix(hex, 16))
                    .unwrap_or_else(|| raw.parse())
                    .map_err(|_| "--seed takes a u32")?;
            }
            "--help" | "-h" => {
                println!(
                    "usage: telemetry-serve [--addr 127.0.0.1:9741] [--web tools/telemetry-dashboard] \
                     [--units 120] [--structures 6] [--ticks 0] [--seed 0x5EED0001]\n\
                     or: telemetry-serve --inspect-map <file> --ra2-dir <root> --observe-cell <x,y>...\n\
                     or: telemetry-serve --inspect-save <path>\n\
                     or: telemetry-serve --trace-save <path> --ra2-dir <root> [--ticks 300]\n\
                     (--observe-entity <id> | --observe-team <id> | --observe-cell <x,y>)...\n\
                     \n\
                     --inspect-save <path> prints a local read-only JSON report and exits.\n\
                     Inspection admits only compatible saves; no server or ticks are started.\n\
                     Trace instead cold-loads and advances at most 300 frames locally.\n\
                     Trace requires observers (at most 32 entities, 32 teams and 32 cells).\n\
                     Inspection reports contain complete retained owners; traces contain requested IDs.\n\
                     Runs the synthetic two-house simulation headlessly and serves the\n\
                     browser dashboard. --ticks 0 (the default) runs until interrupted."
                );
                std::process::exit(0);
            }
            other => return Err(format!("unrecognised argument {other}")),
        }
    }
    Ok(Args {
        addr,
        web_root,
        per_category,
        structures,
        ticks,
        seed,
        inspect_save,
        inspect_map,
        trace_save,
        ra2_dir,
        observe_entities,
        observe_teams,
        observe_cells,
    })
}

/// Two passive houses with a base, an army and a sky, spread over the map.
/// Mirrors `parity-digest`'s synthetic world; entities are lifecycle-free
/// diagnostic inserts, so nothing here is gameplay.
///
/// A fresh synthetic `Simulation` carries no map, so the session's extent is
/// set here — the dashboard frames its ground plane from it, and wander
/// orders target cells inside it.
fn build_world(seed: u32, per_category: u16, structures: u16) -> Simulation {
    let mut sim = Simulation::synthetic(
        u64::from(seed),
        &vera20k_core::sim::scenario_session::ScenarioDescriptor {
            map_name: "synthetic.map".to_owned(),
            map_width: DEFAULT_MAP_SIDE,
            map_height: DEFAULT_MAP_SIDE,
            ..Default::default()
        },
    );
    let width = sim.session().map_width();
    let height = sim.session().map_height();
    let homes = [
        (10u16, 10u16),
        (width.saturating_sub(30), height.saturating_sub(30)),
    ];

    let mut rng = u64::from(seed).wrapping_mul(2654435761);
    let mut next_random = move || {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((rng >> 33) as u32) & 0xFFFF
    };

    for (index, (owner_name, side)) in [("Americans", 0u8), ("Russians", 1u8)].iter().enumerate() {
        let owner = sim.intern(owner_name);
        let mut house = HouseState::new(owner, *side, None, index == 0, 10_000, 10);
        // Raw-inserted entities would otherwise resolve the match on tick 1.
        house.make_multiplay_passive();
        sim.register_house(house);

        let (hx, hy) = homes[index];
        let type_ref = sim.intern("GACNST");
        let infantry_ref = sim.intern("E1");
        let vehicle_ref = sim.intern("HARV");
        let aircraft_ref = sim.intern("MIG");

        for slot in 0..structures {
            sim.insert_synthetic_techno_for_diagnostics(
                hx + (slot % 3) * 2,
                hy + (slot / 3) * 2,
                0,
                0,
                owner,
                Health::new(1000),
                type_ref,
                EntityCategory::Structure,
                0,
                6,
                false,
            );
        }
        let placements = [
            (EntityCategory::Infantry, infantry_ref, false),
            (EntityCategory::Unit, vehicle_ref, true),
            (EntityCategory::Aircraft, aircraft_ref, true),
        ];
        for (category, type_ref, is_voxel) in placements {
            for _ in 0..per_category {
                let spread_x = u16::try_from(next_random() % 40).unwrap_or(0);
                let spread_y = u16::try_from(next_random() % 40).unwrap_or(0);
                let x = (hx + spread_x).min(width.saturating_sub(1));
                let y = (hy + spread_y).min(height.saturating_sub(1));
                sim.insert_synthetic_techno_for_diagnostics(
                    x,
                    y,
                    0,
                    0,
                    owner,
                    Health::new(1000),
                    type_ref,
                    category,
                    0,
                    6,
                    is_voxel,
                );
            }
        }
    }
    sim
}

/// Synthetic stroll: walk every non-structure entity toward a fresh random
/// cell, so the dashboard's map view animates.
///
/// The synthetic entities are lifecycle-free diagnostic inserts: the movement
/// system never touches them (they are not in the logic vector), so issuing
/// real move commands does nothing — the startup probe proved that with
/// 0/732 movers. This stroll therefore places them through
/// `Simulation::place_synthetic_for_diagnostics`, which moves only such inserts.
/// It is demo animation, not simulation: the real engine moves entities
/// through the locomotor and command path, and the dashboard renders whatever
/// the sim says either way.
fn stroll(sim: &mut Simulation, tick: u64, targets: &mut HashMap<u64, (f32, f32)>) {
    let width = u32::from(sim.session().map_width());
    let height = u32::from(sim.session().map_height());
    if width == 0 || height == 0 {
        return;
    }
    let mut rng = u64::from(tick)
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let mut next_random = move || {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (rng >> 33) as u32
    };
    let mut fresh_target = || {
        (
            (next_random() % width) as f32 + 0.5,
            (next_random() % height) as f32 + 0.5,
        )
    };
    let walkers: Vec<(u64, f32, f32)> = sim
        .entities()
        .values()
        .filter(|entity| entity.category() != EntityCategory::Structure)
        .map(|entity| {
            let position = &entity.position;
            (
                entity.stable_id(),
                f32::from(position.rx()) + position.sub_x().to_num::<f32>() / 256.0,
                f32::from(position.ry()) + position.sub_y().to_num::<f32>() / 256.0,
            )
        })
        .collect();
    for (id, cur_x, cur_y) in walkers {
        let target = targets.entry(id).or_insert_with(&mut fresh_target);
        let (dx, dy) = (target.0 - cur_x, target.1 - cur_y);
        let distance = (dx * dx + dy * dy).sqrt();
        let (next_x, next_y) = if distance <= STROLL_STEP {
            *target = fresh_target();
            (cur_x + dx, cur_y + dy)
        } else {
            (
                cur_x + dx / distance * STROLL_STEP,
                cur_y + dy / distance * STROLL_STEP,
            )
        };
        sim.place_synthetic_for_diagnostics(
            id,
            DriveCoord {
                x: (next_x * 256.0) as i32,
                y: (next_y * 256.0) as i32,
                z: 0,
            },
        );
    }
}

/// Synthetic terrain for the demo world: the app's grid is the loaded map's,
/// but the synthetic world has none, so the 3D view would be flat without
/// this. The layout is designed, not real map data, so one look shows the
/// dashboard's terracing and all three land features the wire carries:
///
///  - a level-2 plateau carrying a rectangular level-0 **lake** (water land
///    type, so it paints solid blue with no grid);
///  - the lake's north shore is one **ramp** row, a non-zero slope type whose
///    top tilts down into the water instead of dropping a step face;
///  - a raised **bridge** deck crosses the lake at its middle row, lifted to
///    the plateau level and flush with both shores, with side faces down to
///    the water — the same `deck_rises` the app encodes from a structural deck.
///
/// Encoded row-major, two lowercase hex digits per cell, in the same order and
/// format the app's capture uses.
fn synthetic_terrain(width: u16, height: u16) -> TerrainGrid {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    /// Plateau ground level; the lake sits `Lake` levels below it.
    const PLATEAU: u8 = 2;
    const LAKE: u8 = 0;
    /// `rules::terrain_rules::LandType::Clear`.
    const CLEAR: u8 = 0;
    /// `rules::terrain_rules::LandType::Water`.
    const WATER: u8 = 2;
    /// One ramp tile: any non-zero slope type tilts the cell's top.
    const RAMP: u8 = 1;

    let w = u32::from(width);
    let h = u32::from(height);
    // Lake rectangle, inset from the map edges; the ramp row lies just north
    // of it and the bridge row down its middle.
    let lake_x0 = w * 3 / 8;
    let lake_x1 = w * 5 / 8;
    let lake_y0 = h * 3 / 8;
    let lake_y1 = h * 5 / 8;
    let ramp_row = lake_y0.saturating_sub(1);
    let bridge_row = (lake_y0 + lake_y1) / 2;
    // The span reaches a few cells onto each shore so its abutments meet the
    // plateau; only the cells over water carry a raised deck.
    let span_x0 = lake_x0.saturating_sub(3);
    let span_x1 = lake_x1 + 3;

    let stride = usize::from(width) * usize::from(height) * 2;
    let mut levels = String::with_capacity(stride);
    let mut land_types = String::with_capacity(stride);
    let mut slope_types = String::with_capacity(stride);
    let mut deck_rises = String::with_capacity(stride);
    let hex = |out: &mut String, value: u8| {
        out.push(HEX[usize::from(value >> 4)] as char);
        out.push(HEX[usize::from(value & 0x0f)] as char);
    };

    for ry in 0..h {
        for rx in 0..w {
            let over_lake = rx >= lake_x0 && rx < lake_x1 && ry >= lake_y0 && ry < lake_y1;
            let level = if over_lake { LAKE } else { PLATEAU };
            let land = if over_lake { WATER } else { CLEAR };
            // The north shore tilts down into the water; its own level stays the
            // plateau's, and the corner interpolation reaches the lake.
            let slope = if ry == ramp_row && rx >= lake_x0 && rx < lake_x1 {
                RAMP
            } else {
                0
            };
            // A bridge cell over the water lifts a deck up to the plateau, so
            // the span lands flush with both shores; shore cells are ordinary
            // ground and add no rise.
            let deck = if ry == bridge_row && over_lake && rx >= span_x0 && rx < span_x1 {
                PLATEAU - LAKE
            } else {
                0
            };
            hex(&mut levels, level);
            hex(&mut land_types, land);
            hex(&mut slope_types, slope);
            hex(&mut deck_rises, deck);
        }
    }
    TerrainGrid {
        width,
        height,
        levels,
        land_types,
        slope_types,
        deck_rises,
    }
}

/// Project complete retained owners, not the dashboard's bounded collections.
/// Production admission stays authoritative; no resource binding or ticks occur.
fn inspect_save(path: &std::path::Path) -> Result<(), String> {
    use serde_json::json;
    use vera20k_core::sim::intern::InternedId;
    use vera20k_core::sim::snapshot::GameSnapshot;

    let bytes =
        std::fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let snapshot = GameSnapshot::load_with_resources(&bytes)
        .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
    let sim = &snapshot.sim;
    let interner = sim.interner();
    let strings: Vec<_> = (0..interner.len())
        .map(|index| interner.resolve(InternedId::from_index(index as u32)))
        .collect();
    let houses: Vec<_> = sim.houses().in_order().collect();
    let factories = sim.production().factories().iter_insertion_ordered();
    let entities: Vec<_> = sim
        .entities()
        .iter_sorted()
        .map(|(id, entity)| {
            json!({"id": id, "type": interner.resolve(entity.type_ref()),
            "owner": interner.resolve(entity.owner()), "retained": entity})
        })
        .collect();
    let teams = serde_json::to_value(sim.team_script_vm())
        .map_err(|error| format!("cannot project retained teams: {error}"))?;
    let report = json!({
        "inspection_schema": 1,
        "metadata": {"layout": format!("{:#010x}", snapshot.layout),
            "tick": snapshot.tick, "map_name": snapshot.map_name,
            "map_hash": format!("{:#018x}", snapshot.map_hash),
            "rules_hash": format!("{:#018x}", snapshot.rules_hash)},
        "coverage": {"houses": "all in registration order",
            "factories": "all in construction order", "entities": "all in stable-id order",
            "teams_and_definitions": "complete retained TeamScriptVm, including task forces and scripts",
            "interned_ids": "indices into strings",
            "interpretation": "point-in-time retained state; no AI-strength or progress claim",
            "load_policy": "normal production load admission and planning reset; no ticks advanced",
            "resources": "not rebound; derived caches and provider availability are not recomputed"},
        "strings": strings, "houses": houses, "factories": factories,
        "teams_and_definitions": teams, "entities": entities,
    });
    serde_json::to_writer_pretty(std::io::stdout().lock(), &report)
        .map_err(|error| format!("cannot write inspection: {error}"))?;
    println!();
    Ok(())
}

fn main() -> Result<(), String> {
    let args = match parse_args() {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    if [
        args.trace_save.is_some(),
        args.inspect_save.is_some(),
        args.inspect_map.is_some(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count()
        > 1
    {
        return Err(
            "save tracing, save inspection and map inspection are mutually exclusive".into(),
        );
    }
    if let Some(map) = &args.inspect_map {
        if args.ticks != 0
            || !args.observe_entities.is_empty()
            || !args.observe_teams.is_empty()
            || args.observe_cells.is_empty()
        {
            return Err("--inspect-map requires cells only and no advancing ticks".into());
        }
        let root = args
            .ra2_dir
            .as_deref()
            .ok_or("--inspect-map requires --ra2-dir")?;
        let scenario = vera20k_core::headless_scenario::load(root, map, args.seed)?;
        let sim = scenario.sim();
        let cells: Vec<_> = args
            .observe_cells
            .iter()
            .map(|&cell| sim.telemetry_cell_observation(cell))
            .collect();
        let report = serde_json::json!({"map_inspection_schema": 1,
            "metadata": {"map": map, "seed": args.seed,
                "dimensions": [sim.session().map_width(), sim.session().map_height()],
                "theater": scenario.map.header.theater, "tick": sim.session().tick(),
                "map_hash": format!("{:#018x}", scenario.map.ini.content_hash()),
                "rules_hash": format!("{:#018x}", scenario.runtime.resources.rules.simulation_config_hash())},
            "coverage": "production headless initial Battle local-only load; no frames, new orders, save writes or server; not the played 1v1 or native execution; compare connectivity, not zone-number identity across sessions",
            "cells": cells});
        serde_json::to_writer_pretty(std::io::stdout().lock(), &report)
            .map_err(|error| error.to_string())?;
        println!();
        return Ok(());
    }
    if args.trace_save.is_none()
        && (!args.observe_entities.is_empty()
            || !args.observe_teams.is_empty()
            || !args.observe_cells.is_empty())
    {
        return Err("observer arguments require --trace-save".into());
    }
    if let Some(path) = &args.trace_save {
        let root = args
            .ra2_dir
            .as_deref()
            .ok_or("--trace-save requires --ra2-dir")?;
        let ticks = if args.ticks == 0 {
            300
        } else {
            u32::try_from(args.ticks).map_err(|_| "trace ticks exceed u32")?
        };
        let report = vera20k::app::telemetry::trace_saved_game(
            path,
            root,
            ticks,
            &args.observe_entities,
            &args.observe_teams,
            &args.observe_cells,
        )?;
        serde_json::to_writer_pretty(std::io::stdout().lock(), &report)
            .map_err(|error| error.to_string())?;
        println!();
        return Ok(());
    }
    if let Some(path) = &args.inspect_save {
        return inspect_save(path);
    }
    vera20k::util::logging::init_stderr_logger("info").map_err(|error| error.to_string())?;

    let mut runtime = SimRuntime {
        simulation: build_world(args.seed, args.per_category, args.structures),
        resources: SimResources::empty(),
    };
    let spawned = runtime.simulation.telemetry_snapshot();
    println!(
        "synthetic world: {} entities, {} houses, map {}x{}",
        spawned.entities_total,
        spawned.houses.len(),
        spawned.map_width,
        spawned.map_height
    );

    let server = TelemetryServer::start(args.addr, args.web_root)
        .map_err(|error| format!("telemetry server on {}: {error}", args.addr))?;
    println!("dashboard: {}", server.bootstrap_url());
    println!("Ctrl+C to stop");

    // The map never changes in this demo, so build the elevation grid once and
    // hand it to each client that attaches, mirroring the app's one-shot send.
    let terrain = synthetic_terrain(spawned.map_width, spawned.map_height);
    let mut last_attached: u64 = 0;

    let mut ring = TickTimingRing::default();
    let mut last_publish = Instant::now();
    let mut tick: u64 = 0;
    let mut stroll_targets: HashMap<u64, (f32, f32)> = HashMap::new();
    let start_positions: Vec<(u64, f32, f32)> = runtime
        .simulation
        .entities()
        .iter_sorted()
        .map(|(id, entity)| {
            (
                id,
                f32::from(entity.position.rx()),
                f32::from(entity.position.ry()),
            )
        })
        .collect();

    loop {
        let frame_started = Instant::now();
        runtime.simulation.set_frame_profiling(server.has_clients());
        runtime
            .advance_frame_for_tooling(&[], SIM_TICK_MS)
            .map_err(|error| format!("frame {tick}: {error}"))?;
        stroll(&mut runtime.simulation, tick, &mut stroll_targets);
        ring.push(frame_started.elapsed().as_secs_f32() * 1000.0);
        tick += 1;

        if tick % PUBLISH_EVERY == 0 && server.has_clients() {
            let mut snapshot = runtime.simulation.telemetry_snapshot();
            // This artificial world keeps its engine houses passive to avoid
            // victory/defeat. Display them as the demo's two participants; the
            // actual app always publishes the authoritative passive flag.
            for house in &mut snapshot.houses {
                house.passive = false;
            }
            let now = Instant::now();
            let mut timing = ring.summarize(Some(SIM_TICK_MS as f32));
            // Headless: no render frames, so FPS stays 0. The tick rate is the
            // demo's real wall-clock pace.
            timing.tick_hz =
                PUBLISH_EVERY as f32 / now.duration_since(last_publish).as_secs_f32().max(1e-3);
            last_publish = now;
            snapshot.timing = Some(timing);
            // The app fills this section from live app state; this headless
            // demo reads the same facts off its own simulation so the
            // dashboard's header shows a real session instead of blanks.
            let sim = &runtime.simulation;
            snapshot.session = Some(SessionContext {
                map_name: sim.session().map_name().to_owned(),
                game_mode: sim.session().game_mode().as_str().to_owned(),
                game_speed: sim.session().game_options().game_speed.clamp(0, 6) as u8,
                local_owner: sim
                    .session()
                    .current_house()
                    .map(|owner| sim.interner().resolve(owner).to_owned()),
                local_outcome: sim
                    .session()
                    .current_house()
                    .and_then(|owner| sim.ready_outcome_for_owner(owner))
                    .map(|outcome| outcome.kind),
                // Mirrors the app's `BUILD_STAMP`; the crate path is private.
                build_stamp: concat!("VERA20K ", env!("CARGO_PKG_VERSION")).to_owned(),
            });
            snapshot.profile = runtime.simulation.frame_profile().report();
            // A client that attached since the last publish needs the elevation
            // grid once; every other publish leaves `terrain` as `None`.
            let attached = server.attach_generation();
            if attached != last_attached {
                last_attached = attached;
                snapshot.terrain = Some(terrain.clone());
            }
            match serde_json::to_string(&snapshot) {
                Ok(json) => server.publish(json),
                Err(error) => log::warn!("snapshot serialization failed: {error}"),
            }
        }

        // A one-shot movement probe: after a few seconds, report how many
        // entities left their spawn cell, so a frozen world is visible.
        if tick == PROBE_AT {
            let moved = start_positions
                .iter()
                .filter(|(id, x, y)| {
                    runtime
                        .simulation
                        .entities()
                        .get(*id)
                        .is_some_and(|entity| {
                            (f32::from(entity.position.rx()) - x).abs() > 0.5
                                || (f32::from(entity.position.ry()) - y).abs() > 0.5
                        })
                })
                .count();
            println!(
                "after {} ticks: {moved}/{} entities moved (structures stay put by design)",
                tick,
                start_positions.len()
            );
        }

        let elapsed = frame_started.elapsed();
        if let Some(remaining) = Duration::from_millis(u64::from(SIM_TICK_MS)).checked_sub(elapsed)
        {
            thread::sleep(remaining);
        }

        if args.ticks != 0 && tick >= args.ticks {
            let timing = ring.summarize(Some(SIM_TICK_MS as f32));
            println!(
                "stopped after {tick} ticks: p50 {:.2} ms, p99 {:.2} ms, budget use {:.0}%",
                timing.p50_ms,
                timing.p99_ms,
                timing.budget_utilization.unwrap_or(0.0) * 100.0
            );
            return Ok(());
        }
    }
}
