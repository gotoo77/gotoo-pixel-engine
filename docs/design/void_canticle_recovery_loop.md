# Void Canticle — Recovery Loop / Forced Landing

Status: roadmap concept, post-VC2 combat stabilization.

## Intent

A failed flight stage does not always have to mean an immediate hard reset.
Void Canticle can use selected defeats to branch the run into a different,
short gameplay phase: the Pilgrim loses control of the exosuit/reliquary,
performs an emergency descent toward the nearest reachable body, survives a
forced landing, then has to recover enough capability to leave again.

The goal is **fail-forward**, not consequence-free failure.

A defeat can therefore create a story and a tactical problem instead of only
showing `GAME OVER`.

## Proposed transition

```text
flight combat
    |
    | Hull reaches zero
    v
critical failure
    |
    +--> no viable refuge --> run ends
    |
    +--> refuge available
            |
            v
      emergency descent
            |
            v
       forced landing
            |
            v
       recovery phase
            |
            v
      repair / rebuild
            |
            v
       relaunch choice
            |
            v
       altered route
```

## Cinematic language

The transition should remain short and gameplay-oriented rather than becoming
a long non-interactive cutscene.

Possible sequence:

1. shield collapse and Hull critical warning;
2. propulsion/aspect-control failure;
3. nearest viable planet, moon, wreck or megastructure is acquired;
4. player regains limited steering for an emergency descent;
5. impact / black screen / recovery wake-up;
6. switch to the recovery gameplay ruleset.

Ideally the player keeps a little agency during descent. A good emergency
landing may preserve more equipment or determine the landing site.

## Recovery gameplay

The first prototype must be small: roughly 60–120 seconds, not a second full
RPG built before the shmup is stable.

Candidate verbs:

- explore a compact local area;
- salvage repair material;
- choose which subsystem to restore first;
- trade Hull repair against Shield, weapon or mobility repair;
- recover a lost relic or abandon it;
- decide when the exosuit is safe enough to relaunch;
- inspect the next route before take-off.

This phase can later support resource management, route planning, local NPCs,
environmental hazards and narrative discoveries.

## Consequences

Forced landing must have a cost, otherwise dying becomes an optimal strategy.
Possible costs are deliberately systemic rather than arbitrary:

- damaged maximum Hull until a real repair is found;
- temporarily reduced Shield capacity;
- damaged mobility actuator;
- one lost mutation/relic that can potentially be recovered;
- spent salvage or Cinders;
- route deviation;
- increased Void pressure;
- optional scar/modifier carried for the rest of the run.

A recovery phase can therefore produce a *different* run rather than merely
restore the exact pre-death state.

## Relationship with exosuit chassis

The future chassis triangle (mobility / protection / power budget) naturally
feeds this loop.

Examples:

- a heavy chassis survives atmospheric impact better but is harder to steer;
- a light chassis reaches better landing sites but suffers more structural
  damage;
- a high-energy chassis can perform a controlled burn at the cost of draining
  its Shield reserve.

This makes chassis selection matter outside direct bullet dodging without
inventing unrelated statistics.

## Failure policy

Not every death should force a recovery phase.

A stage/sector can explicitly expose one of these outcomes:

- `Terminal`: no refuge exists; the run ends.
- `Recoverable`: a forced landing route exists.
- `Conditional`: recovery requires fuel, a navigation module, a discovered
  refuge, a surviving subsystem, or another readable condition.

The player should understand why recovery was or was not available.

## Arcade escape hatch

Keep a fast restart path. A player who only wants the shmup loop must not be
forced through a two-minute recovery scene on every failure.

A future result screen can therefore offer, when appropriate:

```text
FORCED LANDING
QUICK RESTART
END RUN
```

The exact wording and controls remain to be designed.

## Roadmap placement

### VC2.1 — Combat model and stabilization

- player Hull + Shield;
- Shield regeneration;
- semantic impact/death feedback;
- Elite implosion/nova vertical slice;
- projectile/audio/end-of-stage stabilization.

### VC2.2 — Exosuit chassis / manoeuvrability

- explicit mobility model;
- acceleration / braking / max speed / focus speed;
- first heavy, balanced and light chassis;
- readable mobility / protection / power trade-off.

### VC2.3 — Damage and sustain vocabulary

- EMP / Shield disruption;
- armor penetration;
- repair-per-second;
- life/shield steal where it creates real decisions;
- first damage-type interactions.

### Later vertical slice — Forced Landing

Build one authored failure route from a flight stage into one tiny recovery
area and back to flight.

Success criterion:

> losing a fight creates a short, understandable and enjoyable new tactical
> situation, while still feeling meaningfully worse than winning the fight.

Only after that vertical slice is fun should exploration/resource systems be
generalized or expanded.

## Architectural guardrail

Do not build a generic campaign/event/ECS framework first.

The first slice should prove only this round trip:

```text
VC flight state
  -> recoverable defeat
  -> authored landing transition
  -> compact recovery state
  -> explicit relaunch result
  -> VC flight state
```

If that round trip proves useful for multiple stages or games, extract the
smallest reusable GPE boundary afterwards.
