# Void Canticle — Game Design V0

> **Projet :** Gotoo Pixel Engine (GPE)  
> **Type :** shmup vertical / bullet hell accessible  
> **Univers :** dark science-fantasy, métal, cosmique  
> **Référence de départ :** `hectic-rs` comme source d'étude mécanique et architecturale, sans reprise de code ni d'identité visuelle  
> **Statut :** vision V0 — document de travail

---

## 1. High concept

**Void Canticle** est un shoot'em up vertical en pixel art dans lequel un pèlerin en armure-reliquaire traverse les ruines d'un cosmos où les étoiles sont des divinités-machines autrefois asservies.

Une étoile ancienne, **NEMESIS**, s'est réveillée et émet un chant capable de transformer simultanément la matière, les machines et les organismes vivants : **le Cantique du Vide**.

Le joueur remonte vers sa source à travers des cimetières orbitaux, des processions mécaniques, des nébuleuses organiques et des cathédrales stellaires, jusqu'à ne plus savoir s'il traverse une machine, une créature ou un dieu.

Le jeu vise une sensation immédiatement lisible de shmup classique — déplacement, tir, vagues, scrolling vertical, boss — enrichie progressivement par des patterns plus spectaculaires, du sound design, du polishing et une identité visuelle très forte.

L'objectif n'est pas de produire immédiatement un danmaku extrême. Void Canticle doit rester **accessible, lisible et satisfaisant**, tout en offrant des moments de densité et de spectacle.

---

## 2. Promesse joueur

Le joueur doit ressentir :

- la vulnérabilité d'un petit personnage perdu dans des structures cosmiques immenses ;
- la puissance croissante d'un arsenal capable de découper des essaims entiers ;
- la satisfaction de lire puis traverser un pattern de projectiles ;
- le plaisir immédiat du feedback audiovisuel : impacts, flashes, particules, explosions, musique et basses ;
- une montée en étrangeté constante à mesure que l'on approche de NEMESIS ;
- des boss qui ne sont pas seulement des sacs à PV mais de véritables spectacles mémorables.

Le ton doit évoquer un mélange de :

- science-fiction cosmique ;
- fantasy gothique ;
- iconographie religieuse détournée ;
- biomécanique ;
- métal ;
- ruines technologiques monumentales.

Le jeu ne doit toutefois devenir ni illisible ni gratuitement macabre : la DA doit rester **stylisée, élégante et iconique**.

---

## 3. Le monde

### 3.1 Les Forgerons d'Astre

Une civilisation disparue, aujourd'hui connue sous le nom de **Forgerons d'Astre**, découvrit que les étoiles n'étaient pas de simples boules de plasma mais des formes de vie cosmiques conscientes.

Les Forgerons apprirent à :

- les capturer ;
- canaliser leur énergie ;
- construire autour d'elles des architectures gigantesques ;
- transformer leurs émissions en énergie, calcul et propulsion ;
- enfermer des fragments stellaires dans des reliquaires artificiels.

Ils bâtirent ainsi un empire dont les cathédrales, les vaisseaux et les armes fonctionnaient littéralement grâce à des morceaux d'étoiles vivantes.

Puis les étoiles commencèrent à chanter.

### 3.2 Le Cantique du Vide

Le **Cantique du Vide** est une vibration cosmique qui agit sur plusieurs niveaux à la fois.

Il peut :

- reprogrammer une machine ;
- modifier la structure d'un organisme ;
- fusionner métal et chair ;
- affecter la perception ;
- synchroniser des entités séparées ;
- créer des formes géométriques impossibles.

Les survivants l'ont interprété comme une corruption.

Mais l'histoire laisse volontairement planer un doute :

> les étoiles sont-elles devenues folles, ou ont-elles simplement commencé à se libérer ?

### 3.3 NEMESIS

NEMESIS est l'une des plus anciennes étoiles captives.

Après des millénaires de silence, elle vient de s'éveiller.

Une transmission unique traverse le cosmos :

> **COME HOME**

Le message est incompréhensible pour tous sauf pour l'armure du protagoniste.

---

## 4. Le protagoniste : Le Pèlerin

Le joueur contrôle **le Pèlerin**, une silhouette humanoïde enfermée dans une petite armure-reliquaire volante.

Son apparence doit mélanger :

- chevalier ;
- astronaute ;
- ange mécanique ;
- exosquelette ;
- reliquaire gothique.

Le personnage doit rester petit à l'écran afin de maximiser la lisibilité des projectiles et le contraste avec les décors monumentaux.

### 4.1 La Braise Stellaire

