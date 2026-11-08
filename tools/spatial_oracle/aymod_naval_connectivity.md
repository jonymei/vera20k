# Return Dubai naval connectivity probe

Player report: a full five-member naval team remained in action53 before attacking.
This probe executes NPAE174 `MapClass::RebuildZoneConnectivity @ 0x56C510`
(including flood56CB90, bridge-pair56C6CB and all13 movement-row producers)
on the actual Return Dubai production retained class/height planes.
The Rust consumer is
`sim::team_script_vm::actions::tests::aymod_return_dubai_saved_naval_focus_fails_the_live_zone_precheck`.

Reproduce from the task checkout in a prepared native build environment:

```powershell
$env:RA2_DIR='D:\SteamLibrary\steamapps\common\Command & Conquer Red Alert II'
$env:VERA20K_AYMOD_DIR='D:\Workshop\CnC2Yuri-AYMOD'
$env:VERA20K_REQUIRE_RETAIL_INI='1'
$env:VERA20K_AYMOD_NAVIGATION_EXPORT='D:\Workshop\vera20k\target\aymod-save-analysis\dubai-retained-navigation.json'
python -m tools.cargo_run -- test -p vera20k-core --lib --no-default-features --features npatch aymod_return_dubai_saved_naval_focus_fails_the_live_zone_precheck -- --ignored
$env:VERA20K_GAMEMD_EXE='D:\Workshop\CnC2Yuri-AYMOD\gamemd.exe'
python -m tools.spatial_oracle.bridge_connectivity --retained-plane $env:VERA20K_AYMOD_NAVIGATION_EXPORT --output tools/spatial_oracle/aymod_naval_connectivity.json
```

The local full plane is not committed. The corpus retains only map/Size and
three raw Water-row10 query outputs. It is not a native scenario-load capture:
initial classes/heights come from production VERA, so their native generation
is still unverified. Empty bridge authority/records are required; bridge/tube
map support is not inferred. Native successful bounded allocation/free are the
only replaced callees; no connectivity answer or movement row is supplied.
Existing tiny-fixture direction startup and empty edge-bucket storage are reused.
The larger plane uses separate mapped storage without constructing native Cells;
empty records ensure this rebuild has no Cell/Tile dependency.

The result establishes the same Water2/4 separation when the original algorithm
receives these planes. It does not establish full FNPC, Foot admission, a complete
native game, or that the MOD author intended a permanently stalled team.
