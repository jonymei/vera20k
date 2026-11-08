# NPatch dual rearm kernel

Run against the AYMOD executable selected with `VERA20K_GAMEMD_EXE`:

```sh
python -m tools.spatial_oracle.techno_rearm --npatch --write
python -m tools.spatial_oracle.techno_rearm --npatch --check
```

The existing Fixture executes the selected GetROF at 0x006FCFA0, original
Scenario RNG, rank predicates and floating-point conversion. Only its virtual
class/weapon/type/garrison queries receive supplied fixture answers.

The six cases cover an explicit Unit burst interval, the default -1 random
interval, the last shot's full reload, and the removed Sonic/live-spark early
returns, plus the shared House/veterancy ROF scaling. Outputs retain full reload, short interval and RNG state before/after;
no native register dumps or call traces are fixtures. The default profile and
its existing stock corpus are unchanged. These are numeric-kernel controls,
not evidence for production Stop/Guard dispatch or complete FireAt scheduling.
The first input also executes the three original writer fragments: ordinary
normal and berserk6FF28F..6FF2BE, and Disk6FE4A4..6FE4C8. Their timer and
saved-delay outputs are consumed by the same Rust regression. Berserk CDQ
replaces EDX before the NPatch saved-delay write; Disk uses the full EAX.

In the selected NPatch body, 0x006FD07D/0x006FD095 preserve the short interval
while computing full ROF; 0x006FD0CB no longer adds the still-consumed jitter.
The return at 0x006FD1F5 enters 0x00C2C13D to select the short interval, using
full ROF when the saved interval is -1. The Sonic/particle shortcuts formerly
at 0x006FCFE0..0x006FD035 are removed. Cases avoid the building-ammo and empty
weapon early returns whose second result is not established.