Le cœur de l'armure contient une **Braise Stellaire**, un fragment vivant d'une étoile ancienne.

Cette Braise permet :

- de propulser l'armure ;
- d'alimenter l'arme principale ;
- d'absorber des fragments stellaires libérés par les ennemis ;
- de résister partiellement au Cantique ;
- de déclencher une onde de puissance extrême.

Narrativement, cette Braise constitue aussi le principal paradoxe du héros :

> pour combattre les conséquences de l'asservissement des étoiles, il utilise lui-même un morceau d'étoile captive.

---

## 5. Boucle de gameplay fondamentale

La boucle de base doit rester immédiatement compréhensible :

```text
se déplacer
    ↓
tirer
    ↓
lire les trajectoires ennemies
    ↓
éviter les patterns
    ↓
détruire les ennemis
    ↓
collecter les Cendres
    ↓
charger le Cœur Stellaire
    ↓
utiliser la puissance au bon moment
    ↓
affronter le boss
```

Le joueur doit pouvoir s'amuser dès la première minute sans connaître le lore ni comprendre les systèmes avancés.

---

## 6. Contrôles V0

### Déplacement

- 8 directions ;
- clavier ;
- gamepad ;
- touch à terme sur Web/mobile.

### Tir principal

Tir continu ou répétable très rapidement.

Le joueur ne doit pas avoir à marteler physiquement le bouton.

### Focus / Slow Movement

Une touche réduit la vitesse de déplacement.

But :

- traverser précisément un pattern dense ;
- offrir un contrôle fin ;
- créer une différence de sensation entre navigation générale et esquive précise.

Pendant le focus, la hitbox du joueur peut devenir visible ou mieux signalée.

### Canticle

Une action spéciale consomme une jauge pleine et déclenche une grande onde circulaire.

Effets envisagés :

- destruction ou neutralisation des projectiles proches ;
- dégâts importants ;
- flash lumineux ;
- particules ;
- légère pause d'impact ;
- screenshake contrôlé ;
- énorme signature sonore.

Le Canticle est l'équivalent fonctionnel de la bombe d'un shmup, mais doit posséder une identité propre.

---

## 7. Cendres et Cœur Stellaire

Les ennemis peuvent libérer des **Cendres Stellaires**.

Elles servent à charger une jauge : **le Cœur Stellaire**.

### V0

- petite Cendre : faible charge ;
- grande Cendre : charge importante ;
- jauge pleine : autorise un Canticle ;
- Canticle : remet la jauge à zéro.

### Évolutions possibles, non validées

À ne pas implémenter avant besoin démontré :

- plusieurs niveaux de charge ;
- choix entre bombe et attaque concentrée ;
- scoring basé sur la conservation de charge ;
- aimantation des Cendres ;
- bonus de collecte rapprochée.

---

## 8. Structure de campagne envisagée

La campagne complète idéale comporte cinq zones.

La V0 jouable n'a pas besoin de toutes les produire immédiatement.

---

## 9. Stage I — THE GRAVE ORBIT

### Concept

Le Pèlerin approche de NEMESIS en traversant un immense cimetière orbital.

Des milliers de vaisseaux morts tournent encore autour de l'étoile.

### Décor

- carcasses de vaisseaux ;
- satellites cruciformes ;
- antennes cassées ;
- fragments de stations ;
- cercueils orbitaux ;
- lumières lointaines ;
- silhouettes d'épaves gigantesques en parallaxe.

### Palette / sensation

Froid, sombre, métallique, avec quelques sources lumineuses très contrastées.

### Ennemis

#### Carrion Drone

Petit drone-charognard.

- faible résistance ;
- arrive en essaim ;
- trajectoire simple ;
- idéal pour introduire le tir.

#### Grave Knight

Chasseur blindé en forme de chevalier funéraire.

- entre à l'écran ;
- verrouille une direction ;
- charge en ligne droite ;
- repart hors écran.

#### Bell Wraith

Machine spectrale dotée d'une cloche.

- tire peu directement ;
- produit une onde qui affecte ou réoriente des projectiles déjà présents.

À garder éventuellement pour une version post-V0 si cela demande trop de moteur.

### Boss — THE BELLKEEPER

Une gigantesque cloche spatiale vivante entourée de bras mécaniques.

#### Signature

Chaque coup de cloche déclenche un pattern radial.

#### Phases possibles

1. anneaux simples ;
2. anneaux alternés + tirs ciblés ;
3. bras détruisibles ;
4. cloche fissurée, cadence accélérée.

#### Mise en scène

- arrivée lente ;
- musique qui se retire ;
- premier coup de cloche isolé ;
- vibration visuelle ;
- apparition du nom :

