# GPE.UI — Adversarial Review du Master Prompt v0.1

## Verdict

Le prompt est **très solide sur les garde-fous**, mais il souffre d’un défaut symétrique : il couvre tellement de risques qu’il peut pousser l’agent à **fabriquer une architecture “complète sur papier”** avant d’avoir identifié les 3–5 incertitudes qui méritent réellement une Phase 0.

| Dimension | Évaluation |
|---|---|
| Intention | **Excellente** |
| Discipline anti-rewrite | **Excellente** |
| Falsifiabilité recherchée | **Très bonne** |
| Pressure-testing consommateurs | **Très bon** |
| Protection contre browser/framework accident | **Très bonne** |
| Neutralité architecturale réelle | **Moyenne** |
| Réduction d’incertitude | **Moyenne** |
| Maîtrise du scope | **Faible** |
| Risque de documentation cérémonielle | **Élevé** |
| Risque d’architecture spéculative | **Élevé** |
| Exécutabilité par un agent | **Moyenne** |

**Verdict global : `PASS WITH MAJOR CONDITIONS`**

Je ne lancerais pas encore cette v0.1 telle quelle.

---

## 1. Le problème principal : ce n’est plus vraiment une Phase 0

Le prompt demande simultanément :

- audit complet de GPE ;
- prior art ;
- modèle d’état et ownership ;
- crates/modules/features ;
- layout / responsive ;
- input / focus / events ;
- feedback jeu ;
- styling et custom widgets ;
- texte / i18n / accessibilité ;
- performance / déterminisme ;
- tests / observabilité / inspector ;
- markup / data binding / hot reload ;
- SVG ;
- sécurité / versioning / diagnostics ;
- world-space UI ;
- migration ;
- trois architectures complètes ;
- un MFE ;
- une revue adversariale.

Cela ressemble à :

> **Phase 0 + architecture générale + roadmap produit + étude prospective v2/v3 + migration plan + test architecture + design du premier MFE.**

Le risque est de produire énormément de documentation sans mieux répondre à la question essentielle :

> **Quel est aujourd’hui le plus petit problème UI structurel réel de GPE que nous devons résoudre ?**

---

## 2. Biais involontaire vers une architecture ambitieuse

Le prompt affirme correctement que `UiTree`, retained mode, markup, SVG, inspector, etc. sont seulement des hypothèses candidates.

Mais il consacre ensuite à plusieurs d’entre elles :

- des rapports complets ;
- des questions obligatoires ;
- des pressure tests ;
- des critères de compatibilité future ;
- une architecture « AMBITIOUS ».

Cela crée une **specification gravity** :

> Le texte dit « ne présume pas qu’on en a besoin », mais la structure dit « étudie-les suffisamment pour qu’elles deviennent importantes ».

### Correction recommandée

Adopter une règle plus dure :

```text
No current consumer, no kernel consequence.
```

Une capacité future ne doit influencer le kernel que si elle permet d’identifier un **architectural dead-end difficile ou coûteux à corriger plus tard**.

Sinon :

```text
DEFER
```

---

## 3. Il manque une Architecture ZÉRO

La synthèse force actuellement :

```text
Minimal
Balanced
Ambitious
```

Il manque une possibilité fondamentale :

```text
Architecture 0 — DO NOT CREATE GPE.UI

GPE existing src/ui
+
targeted improvements
+
no new architectural subsystem yet
```

Sans cette option, la question implicite reste :

> « Quelle forme devrait prendre GPE.UI ? »

alors que la vraie question devrait être :

> **« GPE.UI mérite-t-il d’exister en tant qu’abstraction distincte ? »**

Architecture 0 doit être un concurrent obligatoire.

---

## 4. Contradiction dans les livrables

Le prompt demande de créer « au minimum » une longue liste de documents, puis autorise à fusionner certains rapports et demande de ne pas créer artificiellement de nombreux fichiers superficiels.

Cela crée une ambiguïté pour un agent littéral.

### Correction

Préférer :

```text
The topics below MUST be covered.
The file decomposition is NOT mandatory.
Prefer fewer substantial documents over many shallow documents.
```

L’obligation porte sur la **couverture**, pas sur le nombre de fichiers.

---

## 5. Risque de fausse précision sur les coûts

Le prompt demande des coûts conceptuels concernant :

- dependencies ;
- compilation ;
- binary size ;
- runtime state ;
- allocations ;
- maintenance ;
- WASM size ;
- layout/render time.

Sans mesure réelle, un agent peut facilement inventer :

```text
Compile cost: Low
Binary cost: Medium
Runtime cost: Low
```

sans preuve.

### Correction

Toute estimation doit être classifiée :

```text
KNOWN
SUPPORTED ESTIMATE
HYPOTHESIS
UNKNOWN
```

Et appliquer :

> **A cost without measurement or external evidence MUST be marked UNKNOWN.**

---

## 6. Il manque un vrai « uncertainty budget »

La Phase 0 devrait commencer par identifier les inconnues ayant le plus fort impact.

Ajouter avant toute architecture :

```text
Identify the 5 highest-impact uncertainties.

For each:
- why it matters;
- evidence currently available;
- consequence if wrong;
- cheapest way to reduce it.
```

