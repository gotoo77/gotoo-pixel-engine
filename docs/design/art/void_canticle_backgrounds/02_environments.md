# 1. Mini-bible DA — Background gameplay

## 1.1 Piliers visuels

### 1. Le vide comme matière

Le noir n’est pas un espace à remplir. Il fait partie de la composition.

Le background doit conserver de grandes zones sombres afin de créer :

- profondeur ;
- silence ;
- isolement ;
- contraste avec les tirs et les ennemis ;
- sensation d’échelle.

La cible n’est pas une nébuleuse multicolore permanente. L’espace doit rester majoritairement obscur.

### 2. Des objets rares mais monumentaux

Préférer quelques éléments gigantesques à une multitude de petits détails :

- planète morte ;
- singularité ;
- éclipse ;
- anneau brisé ;
- cathédrale orbitale ;
- monolithe ;
- station abandonnée ;
- épave colossale ;
- portail ;
- structure verticale inexplicable.

Un objet doit pouvoir suggérer qu’il existe très loin au-delà de l’aire de jeu.

### 3. Une technologie presque religieuse

Les environnements de VC doivent pouvoir évoquer simultanément :

- architecture ;
- machine ;
- relique ;
- temple ;
- observatoire ;
- artefact cosmique.

La géométrie peut rappeler des motifs rituels : cercles, alignements, aiguilles, anneaux, axes de lumière, répétitions symétriques.

Il faut éviter de rendre explicitement compréhensible la fonction de chaque structure.

### 4. Verticalité

Le jeu est conçu en **9:16 portrait**. La composition doit exploiter cette contrainte :

- spires ;
- colonnes de lumière ;
- couloirs cosmiques ;
- failles ;
- structures qui prolongent visuellement l’axe de déplacement ;
- objets majeurs placés dans le tiers supérieur ou sur les côtés.

La verticalité doit donner l’impression que le joueur descend ou monte à travers un espace sans limite.

### 5. Menace indirecte

Le décor n’a pas besoin d’attaquer constamment le joueur pour être hostile.

La menace peut venir de :

- l’échelle ;
- l’obscurité ;
- une anomalie immobile ;
- une structure qui semble observer le joueur ;
- une lumière qui ne devrait pas être là ;
- un énorme objet traversant lentement le champ ;
- un phénomène gravitationnel ;
- des débris attirés vers une singularité.

---

## 1.2 Palette

La palette générale de VC peut rester centrée sur :

- noir ;
- bleu nuit ;
- violet ;
- magenta ;
- cyan.

Mais les stages doivent avoir des identités chromatiques distinctes.

Palettes secondaires possibles :

- bleu froid ;
- vert toxique / énergétique ;
- rouge sombre / carmin ;
- noir / cendre / orange incandescent très discret ;
- violet d’éclipse.

### Règle de saturation

La saturation doit être concentrée dans les zones significatives :

- portail ;
- singularité ;
- fissure ;
- source d’énergie ;
- anneau ;
- horizon ;
- artefact.

Éviter une saturation uniforme du background.

---

## 1.3 Composition SHMUP

Une image spectaculaire n’est pas automatiquement un bon background de jeu.

### Combat corridor

Réserver approximativement le tiers central du 9:16 comme zone de combat à faible bruit visuel.

Le corridor peut contenir :

- brume légère ;
- étoiles rares ;
- gradients ;
- structures extrêmement lointaines ;
- textures très peu contrastées.

Il doit éviter :

- petits points blancs brillants ;
- détails très nets ;
- objets proches de la taille d’un projectile ;
- motifs qui ressemblent à des bullets ;
- forts contrastes locaux.

### Répartition recommandée

```text
┌─────────────────────────┐
│ landmark / spectacle    │
│ planète, portail, etc.  │
│                         │
│      ┌───────────┐      │
│      │           │      │
│      │  COMBAT   │      │
│      │ CORRIDOR  │      │
│      │           │      │
│      └───────────┘      │
│ structures latérales    │
│ et profondeur           │
└─────────────────────────┘
```

---

## 1.4 Règle de contraste

Bonne cible visuelle :

- **70 %** obscurité / noir ;
- **20 %** matière cosmique / structures ;
- **10 %** lumière spectaculaire.

Ce ratio n’est pas une contrainte mathématique stricte : c’est un garde-fou artistique.

### Règle critique

> **Aucun petit élément brillant et fortement contrasté dans la zone de combat s’il peut être confondu pendant 100 ms avec un projectile.**

---

## 1.5 Ce qu’il faut éviter

Éviter :

- les petites textures qui bouclent visiblement ;
- les wallpapers SF génériques ;
- les nébuleuses lumineuses partout ;
- les motifs répétitifs ;
- le bruit visuel uniforme ;
- les grosses structures très contrastées au centre ;
- les étoiles blanches de taille comparable aux tirs ;
- les backgrounds qui semblent appartenir à un autre jeu ;
- les effets permanents qui deviennent rapidement décoratifs.

---