```text
THE BELLKEEPER
```

---

## 10. Stage II — IRON PILGRIMAGE

### Concept

Une armée automatisée effectue éternellement une procession vers NEMESIS.

### Décor

- ponts mécaniques ;
- statues ;
- bannières métalliques ;
- chaînes ;
- encensoirs géants ;
- colonnes de machines avançant vers la lumière.

### Ennemis

#### Procession Drone

Avance en formation stricte.

#### Iron Seraph

- entre verticalement ;
- déploie ses ailes ;
- tire deux arcs symétriques.

#### Choir Node

- renforce les ennemis proches ;
- encourage le joueur à casser une formation plutôt qu'à simplement tirer sur la cible la plus proche.

### Boss — THE PROCESSION KING

Gigantesque chevalier-cathédrale.

Ses patterns doivent évoquer :

- roues ;
- rosaces ;
- mandalas ;
- formations militaires.

---

## 11. Stage III — THE FLESH NEBULA

### Concept

La frontière entre technologie et biologie disparaît.

### Décor

- astéroïdes organiques ;
- tissus semi-transparents ;
- veines lumineuses ;
- structures osseuses ;
- machines parasitées ;
- yeux ou organes abstraits.

### Ennemis

#### Stellar Ray

Créature large qui traverse l'écran sur une courbe douce.

#### Void Leech

- absorbe des projectiles ennemis voisins ;
- grossit ;
- explose en pattern radial si détruite.

#### Armoured Embryo

- lent ;
- résistant ;
- s'ouvre périodiquement pour tirer.

### Boss — MOTHER APOCRYPHA

Créature monumentale évoquant une Madone cosmique.

Son ventre contient une singularité.

La silhouette doit être belle avant d'être grotesque.

---

## 12. Stage IV — CATHEDRAL OF THE BLACK SUN

### Concept

Le Pèlerin entre dans la mégastructure construite autour de NEMESIS.

### Décor

- vitraux dans le vide ;
- tuyaux d'orgue de plusieurs kilomètres ;
- engrenages ;
- statues d'anges sans visage ;
- arcs gothiques ;
- lumière noire et dorée ;
- structures qui semblent à la fois mécaniques et religieuses.

### Ennemis

#### Cherub Drone

Arrive en groupe et forme une figure géométrique avant de tirer.

#### Mechanical Gargoyle

S'accroche au décor puis plonge sur le joueur.

#### Choir Engine

Générateur stationnaire produisant plusieurs petits drones synchronisés.

### Boss — THE SEVEN-WINGED ARCHON

Ange mécanique monumental doté de sept ailes.

Chaque aile agit comme générateur de projectiles.

Le joueur peut perdre progressivement les ailes du boss :

```text
7 → 6 → 5 → 4 → 3 → 2 → 1
```

La destruction d'une aile modifie la géométrie globale du pattern.

L'objectif est d'obtenir un boss immédiatement mémorisable visuellement et mécaniquement.

---

## 13. Stage V — NEMESIS / THE LIVING STAR

### Concept

Le joueur atteint l'intérieur de l'étoile.

Le décor cesse progressivement d'être interprétable.

Il pourrait ressembler simultanément à :

- un réseau neuronal ;
- une constellation ;
- un circuit ;
- un organisme ;
- une architecture sacrée.

### Ennemis

Les ennemis deviennent plus abstraits :

- polyèdres lumineux ;
- silhouettes humanoïdes ;
- copies déformées du Pèlerin ;
- constellations mouvantes ;
- formes synchronisées par le Cantique.

### Boss final — THE CROWNLESS SUN

Le dernier adversaire n'est pas réellement un gardien de NEMESIS.

Il est une manifestation de NEMESIS elle-même.

Le combat doit remettre en question l'interprétation morale donnée jusque-là au Cantique.

La conclusion exacte n'est pas figée en V0.

---

## 14. Langage de patterns

Void Canticle doit pouvoir exprimer facilement des patterns simples et composables.

Conceptuellement, on vise à terme quelque chose comme :

```text
AtPlayer
Arc
Circle
Burst
Spiral
Multiple
```

Exemple :

```text
Multiple
├── Arc gauche
├── Arc droite
└── tir ciblé
```

### Règle architecturale

Ne pas créer immédiatement un DSL ou framework généraliste.

Commencer par coder les patterns réellement nécessaires au premier stage et au premier boss.

On n'extrait une abstraction commune qu'après avoir au moins deux consommateurs réels ou une duplication suffisamment claire.

---

## 15. Direction artistique

### 15.1 Principes

