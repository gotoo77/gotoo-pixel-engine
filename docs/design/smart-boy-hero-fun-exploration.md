# Smart Boy Hero - Fun Exploration

Date: 2026-08-16

Ce document conserve l'exploration de game design menée apres la V0 de Smart
Boy Hero. Il ne constitue pas une roadmap d'engagement. Les pistes listees ici
peuvent etre testees, abandonnees ou combinees plus tard.

## Diagnostic V0

Smart Boy Hero fonctionne techniquement: niveaux courts, Hero Power, Guards,
Walkers, bonus fixes, Mystery Bonus, portes, leviers, WAIT, sortie et restart.
Le probleme n'est plus la coherence minimale des regles.

Le coeur actuel reste cependant assez plat. Le joueur observe une grille,
calcule le bon ordre, puis execute. Les actions disponibles sont surtout:

- se deplacer;
- attendre;
- collecter automatiquement;
- combattre automatiquement;
- activer automatiquement un levier.

La plupart des objets existent donc seulement en relation avec le heros. Les
Guards taxent le Power du heros. Les bonus augmentent le Power du heros. Les
leviers sont actives par le heros. Les Walkers ajoutent du timing, mais leurs
effets systémiques restent limites: ils attaquent le heros, rebondissent, ou
sont detruits par le heros.

Les causes dominantes du manque de fun sont:

- expression faible: MOVE et WAIT creent peu d'intentions differentes;
- interactions pauvres: les entites agissent rarement les unes sur les autres;
- Power trop comptable: il sert surtout de seuil "passage ou mort".

La question centrale reste ouverte:

> SBH doit-il rester un puzzle de calcul ou devenir un jouet systemique minimal ?

## Forces a Preserver

- Micro-niveaux lisibles.
- Regle de combat simple: `Hero Power > Enemy Power`.
- Humour et codes absurdes de publicite mobile.
- Morts rapides et restart rapide.
- Grille petite, propice a l'experimentation.
- Walkers deja prometteurs parce qu'ils introduisent le temps.
- Feedbacks courts qui rendent les causes visibles sans ralentir le rythme.

## Principe Transversal: Audio Systemique

Le son ne doit pas etre traite seulement comme du polish tardif. Dans SBH, il
peut servir a comprendre qu'une regle vient d'en declencher une autre.

Statut 2026-08-16: hypothese en cours d'experimentation dans SBH avec des SFX
temporaires configures par JSON local. Le but reste de tester la lisibilite des
causalites, pas de figer une direction artistique sonore.

Note architecture: la passe actuelle utilise `include_dir` pour embarquer les
WAV references par `assets/smart_boy_hero/sfx.json`, puis alimenter `SoundBank`.
Ce choix est volontairement local et provisoire. Il rend les SFX robustes en
natif/Web et garde le mapping declaratif facile a remplacer, mais il impose un
rebuild pour tout changement d'asset et ne vise ni hot-swap, ni modding, ni
strategie generale d'assets GPE. La position actuelle est de conserver ce modele
pour SBH et de reevaluer une abstraction de resolution d'assets seulement quand
un autre consommateur concret, par exemple sprites, maps, dialogues ou musiques,
fera apparaitre le meme besoin cross-platform.

Exemples de feedbacks utiles:

- plaque activee: clic;
- porte ouverte: kchunk;
- Walker qui percute quelque chose: impact;
- bonus: pling;
- Mystery Bonus important: feedback plus excessif;
- piege arme: clic/tac mecanique;
- piege declenche: impact identifiable;
- mort: feedback immediat;
- victoire: feedback court.

Benefice potentiel: rendre les chaines causales plus lisibles et plus
satisfaisantes. Si un Walker active une plaque qui ouvre une porte, le joueur
doit pouvoir le percevoir comme une consequence, pas comme un hasard.

Interactions possibles:

- Plaques Vivantes: clic de plaque et son de porte differencies.
- Pieges: son distinct entre armement et declenchement.
- RNG a Choix: feedback plus expressif quand le hasard cree une situation.
- Collisions Ennemies: impact court pour confirmer la consequence.