La Phase 0 doit optimiser :

> **information gain / effort**

et non :

> **coverage completeness**.

Il est tout à fait possible que 80 % du vrai problème repose sur seulement :

```text
identity/state
layout
semantic navigation
integration with existing Ui
```

---

## 7. L’audit réel doit devenir un gate

L’obligation d’inspecter le repository réel est excellente.

Mais l’état du code doit être autorisé à **changer ou tuer la suite du programme de recherche**.

Ajouter :

```text
GATE A — EXISTING SYSTEM UNDERSTOOD

Before architecture research can continue:

- identify actual existing UI responsibilities;
- identify real shortcomings demonstrated by consumers;
- identify existing primitives already satisfying future requirements.

If insufficient evidence:
STOP / RESEARCH INCOMPLETE.
```

Si `src/ui` contient déjà une grande partie du kernel nécessaire, la Phase 0 doit pouvoir se recentrer autour de cette réalité.

---

## 8. Les consommateurs restent parfois trop conceptuels

Arcade, Pause, Settings, HUD et Debug sont de bons pressure tests.

Mais il faut imposer :

> Chaque capacité majeure proposée doit être justifiée par **au moins un usage réel actuellement observable dans un consumer**, ou être explicitement marquée `SPECULATIVE`.

Exemple correct :

```text
stable ID

WHY?
- dynamic Arcade list use-case
- Pause focus restoration use-case
```

Exemple insuffisant :

```text
Stable IDs are useful for retained UIs.
```

---

## 9. Future markup compatibility risque de contaminer le kernel

Le principe :

```text
Rust API ──────┐
               ├── SAME INTERNAL MODEL
Markup ────────┘
```

est séduisant mais dangereux.

Il peut pousser à modifier le design Rust actuel uniquement pour satisfaire un frontend déclaratif hypothétique.

Cela pourrait influencer :

- closures ;
- génériques ;
- ownership ;
- custom rendering ;
- state wiring.

### Correction

Adopter :

> **Rust ergonomics wins unless a current requirement independently justifies the abstraction.**

Et :

> Future markup compatibility may reject a dead-end, but may not introduce kernel machinery by itself.

---

## 10. Même problème pour SVG

SVG reçoit quasiment un programme R&D complet.

La première question devrait être :

> **Existe-t-il actuellement un consumer nécessitant réellement du vector UI runtime ?**

Si non :

```text
SVG = DEFERRED CAPABILITY PRESSURE

Only answer:
- could current kernel prohibit it later?
- if yes, how?
- otherwise stop investigation.
```

---

## 11. Compatibilité future != préparation future

Il faut distinguer :

> « Est-ce que cette décision nous enferme ? »

de :

> « Construisons-nous dès maintenant les abstractions pour toutes les capacités futures ? »

Le premier produit une architecture **évolutive**.

Le second produit une architecture **anticipatrice**.

GPE doit privilégier le premier.

---

## 12. Il manque le DAG des dépendances

Avant de recommander une extraction en crate(s), il faut étudier explicitement :

```text
gpe
gpe-ui
framebuffer
input
fonts
images
audio
```

et leurs relations.

Ajouter un livrable conceptuel obligatoire :

```text
CANDIDATE DEPENDENCY DAG
```

avec :

```text
crate/module
depends on
must NOT depend on
public types crossing boundary
reason
```

Cela permettra de détecter :

- cycles ;
- extractions artificielles ;
- mauvais ownership de types ;
- fragmentation excessive.

---

## 13. Il manque le modèle temporel d’une frame UI

Le prompt traite séparément input, layout, events, render et state, mais pas assez explicitement leur ordre.

Il faut comparer des pipelines comme :

```text
input
→ tree/build
→ layout
→ hit-test
→ focus mutation
→ actions
→ gameplay mutation
→ render
```

ou d’autres variantes.

Ajouter pour chaque architecture :

```text
FRAME TRANSACTION MODEL
```

Ce sujet impacte :

- déterminisme ;
- borrow ergonomics ;
- mutation pendant traversal ;
- focus ;
- callbacks ;
- invalidation ;
- latence d’une frame ;
- animations.

C’est plus structurant que SVG.

---

## 14. Propagation / consommation des événements insuffisamment explicite

Ajouter aux questions R5 :

```text
direct dispatch?
parent propagation?
capture/bubble?
consumption?
priority?
```

Il ne s’agit pas de copier le DOM.

Il faut simplement définir qui reçoit un événement, qui peut le consommer, et comment les modales/overlays/custom widgets s’intègrent.

---

## 15. Rust ergonomics ne peut pas être totalement validé sur papier

Les pseudo-usages sont utiles mais insuffisants.

Une API conceptuellement élégante peut provoquer immédiatement :

```text
cannot borrow `ui` as mutable more than once
```

Le résultat Phase 0 devrait donc être limité à :

```text
CONCEPTUALLY ERGONOMIC
```

La propriété réelle :

```text
RUST ERGONOMIC
```

doit être validée par le MFE compilé.

---

## 16. Le MFE candidat ressemble encore un peu trop à une mini-démo