La DA doit privilégier :

- silhouette ;
- contraste ;
- grands aplats ;
- décors monumentaux ;
- petites unités très lisibles ;
- projectiles immédiatement identifiables ;
- effets lumineux contrôlés ;
- mélange gothique / biomécanique / cosmique.

### 15.2 Rapport d'échelle

Principe central :

> petit héros, monde immense.

Le Pèlerin occupe peu de pixels.

Les boss et certains éléments de décor peuvent dépasser largement la taille de l'écran logique.

### 15.3 Typographie

Les menus doivent avoir une forte identité.

Direction envisagée :

- grande typographie blackletter / gothique pour titres et boss ;
- petite police pixel lisible pour HUD et options ;
- contraste très net entre monumental et fonctionnel.

Il faudra utiliser des polices compatibles avec la licence choisie pour le projet.

### 15.4 Palette

Pas de palette unique imposée à ce stade.

Chaque stage peut disposer d'une dominante claire :

- Grave Orbit : bleu/noir/acier ;
- Iron Pilgrimage : rouille/or sombre ;
- Flesh Nebula : pourpre/ivoire/noir ;
- Black Sun Cathedral : noir/or/blanc brûlé ;
- Nemesis : lumière stellaire, couleurs progressivement irréelles.

---

## 16. Audio et musique

L'audio est une composante fondamentale de Void Canticle, pas un embellissement tardif.

### 16.1 SFX minimum

- tir joueur ;
- impact ennemi ;
- destruction ennemi ;
- collecte de Cendre ;
- Canticle ;
- joueur touché ;
- mort joueur ;
- entrée boss ;
- changement de phase boss ;
- navigation menu ;
- validation / retour ;
- pause.

### 16.2 Principes de sound design

Éviter la fatigue auditive malgré un tir continu.

Techniques possibles :

- volume modéré sur le tir principal ;
- variations légères ;
- priorité sonore ;
- sons d'impact plus lisibles que le tir ;
- gros contraste dynamique pour le Canticle et les boss.

### 16.3 Musique

Direction :

```text
metal
+ chœurs
+ synthés
+ industriel
+ ambient cosmique
```

La musique n'a pas besoin d'être metal en permanence.

Progression envisagée :

- Stage I : ambient spatial + percussion sourde ;
- Stage II : martial / mécanique ;
- Stage III : organique / dissonant ;
- Stage IV : orgue + chœurs + metal ;
- Stage V : quasi liturgique puis mur sonore final.

Certaines attaques de boss peuvent être synchronisées rythmiquement, sans transformer le jeu en rhythm game.

---

## 17. Juice / polishing

Void Canticle doit être développé jouable d'abord, mais le genre dépend fortement du feedback.

Éléments de polishing envisagés :

- flashes d'impact ;
- explosions multi-frames ;
- particules ;
- trails de projectiles ;
- apparition spectaculaire des boss ;
- transitions musicales ;
- léger screenshake ;
- micro hit-stop sur événements importants ;
- changement visuel de phase boss ;
- destruction progressive de parties de boss ;
- ralentissement très bref lors d'un kill majeur ;
- overlay de pause sur jeu visible en arrière-plan.

Règle : chaque effet doit améliorer le feedback sans détériorer la lisibilité des patterns.

---

## 18. Score et progression

Pas nécessaire pour la première vertical slice.

À étudier ensuite :

- score par kill ;
- multiplicateur de combo ;
- bonus de proximité ;
- bonus de collecte ;
- no-hit ;
- clear de stage ;
- classement local ;
- replay déterministe éventuel.

Le jeu doit rester amusant sans dépendre du scoring.

---

## 19. Coop locale

La coop locale deux joueurs est souhaitée à terme.

Elle doit rester simple :

```text
PlayerId
+ mapping d'entrée
+ position
+ état individuel
```

Ne pas construire une infrastructure réseau ou multijoueur générale pour cela.

Questions à résoudre plus tard :

- Cendres partagées ou individuelles ?
- Canticle partagé ?
- revive ?
- scaling des boss ?

---

## 20. Ce que Void Canticle doit tester dans GPE

Void Canticle est un consommateur de GPE.

Le jeu doit donc révéler les besoins du moteur au lieu de servir de justification artificielle à des abstractions préconçues.

### Besoins probablement immédiats

- sprite rendering propre ;
- animation de sprites ;
- scrolling de backgrounds ;
- résolution logique portrait ;
- viewport / letterboxing ;
- nombreux projectiles ;
- collision AABB / point-like hitboxes ;
- input clavier/gamepad ;
- audio avec plusieurs SFX concurrents ;
- menus ;
- pause ;
- texte stylé ;
- chargement de niveaux / scènes simples.