Cout estime: faible pour des WAV temporaires synthetiques, moyen si une identite
sonore originale est construite plus tard avec enregistrements et nettoyage.

Risques:

- masquer une regle mal lisible au lieu de la clarifier;
- bruiter trop souvent les tours de Walkers;
- lancer trop tot un pipeline audio ou une abstraction moteur inutile.

Ordre de test suggere: apres une mecanique systemique minimale validee. GPE a
deja de l'audio one-shot, donc le premier test devrait rester local a SBH avec
des sons temporaires simples.

## Pistes de Gameplay

### 1. Shove

Principe: le heros peut pousser un ennemi adjacent d'une case si la case
derriere est libre. Variante possible: pousser coute du Power.

Potentiel de fun: eleve. Le joueur manipule le plateau, repositionne des Guards
ou des Walkers, et peut se sentir malin en liberant un passage autrement qu'en
payant le combat.

Interactions possibles:

- Guards: les deplacer hors d'un verrou.
- Walkers: modifier leur ligne ou leur timing.
- Power: cout de poussee, choix entre combattre et pousser.
- Portes/plaques: pousser une entite sur un declencheur.

Cout: moyen.

Risques:

- trivialiser les Guards si la poussee est gratuite;
- ajouter une ambiguite entre attaquer et pousser;
- transformer trop vite SBH en Sokoban si les niveaux abusent de caisses
  vivantes.

### 2. Collisions Ennemies

Principe: un Walker qui percute un Guard ou un autre ennemi applique une regle
lisible de collision, par exemple destruction du plus faible.

Potentiel de fun: tres eleve. Le joueur ne fait pas seulement survivre son
heros; il orchestre les dangers pour qu'ils se reglent entre eux.

Interactions possibles:

- Walkers contre Guards.
- WAIT pour synchroniser la collision.
- Power ennemi utilise comme force systemique, pas seulement comme taxe.
- Portes et murs pour contraindre les trajectoires.

Cout: moyen.

Risques:

- mauvaise lisibilite si les consequences de collision ne sont pas evidentes;
- niveaux trop scriptes si chaque collision a une seule fenetre;
- complexite supplementaire dans l'ordre de resolution des tours.

### 3. Plaques Vivantes

Principe: certains leviers deviennent des plaques de pression. Elles peuvent
etre activees par le heros ou par un Walker. Une porte liee a la plaque s'ouvre
pendant que la plaque est occupee. Les leviers classiques peuvent continuer a
ouvrir de facon permanente.

Potentiel de fun: tres eleve. Le joueur decouvre qu'une autre entite peut agir
sur le monde. WAIT devient un outil de synchronisation, pas seulement une pause.

Interactions possibles:

- Walkers: declencheurs mobiles et temporises.
- Portes: fenetres d'ouverture dynamiques.
- Leviers: alternative permanente par detour.
- Guards/Power: peuvent proteger une route ou rendre une alternative couteuse.

Cout: faible a moyen.

Risques:

- devenir un pur puzzle de timing si les solutions sont trop etroites;
- confusion entre levier permanent et plaque temporaire;
- besoin d'un rendu suffisamment clair pour distinguer les deux.

Statut 2026-08-16: une micro-experience locale SBH existe en niveaux 11-13.
Le signal recherche est positif si le joueur formule spontanement qu'un Walker
peut agir sur le monde. Cette experience ne suffit pas encore a trancher la
question centrale, mais elle justifie de tester une consequence plus forte que
l'ouverture de porte: un environnement qui peut neutraliser une menace.

### 4. Power Spend

Principe: le Power devient une ressource volontaire. Le joueur peut payer du
Power pour forcer une porte, pousser plus fort, survivre a une egalite ou
declencher une action speciale locale.

Potentiel de fun: moyen a eleve. Le Power cesse d'etre seulement un resultat
arithmetique et devient un budget tactique.

Interactions possibles:

