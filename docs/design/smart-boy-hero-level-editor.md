# Smart Boy Hero - Level Editor Draft

Date: 2026-08-18

Ce document propose une direction pour sortir SBH du cycle "petit changement de
niveau code a la main". L'objectif est que le design d'un niveau devienne une
operation d'edition et de playtest, pas une modification Rust.

## Objectif

Construire un editeur SBH utilisable par le designer pour:

- placer le heros, la sortie, murs, portes, plaques, pieges, bonus, ennemis,
  nourriture, pits et boulders;
- regler les proprietes des objets sans toucher au code;
- tester le niveau immediatement;
- exporter un fichier de niveau versionnable;
- charger ce fichier en natif et en Web.

## Decision Provisoire

Le bon premier socle est un format de niveau declaratif en JSON, puis un editeur
Web qui lit/ecrit ce format.

Raison: SBH a deja un world pur qui consomme une structure `Level`. Le point
dur n'est pas le rendu; c'est que les niveaux sont encore compiles dans
`examples/smart_boy_hero/world.rs`. Avant un bel editeur, il faut donc une
frontiere de donnees stable.

## Schema Cible

Exemple de niveau:

```json
{
  "id": "clockwork_keep",
  "name": "THE CLOCKWORK KEEP",
  "width": 26,
  "height": 18,
  "timing": "semi_continuous",
  "hero": { "x": 2, "y": 9, "power": 55 },
  "exit": { "x": 24, "y": 8, "kind": "checkpoint" },
  "walls": [[0, 0], [1, 0]],
  "doors": [
    { "x": 21, "y": 8, "group": 2, "initially_open": false, "role": "core_gate" }
  ],
  "plates": [
    { "x": 15, "y": 12, "group": 5, "kind": "pressure" }
  ],
  "traps": [
    { "x": 17, "y": 9, "group": 1, "initially_active": false }
  ],
  "boulders": [
    {
      "x": 14,
      "y": 8,
      "direction": "right",
      "group": 5,
      "corridor": [[15, 8], [16, 8], [17, 8], [18, 8], [19, 8], [20, 8]]
    }
  ],
  "enemies": [
    { "x": 17, "y": 8, "kind": "walker", "direction": "right", "power": 34, "role": "key_warden" }
  ],
  "bonuses": [
    { "x": 3, "y": 3, "kind": "fixed", "amount": 12 }
  ],
  "foods": [],
  "pits": []
}
```

La propriete `corridor` du boulder ne doit pas devenir une nouvelle regle de
simulation au depart. Elle sert d'aide d'edition: l'editeur affiche et valide le
couloir attendu, pendant que le vrai comportement reste determine par murs,
portes, boulders et limites de carte. Si un boulder sort du corridor annote,
l'editeur signale une incoherence de design.

## Editeur MVP

Premier MVP utile:

- canvas iso ou grille orthogonale simple avec overlay de coordonnees;
- palette d'outils: wall, hero, exit, door, plate, trap, boulder, enemy, bonus;
- panneau proprietes pour l'objet selectionne;
- selection d'un `group` par couleur pour portes, plaques, pieges et boulders;
- simulation locale du boulder selectionne pour previsualiser son trajet;
- bouton Playtest qui lance le niveau dans le vrai `SmartBoyWorld`;
- export JSON.

La grille orthogonale est acceptable pour le MVP. L'iso peut venir ensuite si la
lisibilite designer le justifie.

## Validations Necessaires

Validation structurelle:

- exactement un heros;
- exactement une sortie;
- toutes les cellules sont dans les bounds;
- pas de collision entre objets exclusifs;
- portes, plaques, pieges et boulders referencent des groupes valides;
- `core_gate` unique si le niveau utilise une cle coeur;
- au moins un `key_warden` si une `core_gate` existe.

Validation gameplay:

- la sortie est atteignable en theorie depuis le depart en ignorant les ennemis;
- une porte fermee sans plaque/levier lie est signalee;
- un boulder pret doit avoir une plaque/levier qui peut le liberer;
- un boulder avec corridor annote doit parcourir ce corridor dans la simulation;
- un corridor de mort doit contenir au moins une cible utile, par exemple ennemi
  ou interrupteur indirect;
- le niveau doit pouvoir atteindre `Phase::Won` dans un scenario de test connu.

## Integration

Etape 1: extraire un `LevelSpec` serialisable dans un module dedie, puis fournir
une conversion `LevelSpec -> Level`.

Etape 2: convertir l'iso slice actuelle en JSON charge par `include_str!`, tout
en gardant les niveaux historiques Rust tant que le format se stabilise.

Etape 3: ajouter des tests de parsing et de validation sur des fixtures JSON.

Etape 4: creer un editeur Web local qui modifie le JSON et appelle les memes
validateurs que le jeu.

Etape 5: ajouter le playtest direct: l'editeur injecte le `LevelSpec` courant
dans `SmartBoyWorld` sans rebuild quand on est en mode dev Web.

## Questions Ouvertes

- L'editeur doit-il vivre dans `web/` comme outil statique ou dans un exemple
  WASM separe?
- Les niveaux finaux doivent-ils rester embarques dans le binaire ou etre
  chargeables dynamiquement en Web?
- Le `corridor` du boulder doit-il rester une annotation de validation ou
  devenir un rail contraignant dans la simulation?
- Faut-il une notion explicite d'objectif, par exemple `reach_exit`,
  `collect_key_then_exit`, `kill_target`, ou la sortie unique suffit-elle?
