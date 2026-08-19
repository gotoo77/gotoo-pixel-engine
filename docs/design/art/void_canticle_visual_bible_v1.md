# Void Canticle — Visual Bible V1

Status: **VC1.8 art-production baseline**.

This document records the visual rules being validated directly in the playable Grave Orbit slice. It is not a generic GPE asset specification and it does not prescribe a future atlas/import pipeline.

## 1. North star

Void Canticle should read as **gothic reliquary technology in a hostile cosmic void**.

The screen may become spectacular and dense, but important gameplay families must remain identifiable without relying only on hue.

> Silhouette first, value/contrast second, color third, particles last.

The art must support the existing gameplay instead of hiding it.

## 2. Scale hierarchy

The player remains deliberately small. The world and its guardians should feel larger than the Pilgrim without consuming the entire readable playfield.

Current target hierarchy:

- pickup / projectile: 3–9 px visual core;
- Carrion / small drone: roughly 17 px silhouette;
- specialist / tactical threat: roughly 17–21 px silhouette;
- Bellkeeper: roughly 31 px authored core plus phase halo / appendages;
- later bosses may exceed this significantly when their gameplay justifies it.

Collision radii remain gameplay data. Art is allowed to extend beyond the collision core as wings, chains, halo, smoke, cloth or energy.

## 3. Material language

### Pilgrim / player technology

- warm bone/ivory metal;
- gold/brass reliquary accents;
- violet stellar technology;
- warm white projectile cores;
- controlled, intentional geometry.

### Grave Orbit machinery

- cold desaturated metal;
- dirty bone plating;
- rust-red wounds/eyes;
- near-black mechanical cavities;
- asymmetry and broken appendages.

### Void manifestations

- violet / magenta energy;
- cyan spectral highlights when the effect is informational or field-like;
- geometric halos, crosses, rings and impossible symmetry;
- effects should look less like conventional ammunition and more like a rule imposed on space.

## 4. Enemy silhouette rules

### Carrion Drone

Read: **wide scavenger insect / broken mechanical bird**.

- thin spread wings;
- compact red ocular core;
- dangling broken legs / antennae;
- fragile silhouette: should look disposable even before it dies quickly.

### Grave Knight

Read: **armoured falling knight / funeral interceptor**.

- strong vertical silhouette;
- helmet/crest at top;
- broad plated shoulders;
- two heavy lower propulsion points;
- should feel like a physical projectile when it begins its charge.

### Bell Wraith

Read: **floating bell apparition**.

- hollow central body;
- spectral tapered tail;
- circular/arched upper silhouette;
- halo animation is part of identity, not decoration.

### Relic Carrier

Read: **horizontal reliquary courier**.

- wider than tall;
- gold immediately signals reward value, but the horizontal courier silhouette must remain readable in monochrome;
- visible wake reinforces its escape direction.

### Choir Node

Read: **stationary liturgical transmitter**.

- cross/radial silhouette;
- cyan core;
- clear field/halo geometry;
- visual links to buffed units remain intentionally visible.

### Void Leech

Read: **spined absorber / hostile aperture**.

- wider irregular spikes than Choir Node;
- dark central cavity;
- charge pips expose absorbed-shot state;
- must never be mistaken for a passive pickup.

### Bellkeeper

Read: **living cathedral bell**.

- heavy central bell/reliquary mass;
- visible cavity/clapper/core;
- external phase language is more important than a simple palette swap.

Phase vocabulary:

- PROCESSION: chains / restrained outer ring;
- RESONANCE: expanding concentric geometry;
- FINAL TOLL: red-gold cross and unstable corona.

## 5. Projectile vocabulary

VC1.7 established the first projectile pass and VC1.8 keeps it as the baseline:

- Pilgrim main fire: warm authored bolts;
- heavy player shots: larger reliquary bolts;
- hostile fire: directional needle/missile heads, not anonymous circles;
- XP: cyan-white shards with attraction trails;
- orbitals: tiny liturgical relics.

Future projectile art should preserve source/readability before adding detail. A projectile family should ideally be identifiable by **shape + motion**, not only by RGB value.

## 6. Effects hierarchy

Effects should communicate importance:

1. muzzle / routine shot: tiny, short-lived;
2. hit: sparks;
3. kill: fragments + short burst;
4. specialist/threat kill: heavier fragmentation;
5. level/mutation/synergy: radial authored event;
6. Void Pressure transition: screen-space ritual event;
7. Canticle / boss phase / boss death: largest audiovisual punctuation.

Do not give every event the same particle density.

## 7. Animation principle

Small enemies do not need expensive full animation immediately. Prefer a few readable authored poses plus secondary motion:

- wing flex;
- halo pulse;
- dangling part movement;
- propulsion flare;
- charge pips;
- phase corona.

Animation must expose state whenever possible.

## 8. Current implementation boundary

VC1.8 deliberately keeps the art local to Void Canticle:

- authored pixel sprites are constructed through the existing GPE `Sprite` API;
- previous procedural shapes remain underneath as glow/aura where useful;
- no atlas manager;
- no generic animation system;
- no generic enemy renderer;
- no Aseprite importer yet.

A future external sprite-sheet pipeline should be introduced only after the playable game demonstrates a concrete need that the current authored-sprite approach cannot reasonably satisfy.

## 9. Acceptance criterion

A successful VC1.8 run should make a player able to answer, at a glance:

- which object is a Carrion vs a Grave Knight;
- which threat is a Choir Node vs a Void Leech;
- where the reward Carrier is going;
- which Bellkeeper phase is active;
- what belongs to the Pilgrim, the enemies, the Void and the loot layer.

The intended reaction is not merely “there are more pixels”. It is:

> **the battlefield has characters now.**