- Portes: payer pour forcer un raccourci.
- Guards: choisir entre combat, contournement et depense.
- Bonus: arbitrer entre securite et economie.
- Shove: cout explicite de manipulation.

Cout: moyen.

Risques:

- rendre le jeu encore plus mathematique;
- multiplier les exceptions;
- affaiblir la lisibilite du combat `>`.

### 5. Bonus Deplacables

Principe: certains bonus peuvent etre pousses ou deplaces par des Walkers. Le
bonus n'est collecte que lorsque le heros entre dessus.

Potentiel de fun: moyen. Cela cree de la preparation et des detournements, mais
peut vite devenir laborieux.

Interactions possibles:

- Walkers qui transportent ou poussent un bonus.
- Shove pour repositionner une recompense.
- Power: prendre maintenant ou sauver le bonus pour plus tard.

Cout: moyen.

Risques:

- derive Sokoban;
- frustration si le bonus est pousse dans un coin;
- lisibilite faible si le joueur ne comprend pas pourquoi le bonus a bouge.

### 6. RNG a Choix

Principe: le hasard ne decide pas directement la victoire. Il cree une situation
nouvelle, puis le joueur choisit comment y repondre: deux bonus reveles, une
porte ouverte, un danger repositionne, une route sure ou risquee.

Potentiel de fun: moyen. Le hasard peut creer surprise et adaptation sans
injustice si la decision reste au joueur.

Interactions possibles:

- Mystery Bonus comme generateur de situation.
- Portes ou bonus alternatifs.
- Power comme marge de risque.
- Routes sures contre routes opportunistes.

Cout: faible a moyen.

Risques:

- mauvais si le jet dit simplement "tu gagnes" ou "tu perds";
- peut nuire au caractere puzzle si la reproductibilite est mal gereee;
- demande un feedback clair.

### 7. Fake Ad Deal

Principe: avant ou pendant un niveau, le jeu propose un contrat absurde de type
publicite mobile: "NO WAIT = +4", "NO GUARD KILL = DOOR", "UNDER 6 TURNS =
BONUS". Le joueur peut ignorer ou tenter le deal.

Potentiel de fun: eleve mais lateral. Cela cree de l'humour, de l'auto-defi et
des solutions personnelles sans changer tous les objets.

Interactions possibles:

- WAIT: interdire ou encourager l'attente.
- Power: bonus conditionnel.
- Guards: routes pacifistes ou agressives.
- RNG: deals tires ou proposes en choix.

Cout: moyen.

Risques:

- couche meta qui ne repare pas le coeur systemique;
- trop de texte ou d'UI;
- transformer le jeu en liste de challenges plutot qu'en jouet de plateau.

## Famille Candidate: Pieges

Les pieges forment une famille de mecaniques distincte des 7 pistes initiales.
Leur interet principal est d'introduire une relation:

```text
joueur -> environnement -> ennemi
```

au lieu de seulement:

```text
joueur -> ennemi
```

Cela peut rapprocher SBH d'un jouet systemique minimal, surtout si les ennemis
peuvent eux aussi declencher ou subir le decor.

### A. Pieges Visibles Actifs

Principe: pics, dalle de degats, concasseur ou case dangereuse deja active sur
le plateau. Les ennemis devraient pouvoir les subir.

Benefice potentiel: le joueur utilise le plateau contre un ennemi au lieu de
seulement comparer les Power.

Interactions possibles:

- Walker + piege: attirer ou synchroniser un Walker vers une case mortelle.
- Shove + piege: pousser un ennemi dans le danger, si Shove est teste plus tard.
- Collisions Ennemies + piege: une collision peut projeter ou bloquer une entite
  dans une zone dangereuse, si cette piste existe.
- Power: le piege peut reduire ou remplacer une taxe de combat.

Cout estime: faible a moyen.

Risques:

- rendre les Guards trop faciles a neutraliser;
- creer des morts arbitraires si le danger n'est pas extremement lisible;
- ajouter du calcul spatial sans surprise si les pieges ne sont que des murs
  rouges.

### B. Pieges Visibles Desarmes ou Activables

