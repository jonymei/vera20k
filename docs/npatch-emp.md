# NPatch EMP compatibility

This is VERA's player-facing NPatch compatibility design, not an instruction-for-
instruction port of NPAE #0011 Ext #174. It requires the `npatch` capability;
retail builds keep their existing behavior. It does not translate MOD files to
Ares syntax or enable unrelated Ares gameplay.

## Delivery and duration

`EMEffect=yes` uses the existing retained Warhead field. Each actual detonation
delivers EMP before ordinary damage. For zero-spread hits only the actual object
target is affected. Positive `CellSpread` uses the existing deterministic ground
collector and unique-object ordering. `AffectsAllies`, invulnerability and zero
armor interaction are honored.

Positive weapon Damage supplies the duration in simulation frames, clamped to
1..900. Zero Damage defaults to 90 frames; negative Damage gives a one-frame
pulse, not an undocumented repair operation. A repeated hit refreshes to the
greater of the remaining duration and the new pulse, rather than adding time.
These bounded defaults are explicit engineering choices, not asserted native
NPAE arithmetic. In a combined Ares/NPatch build an explicitly configured
nonzero `EMP.Duration` owns delivery instead, so no hit delivers twice.

Mechanical ground vehicles and powered buildings are susceptible. Ordinary
organic infantry/vehicles are immune; cyborg infantry may be disabled. Aircraft,
jumpjets and rockets are not forced out of flight. Falling objects, construction,
selling/unloading transitions and war-factory contacts are not interrupted.
Team membership, harvesters, mind control and transport managers do not silently
make ordinary mechanical targets immune.

The private `EmpState` remains the single counter/held-animation owner shared
with Ares. Existing Techno AI tails decrement it, the existing Deactivate and
Reactivate owners stop/resume locomotion, control, firing and deploy admission,
and the existing building operational edges handle powered defenses/providers.
Ownership changes and power-center recovery cannot bypass a live counter. On
expiry the owner reactivates only where power allows, retires the held sparkle
and returns computer-controlled ground victims to the existing Hunt mission.
Lifecycle pointer-expiry/destruction already retires the same held Anim; no new
timer, dead-object sweep or duplicate power state is introduced.

## AYMod's Tesla projectile

AYMod authors `TankBolt -> ElectricbounceT` (elite `TankBoltE -> ElectricbounceTE`)
with `Airburst=no`, `Cluster=1` and `AirburstWeapon=TankBoltEMP`. The child weapon
uses `EMPulsWH` with `EMEffect=yes`. Stock Airburst requires `Airburst=yes`, so it
previously discarded this producer before EMP delivery was possible.

Stock Rules processing also discovers that child only in the Bullet sweep,
after the Weapon sweep: a child referenced nowhere else retains constructor
fields until a later pass. NPatch compatibility eagerly discovers this impact
child while its parent's Weapon is being read. The existing Weapon factory and
same ordered sweep then read the new child, including its own Projectile and
Warhead. No second body reader or duplicate registry is introduced. This loader
ordering is also an explicit compatibility choice, not asserted NPAE chronology.

The NPatch compatibility policy emits one real child projectile after the
parent's shared impact receivers, preserving the live source and original target.
It uses the existing projectile allocation, launch, admission and collision
owners. It is not the stock nine-neighbor fan. Normal `Airburst=yes` behavior and
the existing Ares unsupported-fan boundary are unchanged; mixed builds accept
only this explicitly selected NPatch impact-child path.

## Validation boundary

Focused regressions cover the actual parent-impact/child-projectile/EMP chain,
refresh/expiry and ownership change, and organic/allied immunity plus powered-
tower disable/recovery/destruction. A whole AYMod battle, airborne EMP, precise
NPAE duration/mask semantics, all linked-building manager effects and combat
balance are not implied by these checks. This deliberately improves playable
coverage without claiming native parity for those mechanisms.