Le candidat proposé :

```text
Panel
Text
Button
Row/Column
constraints
semantic navigation
mouse/touch
two resolutions
headless tests
```

est cohérent, mais il ne faut pas le préfiger avant d’avoir identifié les risques dominants.

Ajouter :

```text
Risk × Uncertainty × Information Gain matrix
```

Puis choisir le MFE en fonction du résultat.

La véritable incertitude critique pourrait par exemple être :

```text
stable identity
+
dynamic tree
+
focus restoration
```

plutôt que le responsive layout.

---

## 17. Excellente revue adversariale, mais certains adversaires doivent agir plus tôt

Les adversaires définis sont très bons.

Mais trois d’entre eux devraient intervenir **pendant** chaque proposition architecturale :

```text
Small Game
Rust Ergonomics
Migration
```

Ils doivent pouvoir éliminer une solution avant qu’elle ne contamine les rapports suivants.

---

## 18. `Confidence: LOW / MEDIUM / HIGH` global est trop faible

Un niveau de confiance global masque les différences entre décisions.

Préférer :

```text
Decision confidence:
- evidence-backed
- consumer-backed
- prior-art-backed
- hypothesis only
- requires MFE
```

Exemple :

```text
Kernel direction           HIGH
Stable IDs                 MEDIUM
Layout algorithm           LOW
Markup compatibility       LOW
SVG                         UNKNOWN
```

---

## 19. Petite contradiction sur l’état terminal

Le rapport autorise :

```text
READY FOR ADVERSARIAL REVIEW
or
BLOCKED
```

mais impose ensuite un état terminal « RESEARCH COMPLETE — READY... ».

Si la mission est bloquée, ces deux affirmations sont incompatibles.

Prévoir :

```text
COMPLETE — READY FOR INDEPENDENT ADVERSARIAL REVIEW
```

ou :

```text
INCOMPLETE — BLOCKED
```

---

# Points particulièrement réussis

Le prompt doit être conservé comme base.

Excellentes décisions déjà présentes :

- anti-rewrite ;
- `policy != mechanism` ;
- état gameplay autoritaire ;
- deterministic-by-default ;
- headless-first ;
- pay-only-for-what-you-use ;
- no hidden globals ;
- escape hatches ;
- migration-before-removal ;
- testability-as-architecture ;
- interdiction d’implémentation en Phase 0.

Le contrat de décision suivant est particulièrement fort :

```text
Problem
Evidence
Candidate
Alternatives
Trade-offs
Consumer pressure
Testability
Cost
Failure mode
Revisit trigger
```

À conserver.

---

# Restructuration recommandée

## Logique actuelle

```text
AUDIT
↓
EXPLORE EVERYTHING
↓
COMPARE 3 ARCHITECTURES
↓
SELECT
↓
MFE
```

## Logique recommandée

```text
AUDIT
↓
OBSERVED PROBLEMS
↓
TOP UNCERTAINTIES
↓
MINIMUM NECESSARY RESEARCH
↓
ARCHITECTURE 0 / A / B / C
↓
ADVERSARIAL REDUCTION
↓
RISK-RANKED MFE
```

La première cherche :

> une bonne architecture UI.

La seconde cherche :

> **le minimum d’architecture UI dont GPE peut prouver avoir besoin.**

C’est cette seconde logique qui est la plus cohérente avec GPE.

---

# Direction concrète — v0.2 avant exécution

Faire une **v0.2 du master prompt avant de lancer la Phase 0** avec les modifications prioritaires suivantes :

1. Ajouter **Architecture 0 — evolve existing `src/ui`, no new subsystem**.
2. Ajouter un **Gate 0 : observed problems + top 5 uncertainties** avant le prior art général.
3. Remplacer l’obligation d’environ 19 fichiers par une obligation de **coverage**, avec regroupement libre.
4. Déclarer `markup`, `SVG`, `world-space`, `inspector`, `advanced accessibility` **DEFER BY DEFAULT**, sauf preuve consommateur ou architectural dead-end.
5. Interdire toute estimation de coût inventée : **unsupported cost = UNKNOWN**.
6. Ajouter un **crate/module dependency DAG** obligatoire.
7. Ajouter un **frame transaction / event timing model** obligatoire.
8. Ajouter propagation et consommation des événements aux questions R5.
9. Faire du MFE le résultat d’une **risk / uncertainty / information-gain matrix**.
10. Remplacer la confiance globale par une **confidence per major decision**.
11. Faire intervenir `Small Game`, `Rust Ergonomics` et `Migration` pendant la conception.
12. Corriger les contradictions :
    - `minimum files` ↔ `merge allowed`
    - `BLOCKED` ↔ `RESEARCH COMPLETE`

---

# Verdict terminal

## État actuel

```text
PASS WITH MAJOR CONDITIONS
```

## Après intégration des corrections prioritaires

Cible :

```text
READY TO EXECUTE PHASE 0
```

Le but n’est pas nécessairement de rendre le prompt beaucoup plus court.

Le but est de lui donner le droit de :

> **ne pas explorer ce qui ne mérite pas d’être exploré.**
