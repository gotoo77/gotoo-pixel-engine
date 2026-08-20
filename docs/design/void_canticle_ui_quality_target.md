# Void Canticle — UI / Presentation Quality Target

## Status

Design target for upcoming Void Canticle presentation lots.

The current functional menus are intentionally not considered final art. The target quality is a premium, authored pixel-art / retro-futurist gothic UI with the same level of visual care as the gameplay presentation-space pass.

## Reference direction

The reference screens supplied during VC2.7 development establish the target for:

- exosuit / chassis selection;
- level-up choices;
- mutation choices;
- support augment choices;
- later build / codex / pause presentation where applicable.

The important point is not to reproduce any one mock-up literally. The target is the perceived production value: hierarchy, framing, illustration, iconography, semantic color, depth, and animation.

## Visual language

### Shared foundation

- full presentation-space rendering, not enlarged 180x320 menu rectangles;
- dark Grave Orbit / starfield background remains visible and participates in composition;
- ornate sci-fi / liturgical frames built from fine 1 px details;
- restrained glow instead of large opaque blocks;
- clear visual hierarchy between title, category, choice name, effect, stats and controls;
- selection must feel illuminated / energized rather than merely inverted or boxed;
- small ambient particles, glyphs and moving accents are acceptable when they do not compete with readability;
- UI colors retain semantic meaning across screens.

### Semantic palette

- cyan / cold blue: level-up, conventional upgrade, information, shield / mobility;
- magenta / crimson: mutation, dangerous evolution, Void-influenced build changes;
- gold / amber: hull, relic, exosuit structure, rare / important identity accents;
- violet: Void / Canticle / synergy / supernatural linkage;
- near-black / navy: structural background and negative space.

## Chassis selection target

The chassis selector should ultimately behave like a showcase screen, not a text menu.

Each chassis card should expose:

- large dedicated ship illustration;
- chassis name and role;
- Hull / Shield / Move statistics with compact icons;
- passive name and short mechanical description;
- clearly visible active selection state;
- consistent navigation help.

Bulwark, Pilgrim and Wraith must look like distinct products / vessels, not recolors of one base sprite.

Desired feeling: choosing the run's identity before launch.

## Level-up target

Each offered upgrade should eventually have:

- dedicated icon / micro-illustration;
- upgrade name;
- exact mechanical effect;
- semantic category color;
- strong selected state;
- optional contextual detail / tooltip affordance.

Examples already present in the design language:

- Magnet Field -> magnet / field icon;
- Rapid Fire -> projectile fan / firing icon;
- Vital Spark -> reinforced living core / Hull icon;
- Stellar Power -> radiant weapon core;
- Core Surge -> Canticle core charge motif.

The player should be able to recognize familiar upgrades from shape and color before reading the text.

## Mutation target

Mutation choices should visually feel more dangerous and transformative than ordinary level-ups.

Requirements:

- separate magenta / crimson framing language;
- stronger energy / corruption accents;
- distinctive icons for Split Volley, Piercing Lance, Orbitals, Death Nova, etc.;
- no ambiguity between a normal upgrade and a build mutation;
- selected mutation should feel like a consequential evolution, not a generic menu row.

## Support augment target

Support modules should use their own identity while remaining compatible with the shared system.

Examples:

- Nanite Repair -> repair swarm / medical-mechanical motif;
- Shield Capacitor -> charged shield cell / capacitor motif.

A synergy banner may bridge mutation and support choices when the current build creates a named combination.

## Information architecture

Do not put every system into one giant modal.

Prefer:

1. title / current context;
2. 2-3 highly readable choices;
3. concise mechanical copy;
4. optional secondary details;
5. navigation instructions isolated from gameplay information.

Avoid permanent instructional clutter once controls are learned.

## Animation target

Menus should not be static screenshots, but animation must remain subtle:

- selected frame breathing / pulsing;
- small icon animation;
- starfield / glyph drift;
- energy travelling through frame ornaments;
- short transition when selection changes;
- confirmation burst / collapse into gameplay.

No continuous large-scale motion that makes text harder to read.

## Planned lots

### UI-1 — Chassis showcase

Replace the current V22-style text list with a presentation-space chassis selector using dedicated visual silhouettes, stats and passive modules.

Success criterion: Bulwark / Pilgrim / Wraith can be distinguished instantly without reading their names.

### UI-2 — Upgrade cards

Rebuild LEVEL UP in presentation space with dedicated icons, richer card frames and exact effect copy.

Success criterion: recurring upgrades are recognizable visually and the screen feels consistent with the HD gameplay art.

### UI-3 — Mutation + Support identity

Give MUTATION and SUPPORT AUGMENT distinct semantic presentation while sharing layout primitives with level-up where useful.

Success criterion: the player can identify which class of decision is being made before reading the title.

### UI-4 — Synergy / transition polish

Add named synergy presentation, lightweight transitions and confirmation feedback after the core layouts are stable.

## Architectural guardrail

Do not respond to this target by creating a generic GPE UI framework first.

Implement the screens locally in Void Canticle using the existing presentation-space renderer. Extract a reusable GPE primitive only after at least one additional real consumer demonstrates the same need.

Likewise, do not add another Vxx wrapper merely for visual polish. Continue evolving the current VC2.7 presentation ownership model.