Principe: un piege existe sur une trajectoire, mais doit etre arme par un levier
ou une plaque. WAIT peut synchroniser l'activation.

Benefice potentiel: tres fort. Le joueur fabrique une consequence:
"j'arme le decor au bon moment pour battre un danger que je ne peux pas battre
directement".

Interactions possibles:

- Plaques Vivantes + piege: un Walker peut armer le piege qui tuera un autre
  acteur plus tard.
- WAIT + piege: synchroniser armement et passage.
- Portes/leviers: alterner entre ouvrir un chemin et armer une menace.
- Audio systemique: distinguer "arme" et "declenche".

Cout estime: moyen.

Risques:

- timing trop serre;
- ordre de resolution difficile a expliquer;
- basculer dans un puzzle scripté si chaque niveau a une seule sequence.

### C. Pieges Caches mais Detectables

Principe: des pieges non immediatement actifs peuvent etre reperes par indices
visuels, comportement observable, action de detection, ou autre information
honnête.

Benefice potentiel: colle fortement au theme "Smart Boy": plaisir de reperer
un danger avant de tomber dedans.

Interactions possibles:

- Fake Ad Deal: contrats absurdes autour de la detection ou de l'evitement.
- RNG a Choix: revelation partielle de dangers, a condition que le joueur garde
  une decision.
- Power: payer pour sonder ou desactiver, seulement si Power Spend est teste.

Cout estime: moyen a eleve.

Risques:

- piege invisible = mort arbitraire, a eviter absolument;
- ajouter une couche de lecture trop subtile pour des micro-niveaux;
- exiger une nouvelle action de detection, donc un vrai nouveau verbe.

### D. Pose de Pieges par le Joueur

Principe: le joueur peut placer ou armer lui-meme un piege.

Benefice potentiel: expression et preparation tactique fortes. Le joueur ne
resout plus seulement le niveau; il construit une solution.

Interactions possibles:

- Walkers: poser un piege sur leur trajectoire future.
- Shove: pousser une cible vers un piege prepare.
- Power Spend: payer du Power pour poser ou armer.
- Fake Ad Deal: contrats qui recompensent une elimination indirecte.

Cout estime: eleve.

Risques:

- vrai nouveau verbe, donc complexite UI et apprentissage;
- explosion de cas de design;
- peut diluer l'identite immediate de SBH si introduit trop tot.

Ordre de test suggere pour les pieges:

1. Piege visible active subi par un Walker.
2. Piege visible arme par plaque ou levier.
3. Seulement ensuite: detection ou pose de piege par le joueur.

Position actuelle: les pieges ne doivent pas interrompre l'experience Plaques
Vivantes. Ils doivent d'abord rester une extension future naturelle de Plaques
Vivantes, car cette experience teste la base: une entite autre que le heros peut
agir sur le monde. Si ce test provoque la reaction recherchee, alors les pieges
deviennent probablement la prochaine experience prioritaire devant Shove.

Statut 2026-08-16: une premiere micro-experience locale SBH a ete ajoutee en
niveaux 14-16 pour tester uniquement les pieges visibles/activables. Regle
choisie pour cette experience: un piege actif tue immediatement le heros ou un
Walker qui entre dessus; un piege inactif est traversable. Il n'y a ni piege
cache, ni Shove, ni collision ennemie, ni nouveau verbe joueur. Les pieges
peuvent etre actifs des le depart ou lies a un groupe existant; une plaque de
pression les arme temporairement, et un levier verrou les armerait de facon
permanente si un niveau en avait besoin.

Intention des niveaux experimentaux:

- 14 - WATCH YOUR STEP: distinguer visuellement une case dangereuse active et
  comprendre qu'y entrer tue.
- 15 - SET THE TRAP: tenir une plaque avec WAIT pour armer un piege au bon
  moment et detruire un Walker trop fort pour le heros.
- 16 - CLOCKWORK: proposer deux lectures reelles du meme systeme, soit attendre
  pour laisser le piege tuer le Walker, soit quitter la plaque trop tot et payer
  le combat avec du Power.

