# NPatch Temporal Gunner Controls

This profile extends the existing `ifv_turret_switching` fixture rather than
reimplementing GetROF, manager transfer or gunner selection. It executes the
AYMOD NPAE174 instructions with the same physical retail FV/CLEG inputs used
by the stock regression. It is not a whole AYMOD game or boarding comparison.
The separate real-AY TTNK test covers its actual firing/command/save chain.

From the prepared checkout in PowerShell:

```powershell
$env:VERA20K_GAMEMD_EXE='D:\Workshop\CnC2Yuri-AYMOD\gamemd.exe'
$env:VERA20K_SHRAPNEL_INPUTS=(Resolve-Path ini).Path
$env:VERA20K_IFV_MAP='D:\Workshop\CnC2Yuri-AYMOD\重返迪拜.yrm'
python -m tools.spatial_oracle.ifv_turret_switching --temporal-gunner-npatch --check
```

Use `--write` only to regenerate measured outputs. The `ini/` directory must
contain the extracted retail RULESMD/ARTMD/MPBattleMD files; LANGRULE is optional.
The map is a physical INI layer. Consumed final type, weapon, difficulty and
country values are asserted through the production retail reader.

The three cases are active rookie, signed burst-index overflow, and the actual
post-shot FireAt rearm boundary followed by callbacks. They retain only logical
inputs and asserted state/RNG outputs, not call/write traces or register dumps.
Main and MapGen are measured unchanged and reference their full initial states;
Scenario retains its complete state and ordered raw draws after every callback.
The stock twelve-case corpus remains unchanged and is consumed without NPatch.

Native initialization follows startup52BA82/52BB02/52BB07: EBX0, real B97000
allocation and B99000 constructor. Original B98000 reads each bounded physical
layer from EDI. The shared allocator transport also handles CRT malloc7C9430;
original strdup, strlen, size arithmetic and strcpy still execute. No extension
scalar, ROF value, RNG result, manager operation or executable instruction is
replaced. Full registry discovery and unrelated file/visual services retain the
existing fixture boundaries described in the sidecar.

Rust consumes the cases in `passenger::gunner_tests::temporal_oracle`. Its FireAt
fragment is imported as an existing measured boundary before subsequent
callbacks, not counted as a Rust whole-FireAt comparison. These controls do not
establish every rank/House combination or unrepresented multi-manager topology.