### Besoins à mesurer avant abstraction

- sprite batching / instancing GPU ;
- spatial partitioning pour collisions ;
- système générique de particules ;
- DSL de bullet patterns ;
- asset atlas généré ;
- système générique de boss phases ;
- camera shake moteur ;
- scripting externe des stages.

### Règle

Pour chaque nouvelle abstraction moteur :

1. identifier le problème réel rencontré dans Void Canticle ou un autre jeu ;
2. implémenter la solution minimale ;
3. valider le comportement dans le jeu ;
4. généraliser seulement si un deuxième consommateur ou une duplication le justifie.

---

## 21. Vertical slice V0 proposée

Ne pas commencer par cinq stages.

La première cible doit être une tranche verticale complète : **Stage I — The Grave Orbit**.

### Contenu V0

- écran titre minimal ;
- lancement partie ;
- Pèlerin déplaçable ;
- tir principal ;
- focus / slow movement ;
- scrolling vertical ;
- deux ou trois types d'ennemis ;
- quelques trajectoires courbes ;
- tirs ennemis ;
- collisions ;
- vie du joueur ;
- Cendres ;
- jauge du Cœur Stellaire ;
- Canticle ;
- pause ;
- premier boss : The Bellkeeper ;
- SFX essentiels ;
- musique placeholder ou première ambiance ;
- victoire / mort / restart.

### Ce qui n'est pas requis pour V0

- cinq stages ;
- coop ;
- scoring avancé ;
- arbre d'améliorations ;
- narration complexe ;
- éditeur de niveaux ;
- DSL externe ;
- particules génériques sophistiquées ;
- système d'assets universel.

---

## 22. Milestones proposées

### VC0 — Skeleton

Objectif : vérifier que GPE supporte naturellement un shmup portrait.

- nouvelle démo / jeu Void Canticle ;
- viewport portrait ;
- joueur ;
- mouvement ;
- tir ;
- une cible ;
- collision ;
- natif + Web si possible dès le début.

### VC1 — Grave Orbit gameplay

- scrolling ;
- vagues ;
- trois ennemis ;
- trajectoires ;
- tirs ennemis ;
- focus ;
- vie ;
- Cendres ;
- Canticle.

### VC2 — Bellkeeper

- entrée boss ;
- barre de vie ;
- phases ;
- patterns radiaux ;
- victoire ;
- effets de destruction.

### VC3 — Audio + juice

- SFX complets ;
- musique ;
- impacts ;
- particules ;
- screenshake contrôlé ;
- flashes ;
- polish menu/pause.

### VC4 — Art pass

- sprites définitifs ou semi-définitifs ;
- animations ;
- background Grave Orbit ;
- UI ;
- typographie ;
- identité visuelle cohérente.

### VC5 — Vertical slice

Objectif : une version que l'on peut envoyer à quelqu'un en disant simplement :

> « joue à ça »

sans devoir expliquer qu'il s'agit d'un prototype technique.

---

## 23. Principes non négociables

### Lisibilité avant spectacle

Le joueur doit comprendre ce qui le tue.

### Gameplay avant lore

Le lore soutient l'expérience ; il ne doit pas bloquer le développement du jeu.

### Feedback audiovisuel fort

Void Canticle ne doit jamais donner l'impression d'un shmup silencieux ou sans impact.

### Pas de clone de Hectic

Nous reprenons des enseignements généraux :

- scrolling vertical ;
- vagues ;
- patterns ;
- boss ;
- structure simple ;
- identité visuelle assumée.

Mais Void Canticle doit disposer de ses propres :

- personnages ;
- univers ;
- ennemis ;
- patterns ;
- sprites ;
- décors ;
- musiques ;
- progression ;
- identité.

### No speculative abstraction

Le jeu doit pousser GPE à progresser par besoins concrets.

---

## 24. Identité résumée en une phrase

> **Un petit pèlerin portant une étoile captive traverse les cathédrales biomécaniques d'un cosmos en train de se réveiller, tandis que la frontière entre machine, chair et dieu disparaît autour de lui.**

---

## 25. North Star

Si la direction artistique et la technique atteignent le niveau visé, Void Canticle doit produire cette réaction :

> « Je comprends immédiatement comment jouer, mais je veux voir ce qu'il y a après. »

Le premier stage doit commencer comme un shmup vertical immédiatement familier.

Le dernier doit donner l'impression d'avoir voyagé beaucoup plus loin que la distance réellement parcourue à l'écran.

---

*Document V0 — 18 août 2026.*