Observation de play-test prioritaire: le joueur doit-il parler du piege comme
d'un outil qu'il utilise sur le monde, ou seulement comme d'un obstacle rouge a
eviter ? Si la reaction reste "j'evite la case dangereuse", la famille pieges
ne repond pas encore a la question du jouet systemique minimal.

## Experience Candidate: SHOUT / Leurre Sonore

Statut 2026-08-16: une experience locale SBH est en cours pour tester une
capacite minimale de manipulation ennemie. Elle ne valide pas la forme finale du
leurre, ni une economie de charges, ni une migration temps reel.

Regle testee:

```text
SHOUT a la position du heros
    -> Walkers compatibles dans un rayon simple
    -> investigate(target_cell)
    -> patrol
```

`target_cell` est la case depuis laquelle le cri a ete emis. Les Guards restent
statiques pour l'instant. Si un Walker ne trouve pas de route vers la cible, il
abandonne proprement et reprend sa patrouille.

Hypothese de fun:

> si le joueur peut volontairement attirer les ennemis, les pieges cessent
> d'etre seulement des verrous de timing et deviennent des outils.

Variables volontairement non testees dans cette premiere passe:

- caillou lance ou cible libre;
- leurre pose;
- plusieurs types de bruit;
- recharge par kill;
- IA avancee;

Niveaux experimentaux:

- COME HERE: appeler un Walker vers un piege.
- GROUP THERAPY: utiliser une plaque et un seul cri pour provoquer un double
  kill.
- SMART WAY: comparer route brute couteuse en Power et route smart par
  manipulation + piege.

Variable de design future: SHOUT est gratuit ou tres disponible maintenant pour
ne pas melanger le test de manipulation avec une economie de ressources. Si le
plaisir est confirme, les charges de SHOUT ou leur recharge par kill indirect
pourraient devenir une piste separee.

Question de play-test:

> SHOUT ouvre-t-il plusieurs decisions plausibles de position/timing, ou devient-il
> simplement une nouvelle cle dans une serrure de puzzle ?

## Experience Candidate: Monde Semi-Continu

Statut 2026-08-16: une experience locale SBH teste un monde semi-continu a
fixed timestep sur les niveaux SHOUT uniquement. Les niveaux historiques restent
tour par tour pour conserver le checkpoint jouable et eviter une conversion de
campagne prematuree.

Regle testee:

```text
render frames
    -> accumulateur local SBH
    -> world.update_tick()
    -> Walkers / plaques / pieges continuent sans action joueur
```

Objectif de fun:

> verifier si SHOUT + pieges deviennent plus naturels lorsque les Walkers
> continuent a vivre pendant que le joueur observe et se repositionne.

Choix volontairement limites:

- pas de migration temps reel complete;
- pas de mouvement libre pixel-perfect;
- pas de pause tactique / Smart Vision;
- pas de recharge SHOUT par kill;
- pas de loot ou economie de kill;
- pas de scheduler moteur generique.

Le kill recoit uniquement une gratification immediate supplementaire: son
distinct et burst visuel primitif. Le probleme plus profond de recompense
systemique du kill reste ouvert.

Question de play-test:

> le monde semi-continu transforme-t-il vraiment SHOUT en outil d'orchestration,
> ou automatise-t-il simplement le meme puzzle ?

Retour play-test qualitatif:

- le monde semi-continu rend SBH nettement plus vivant;
- SHOUT est plus amusant que le puzzle arithmetique pur;
- attirer volontairement les Walkers dans les pieges semble etre une meilleure
  direction centrale;
- le tour par tour historique reste un checkpoint utile, mais n'est plus la
  direction privilegiee pour le futur SBH.

Limite apparue: un Walker semi-continu pouvait passer a cote du heros sans
reagir, sauf si SHOUT avait ete utilise. Cela rendait SHOUT trop proche d'une
"revelation d'existence" alors qu'il doit plutot manipuler l'attention.

Correction locale retenue pour l'experience suivante:

```text
ChaseHero si le Walker est adjacent au heros
    > Investigate(target) issu de SHOUT
    > Patrol
```

La detection est volontairement limitee a l'adjacence et aux niveaux
SemiContinuous. Les anciens niveaux tour par tour gardent leurs timings
historiques. Il n'y a pas encore de cones de vision, de furtivite, de memoire
avancee, de propagation sonore ou de framework IA.

## Experience Candidate: Vertical Slice 2D Isometrique

Statut 2026-08-16: une branche d'exploration teste une seule salle SBH rendue en
2D isometrique stylisee, separee du prototype principal. L'objectif n'est pas de
convertir la campagne, mais de comparer le meme coeur de gameplay avec une
presentation plus lisible et plus sensorielle.

Hypothese de fun:

> une petite arene mecanique isometrique, animee et mieux juiciee permet-elle de
> ressentir SHOUT + pieges comme un jouet systemique vivant plutot que comme un
> puzzle de rectangles ?

Architecture retenue:

- grille logique orthogonale conservee;
- projection uniquement graphique `world(x, y) -> screen(x, y)`;
- simulation SemiContinuous conservee;
- interpolation visuelle locale entre cases pendant le tick;
- sprite sheet PNG embarque pour le slice;
- tri de profondeur simple par `x + y`;
- pas de renderer GPU de sprites;
- pas de scene graph;
- pas d'ECS;
- pas d'AssetSource / AssetManager.

Capacites GPE introduites car un consommateur concret existe maintenant:

- `Image` RGBA8: necessaire pour les sprites SBH;
- blit framebuffer RGBA avec alpha et clipping: necessaire pour dessiner les
  sprites;
- source rectangle: necessaire pour sprite sheet / animation;
- decode PNG via une petite dependance dediee: necessaire pour un pipeline
  Aseprite -> PNG -> Image RGBA.

Friction asset confirmee:

```text
path logique
    -> bytes embarques
    -> decode PNG/WAV
    -> Image/SoundBank
```

Audio et sprites demontrent maintenant que "path logique -> bytes" deviendra
probablement une responsabilite moteur. La preference reste toutefois de ne pas
creer `AssetSource` maintenant: les deux consommateurs actuels sont encore servis
par des assets embarques, et aucun besoin concret de remplacement runtime,
filesystem natif ou fetch Web n'a encore ete teste dans un jeu.

Limites connues du slice:

- assets placeholders coherents, pas direction artistique finale;
- occlusion evitee par murs bas / salle simple;
- pas de mouvement libre;
- pas de pause tactique / Smart Vision;
- feedback kill plus visible mais toujours sans loot ni economie;
- pas encore de vrai "WOW mechanic" comme le boulet roulant.

Question de play-test:

> la 2D isometrique ameliore-t-elle suffisamment la perception de "petite arene
> mecanique vivante" pour justifier son cout supplementaire ?

Retour play-test qualitatif:

- meme avec des placeholders, l'isometrique augmente fortement l'interet et la
  sensation de jeu;
- la direction `2D isometrique + SemiContinuous + SHOUT + pieges systemiques`
  devient la direction d'exploration principale;
- le feedback audio/combat reste encore trop pauvre pour soutenir le spectacle;
- le prochain test doit mesurer un vrai moment de payoff, pas seulement une
  meilleure presentation du puzzle.

## Experience Candidate: Boulder Roulant

Statut 2026-08-16: premiere experience explicitement orientee "WOW mechanic".
Le Boulder n'est pas encore une mecanique definitive; il sert a verifier si SBH
peut produire un spectacle systemique rejouable.

Hypothese testee:

> plusieurs ennemis manipules par SHOUT + destruction spectaculaire dans une
> meme trajectoire peuvent produire l'envie de rejouer la salle pour optimiser le
> multi-kill.

Modele local:

```text
Boulder Ready
    -> pression/declencheur existant
    -> Rolling en ligne droite
    -> ecrase Walkers et heros sur sa trajectoire
    -> Stopped sur mur / limite / obstacle bloquant
```

Choix limites:

- pas de physique generale;
- pas de Rigidbody;
- pas de destructible environment;
- pas de nouvelle famille de pieges;
- pas de score, loot, XP ou recharge SHOUT;
- pas de loop audio de roulement;
- feedback `SMART xN` local a la course du Boulder.

Critere de play-test:

> le joueur pense-t-il "j'en ai ecrase 3, est-ce que je peux en mettre 5 dans
> l'axe ?" ou seulement "j'ai trouve la solution du puzzle" ?

## Comparatif

| Proposition | Potentiel de fun | Cout | Risque |
| --- | --- | --- | --- |
| Shove | Eleve | Moyen | Moyen |
| Collisions Ennemies | Tres eleve | Moyen | Moyen |
| Plaques Vivantes | Tres eleve | Faible/Moyen | Moyen |
| Power Spend | Moyen/Eleve | Moyen | Eleve |
| Bonus Deplacables | Moyen | Moyen | Eleve |
| RNG a Choix | Moyen | Faible/Moyen | Moyen |
| Fake Ad Deal | Eleve mais lateral | Moyen | Eleve |
| Audio Systemique | Moyen/Eleve | Faible/Moyen | Moyen |
| Pieges | Tres eleve | Moyen | Moyen/Eleve |
| SHOUT / Leurre Sonore | Tres eleve | Moyen | Moyen |

## Top 3

1. Plaques Vivantes.
2. Collisions Ennemies.
3. Shove.

Choix #1 pour une seule experience supplementaire: Plaques Vivantes.

Raison: cette piste teste directement si SBH peut devenir un petit systeme que
le joueur manipule sans ajouter tout de suite un nouveau bouton. Elle utilise
les Walkers, WAIT, portes et leviers existants, mais casse la limite actuelle:
"seul le heros agit sur le monde".

Classement mis a jour apres l'idee des pieges: le Top 3 historique ne change
pas comme trace de l'exploration initiale. En pratique, les pieges
visibles/activables sont maintenant la premiere experience candidate apres
Plaques Vivantes. Ils testent une question plus centrale: le plaisir vient-il de
l'orchestration du plateau plutot que d'un nouveau verbe direct ?

## Questions Ouvertes

- SBH doit-il rester un puzzle de calcul ou devenir un jouet systemique minimal ?
- Le plaisir central de SBH pourrait-il venir davantage de l'orchestration des
  interactions entre entites et environnement que de l'optimisation directe du
  Hero Power ?
- Jusqu'ou peut-on enrichir les interactions du plateau avant de perdre la
  lisibilite immediate qui fait partie de l'identite de SBH ?
- Accepte-t-on qu'une entite autre que le heros active le monde ?
- Vaut-il mieux enrichir les interactions existantes avant d'ajouter un nouveau
  verbe joueur ?
- Le Power doit-il rester une contrainte arithmetique ou devenir une ressource
  depensable ?
- Le hasard doit-il seulement varier les nombres ou generer des situations a
  resoudre ?
- Quelle part de solutions alternatives veut-on accepter dans des micro-niveaux
  tres lisibles ?
- Un leurre centre sur le heros suffit-il, ou faut-il rapidement une cible libre
  comme un caillou lance ?
- SHOUT doit-il rester gratuit, avoir des charges, ou etre recharge par kill
  indirect ?
- Le tour par tour suffit-il encore lorsque le joueur manipule activement les
  ennemis, ou faut-il tester un monde semi-continu a fixed timestep ?
- Une fois le monde vivant, faut-il ajouter une pause tactique / Smart Vision,
  ou le rythme lent suffit-il a la lisibilite ?
- Le feedback audiovisuel suffit-il a rendre un kill satisfaisant sans loot, ou
  faut-il une recompense mecanique locale ?

## Non-Engagement

Cette exploration ne valide pas l'implementation definitive de Shove, Pieges,
SHOUT, Power Spend, RNG a Choix, Fake Ad Deal, Audio Systemique ou toute autre
piste. Elle documente des options et leurs risques pour garder une trace de
conception.
