# GPE.UI Phase 0 — Gate 0 : audit du système existant

## Verdict

**Gate 0 — EXISTING SYSTEM UNDERSTOOD : PASS**

**Shortcomings assessment : REAL SHORTCOMINGS IDENTIFIED**, avec une portée limitée au toolkit `Ui` actuel. Les preuves ne démontrent ni l'échec global de `src/ui`, ni le besoin d'un nouveau sous-système, ni le besoin d'une crate. Ces questions sont hors du périmètre de ce document.

Le gate passe parce que les responsabilités existantes, leurs propriétaires d'état, leurs couplages input/rendu, leurs tests et leurs consommateurs observables ont été identifiés. Les limites retenues ci-dessous sont directement caractérisées par le code ou les tests ; aucune limite seulement hypothétique n'est promue en finding.

## Baseline exacte

| Champ | Valeur |
|---|---|
| Repository | `https://github.com/gotoo77/gotoo-pixel-engine` |
| Disponibilité | `LOCAL` |
| Branche | `research/gpe-ui-phase0` |
| HEAD audité | `6ff4f8baddae269baa6a7d182f0ba0c9d985f886` |
| Date de l'audit | `2026-08-31` (`Europe/Paris`) |
| Worktree avant production du document | propre, branche alignée sur `origin/research/gpe-ui-phase0` |
| Baseline historique mentionnée par la mission | `82ba7afa0933e5adeaa7ad5c1238d30e7d957771` ; non utilisée comme verrou |

Toutes les références `src/...` et `examples/...` de ce document désignent le repository et le commit exacts ci-dessus.

## Périmètre de preuve

Les fichiers obligatoires présents ont été lus intégralement :

- `Cargo.toml` ;
- `src/lib.rs` ;
- `src/ui/mod.rs` ;
- `src/ui/toolkit.rs` ;
- `src/ui/pause.rs` ;
- `src/ui/virtual_pad.rs` ;
- `src/ui/ordinal_identity_tests.rs` ;
- `src/ui/tabs_contract_tests.rs` ;
- `src/control.rs` ;
- `src/input.rs` ;
- `src/framebuffer.rs` ;
- `src/bitmap_font.rs` ;
- `src/image.rs` ;
- `src/image_fit.rs` ;
- `src/audio.rs`.

Les consommateurs utilisés comme preuve ont été inspectés dans `examples/tool_window_probe.rs`, `examples/arcade/game.rs` et `src/ui/pause.rs`. La recherche globale des symboles UI a également confirmé que `examples/tool_window_probe.rs` est le seul consommateur d'exemple de `Ui::new` hors tests à cette baseline. Le repository distant `gpe_arcade` n'est pas utilisé comme preuve dans cet audit : son contenu n'était pas nécessaire pour décider Gate 0.

## Carte réelle des responsabilités

### Toolkit immédiat

| Symbole / groupe | Source | Responsabilité réelle | Consommateurs observés | Propriété de l'état | Couplage input | Couplage rendu | Tests observés |
|---|---|---|---|---|---|---|---|
| `Ui<'a>` | `src/ui/toolkit.rs`, `Ui` ligne 143, `Ui::new` ligne 158 | Session immédiate d'une frame : déclaration, interaction et dessin des lignes/widgets ; finalisation du nombre d'interactifs et du scroll dans `Drop` | `examples/tool_window_probe.rs`, `Ui::new` ligne 151 ; tests du toolkit | Emprunte `&mut UiState`; les valeurs produit restent chez le consumer | Lit directement `&Input` et `delta_time` | Emprunte directement `&mut Framebuffer`; possède un `TextRenderer` léger | Tests unitaires dans `toolkit.rs`; contrats dédiés ordinal/tabs |
| `UiState` | `src/ui/toolkit.rs`, ligne 117 | Conserve focus ordinal, nombre d'interactifs précédent, capture pointeur, propriétaire du repeat horizontal, repeat gauche/droite, scroll et hauteur précédente | `ToolWindowProbe.ui_state`, lignes 21, 151 et 194 | Framework, explicitement possédé par le consumer | État dérivé des interactions | Aucun framebuffer possédé | `ordinal_identity_tests.rs`; `tabs_contract_tests.rs`; tests scroll/focus du toolkit |
| `UiTheme` | `src/ui/toolkit.rs`, ligne 77 | Tokens concrets : fonte, échelle, padding, hauteur/espacement de ligne et palette | `UiTheme::default()` dans le tool probe ligne 156 ; thèmes compacts dans les tests | Valeur copiée fournie par le consumer | Aucun | Types `Font` et `Pixel`, utilisés directement par le dessin | Tests de rendu et de géométrie via le toolkit |
| `UiResponse` | `src/ui/toolkit.rs`, ligne 108 | Retour synchrone `focused/hovered/active/clicked/changed` | Tool probe : `button(...).clicked` ligne 178 ; tests | Valeur transitoire de frame | Résultat de la résolution directe des entrées | Aucun couplage propre | Tests button/slider/select et contrats tabs |
| `label`, `section` | `src/ui/toolkit.rs`, lignes 202 et 207 | Lignes non interactives ; `section` ajoute un séparateur | Tool probe lignes 158, 163, 168, 174 et 177 | Aucun état persistant | Aucun | Dessin texte/ligne direct | `section_does_not_consume_an_ordinal` dans `toolkit.rs` |
| `tabs` | `src/ui/toolkit.rs`, ligne 233 | Une barre d'onglets = un ordinal ; retourne une sélection demandée sans muter la page du consumer | Tool probe lignes 159 et 194 | Sélection chez le consumer ; focus dans `UiState` | Flèches gauche/droite et pression souris directes | Dessin des tabs direct | `src/ui/tabs_contract_tests.rs`, notamment transition différée et reset |
| `button`, `toggle`, `select`, `slider_f32` | `src/ui/toolkit.rs`, lignes 274, 282, 295 et 353 | Contrôles immédiats en colonne ; `toggle/select/slider` mutent les valeurs empruntées, `button` retourne une réponse | Tool probe lignes 164–178 | Valeurs autoritaires chez le consumer ; interactions transitoires dans `UiState` | Clavier physique et souris lus directement | Cadre, texte et track dessinés directement | Tests clavier, souris, capture, normalisation, repeat et bornes |
| Scroll racine implicite | `src/ui/toolkit.rs`, calcul à partir de `next_row`, visibilité du focus et finalisation `Drop` ligne 745 | Scroll vertical unique de la surface entière ; garde le widget focalisé visible et dessine une scrollbar | Tool probe, section `OVERFLOW`, lignes 168–171 | `scroll_y` et hauteur précédente dans `UiState` | Déplacement du focus haut/bas ; pas d'entrée wheel observée | Coordonnées physiques décalées et scrollbar directe | `content_height_is_logical_and_scroll_does_not_drift`, focus wrap, clamp et reset dans `toolkit.rs` |
| `RepeatConfig`, `RepeatState` | `src/ui/toolkit.rs`, lignes 7 et 28 | Repeat temporel déterministe avec rattrapage de plusieurs pulses sur frame longue | `select` et `slider_f32` via l'état gauche/droite | Framework dans `UiState`, configuration valeur | Reçoit un `ButtonState` déjà résolu | Aucun | `repeat_is_time_based_and_preserves_catch_up_pulses` et tests select/slider |

### Primitives UI historiques et intégration jeu

| Symbole / groupe | Source | Responsabilité réelle | Consommateurs observés | Propriété de l'état | Couplage input | Couplage rendu | Tests observés |
|---|---|---|---|---|---|---|---|
| `MenuState` | `src/ui/mod.rs`, ligne 120 | Sélection linéaire circulaire pour un nombre fixe d'items | Arcade embarquée : champ ligne 182 ; `PauseGame` : champ ligne 101 ; autres exemples trouvés par recherche globale | Consumer/wrapper | Aucun en propre | Aucun | Navigation dans `src/ui/mod.rs` |
| `draw_panel`, `draw_text_centered`, `draw_menu_item` | `src/ui/mod.rs`, lignes 35, 40 et 54 | Helpers framebuffer stateless pour panel, texte centré et ligne de menu sélectionnée | Arcade `render_catalog`, lignes 294–324 ; pause `render_pause` | Aucun | Aucun | `Framebuffer`, `Rect`, `Pixel`, fonte intégrée | Test panel dans `src/ui/mod.rs`; primitives framebuffer testées séparément |
| `standard_menu_controls` et `menu_*_pressed` | `src/ui/mod.rs`, lignes 78–94 | Politique menu clavier/gamepad partagée ; fabrique un `ControlMap` ou interroge directement `Input` | Arcade `catalog_controls` ligne 349 ; `PauseGame`; plusieurs exemples localisés par la recherche globale | `ControlMap` chez le consumer ; helpers stateless | Touches W/flèches/espace et D-pad/stick/south | Aucun | Parité helpers/`ControlMap` clavier et gamepad dans `src/ui/mod.rs` |
| `ActionId`, `ControlBinding`, `ControlMap` | `src/control.rs`, lignes 6, 19 et 26 | Nomme des intentions, associe clavier/gamepad/virtuel, puis calcule un `ButtonState` par action | Arcade, pause et `VirtualPad` | Consumer ; maps internes de bindings, états et sources virtuelles | `Input` normalisé + source virtuelle | Aucun | Clavier/gamepad même action, device scope et transitions virtuelles |
| `VirtualButton`, `VirtualPad`, `VirtualPadUpdate` | `src/ui/virtual_pad.rs`, lignes 6, 33 et 18 | Hit zones tactiles vers `ActionId`; multi-contact; préserve les entrées de zones ordonnées au sein d'une frame | Arcade : pad lignes 184 et 199–203 ; pause : pads lignes 103–116 | Contacts et visibilité dans le pad possédé par le consumer | Événements tactiles bruts, sortie vers `ControlMap` | Aucun dessin : les consommateurs dessinent leurs zones | Tests touch press/release, retarget, ordre intra-frame, multi-contact, visibilité et reset |
| `PauseConfig`, `PauseGame<G>` | `src/ui/pause.rs`, lignes 26 et 96 | Wrapper de jeu : états running/paused/resume gate, interception modale, menu resume/quit, bouton tactile optionnel et barrière anti-fuite d'entrée | Arcade construit des jeux wrappés via `pause_game`; exemples web/natifs trouvés par recherche globale | Wrapper : état pause, menu, controls et pads ; jeu enfant reste propriétaire du gameplay | `ControlMap` clavier/gamepad + `VirtualPad` tactile | Layout et rendu framebuffer privés au wrapper | Tests enfant bloqué, reprise après relâchement, quit et séparation des zones tactiles |

### Fondations non-UI déjà disponibles

| Symbole / groupe | Source | Responsabilité pertinente pour l'UI | État / couplage | Tests observés |
|---|---|---|---|---|
| `Input`, `ButtonState`, `Key`, `MouseButton`, `Touch` | `src/input.rs`, lignes 304, 209, 7, 33 et 202 | Snapshot de frame et transitions `pressed/held/released` pour clavier, souris, gamepads et séquence tactile ordonnée | Possédé et avancé par la plateforme ; indépendant du framebuffer | Transitions, ordre tactile, devices, metadata et cycle de frame |
| `Framebuffer`, `Font` | `src/framebuffer.rs`, lignes 43 et 4 | Raster RGBA CPU, clipping robuste, primitives géométriques, image et texte bitmap à métriques déterministes | Aucun input ; rendu pixel direct | Couverture étendue clipping, dimensions extrêmes, alpha et glyphes |
| `Image`, `ImageRegion`, `ImageFit`, `ImageFilter` | `src/image.rs`; `src/image_fit.rs`, méthodes lignes 167 et 188 | Images RGBA/PNG, régions, contain/cover/stretch, nearest/linear et mapping exact des centres de pixels | Rendu direct dans `Framebuffer` | Fit, crop, clipping, nearest, bilinear alpha-correct et extents extrêmes |
| `BitmapFont`, `BitmapTextRenderer` | `src/bitmap_font.rs`, lignes 33 et 97 | Fonte bitmap fournie par le jeu et rendu/métriques réutilisables | Le jeu possède les glyphes immuables ; renderer vers framebuffer | Fallback, ponctuation et métriques mises à l'échelle |
| `Audio`, `AudioBus::Ui`, `NoopAudio` | `src/audio.rs`, lignes 70, 43 et 346 | Contrôle audio avec bus UI explicite et backend headless/no-op | Séparé de `src/ui`; accessible via la frame, sans dépendance du toolkit | Enregistrement/playback, routage bus, gain/mute ; variantes native/Web présentes |

## Forces observées

1. **Propriété d'état explicite, sans manager global caché.** `Ui::new` reçoit `&mut UiState`, `&Input`, `&mut Framebuffer` et une valeur `UiTheme`. Les réglages du tool probe (`bar_enabled`, `direction`, `bar_width`, etc.) restent dans `ToolWindowProbe`, tandis que `UiState` ne conserve que l'interaction.

2. **Primitives immédiatement testables sans fenêtre/GPU.** Le toolkit, le contrôle, le virtual pad, le menu, le framebuffer, les fontes, les images et `NoopAudio` possèdent des tests unitaires qui construisent directement `Input`/`Framebuffer`. Les contrats de focus, capture, repeat, scroll et tabs sont ainsi reproductibles en mémoire.

3. **Sémantique d'entrée déjà disponible hors du toolkit.** `ActionId` + `ControlMap` unifient clavier, gamepad ciblé ou quelconque, et sources virtuelles. `VirtualPad` alimente ce même modèle sans imposer le tactile à la logique jeu. L'Arcade et la pause utilisent effectivement cette voie.

4. **Contrats temporels difficiles déjà explicités et testés.** `PauseGame` possède un `ResumeGate` qui attend le relâchement et n'update pas le jeu enfant sur la frame de reprise. `tabs` retourne une demande différée ; le tool probe droppe `Ui`, reset l'interaction, puis change de page. Le virtual pad conserve l'ordre de plusieurs changements de zone dans une seule frame.

5. **Contrôle pixel et comportement aux bords solides.** Les géométries sont entières, le framebuffer clippe les dessins, le texte bitmap a des métriques connues, et `ImageFilter::Nearest` existe aux côtés du filtrage linéaire. Les tests couvrent aussi des coordonnées et dimensions extrêmes.

6. **Le toolkit actuel couvre déjà un écran de réglages compact.** Le tool probe démontre tabs, sections, toggles, sélecteurs, sliders, bouton et overflow/scroll dans un consumer réel du repository, avec état produit conservé côté consumer.

7. **Les couches sont composables sans dépendance audio UI obligatoire.** Les helpers historiques, le toolkit, `ControlMap`, `VirtualPad`, le rendu et l'audio restent séparés. Le bus `AudioBus::Ui` existe, mais aucun son/haptique n'est enfoui dans les widgets.

## Shortcomings assessment

### S1 — L'identité ordinale peut réaffecter un état interactif à un autre widget

**Preuve.** `UiState` identifie focus, capture pointeur et propriétaire de repeat par `usize` ordinal. `src/ui/ordinal_identity_tests.rs` caractérise explicitement :

- `focus_shift_rebinds_to_the_new_widget_at_the_same_ordinal` ligne 59 ;
- `active_pointer_rebinds_to_the_new_slider_at_the_same_ordinal` ligne 85 ;
- `horizontal_repeat_owner_rebinds_to_the_new_slider_at_the_same_ordinal` ligne 177 ;
- `page_shape_change_with_out_of_range_focus_has_a_one_frame_focus_gap_then_clamps` ligne 333.

**Manifestation et contournement observés.** Le contrat public de `UiState::reset_interaction()` exige un reset avant un changement structurel intentionnel. Le tool probe respecte ce contrat après une demande de tab (`examples/tool_window_probe.rs`, ligne 194), et les tests lignes 131, 249 et 396 démontrent que le reset évite les réaffectations.

**Qualification Gate 0.** Limite réelle du modèle actuel pour les structures dynamiques, mais contenue pour le seul consumer `Ui` observé. Aucune preuve de bug restant dans ce consumer et aucune conclusion architecturale n'en découle à ce gate.

### S2 — Le toolkit `Ui` ne consomme pas le modèle d'actions multi-input déjà utilisé par les autres UI

**Preuve.** `Ui::new` reçoit directement `&Input`; sa navigation et son activation lisent `Key::Up/Down/Left/Right/Space` ainsi que `MouseButton::Left` dans `src/ui/toolkit.rs`. Il ne reçoit ni `ControlMap` ni actions sémantiques. En parallèle, l'Arcade (`examples/arcade/game.rs`, champs lignes 182–184, factory ligne 349) et `PauseGame` (`src/ui/pause.rs`, champs lignes 101–104) utilisent `ControlMap` et `VirtualPad` pour clavier, gamepad et tactile.

**Manifestation observée.** Les UI multi-input existantes ne réutilisent pas le toolkit `Ui`; elles assemblent `MenuState`, helpers framebuffer, `ControlMap` et `VirtualPad`. Le seul consumer `Ui` hors tests est une fenêtre d'outil pilotée clavier/souris.

**Qualification Gate 0.** Écart réel entre deux chemins UI existants. Il ne prouve pas que tout doit être unifié, ni qu'un nouveau sous-système est requis ; il établit seulement que le toolkit générique actuel n'offre pas, tel quel, le même chemin gamepad/tactile sémantique.

### S3 — Le layout du toolkit est une colonne racine uniforme, sans primitive de composition géométrique

**Preuve.** Tous les éléments du toolkit passent par `next_row`; largeur = framebuffer moins padding, hauteur = `UiTheme.row_height`, progression verticale = hauteur + espacement. Le scroll est celui de la racine. Aucune API row, stack, grid, zone enfant ou custom rect n'est exposée dans `Ui` à cette baseline.

**Manifestation observée.** `examples/arcade/game.rs` maintient un `ArcadeLayout` séparé (ligne 102) avec deux ensembles de rectangles explicites, `Native` et `Touch`, puis dessine le catalogue manuellement dans `render_catalog` ligne 294. Ce consumer n'utilise pas `Ui`.

**Qualification Gate 0.** Limite réelle de couverture du toolkit, avec un consumer existant qui possède sa géométrie hors toolkit. Le code ne démontre pas que ce choix est coûteux, erroné ou insuffisant pour le catalogue actuel ; aucune capacité de layout supplémentaire n'est donc déclarée nécessaire ici.

## Limites examinées mais non promues en shortcomings structurels

- Le texte intégré couvre explicitement l'ASCII imprimable et replie les caractères inconnus vers un glyph déterministe. `BitmapFont` permet au jeu de fournir d'autres glyphes. Aucun consumer inspecté ne démontre aujourd'hui un besoin de shaping, BiDi ou accessibilité avancée ; ces absences ne sont donc pas transformées en défaut Gate 0.
- Le toolkit ne produit pas de file d'événements sémantiques : il retourne `UiResponse` ou mute des valeurs empruntées. Le tool probe utilise ce contrat sans workaround observé ; aucune insuffisance structurelle n'est affirmée.
- Aucun widget audio/haptique n'existe. L'audio dispose déjà d'un bus UI séparé et aucun consumer inspecté ne démontre qu'un couplage widget/audio est nécessaire.
- Les coûts compile-time, taille binaire, allocations/frame et runtime du toolkit n'ont pas été mesurés. Ils restent inconnus et ne sont pas présentés comme risques ou forces chiffrées.

## Primitives existantes qui satisfont déjà des besoins pertinents

Ces briques ne doivent pas être considérées comme absentes dans la suite de la Phase 0 :

- état produit autoritaire chez le consumer et état d'interaction séparé (`UiState`) ;
- contrôles immédiats `button`, `toggle`, `select`, `slider_f32`, `tabs`, labels et sections ;
- focus linéaire, hover, activation clavier/souris, capture pointeur de slider et réponse explicite ;
- repeat temporel avec rattrapage de pulses ;
- scroll racine et maintien en visibilité du focus ;
- sélection de menu circulaire (`MenuState`) ;
- actions nommées et bindings clavier/gamepad/virtuel (`ActionId`, `ControlMap`) ;
- tactile multi-contact vers actions, avec ordre intra-frame (`VirtualPadUpdate`) ;
- interception pause, modalité simple et barrière de relâchement (`PauseGame`) ;
- rendu pixel entier, clipping, texte bitmap, fontes fournies par le jeu, images nearest/linear ;
- tests headless des mécanismes centraux ;
- bus audio UI optionnel et backend `NoopAudio`, sans dépendance du kernel UI.

## Provenance des consumers effectivement utilisés

| Consumer | Repository @ ref | Disponibilité | Fichiers / symboles inspectés | Ce que la preuve établit |
|---|---|---|---|---|
| Fenêtre d'outil / réglages | `gotoo77/gotoo-pixel-engine@6ff4f8baddae269baa6a7d182f0ba0c9d985f886` | `LOCAL` | `examples/tool_window_probe.rs`: `ToolWindowProbe`, `update_tool_window`, `Ui::new`, tabs/toggle/select/slider/button, `reset_interaction` | Usage réel du toolkit complet, propriété consumer des valeurs, transition de page différée et reset ordinal |
| Catalogue Arcade embarqué | même repository/ref | `LOCAL` | `examples/arcade/game.rs`: `ArcadeLayout`, `ArcadeApp`, `update_catalog`, `render_catalog`, `catalog_controls` | Menu fixe, layouts native/touch explicites, rendu manuel, gamepad et tactile via actions |
| Pause générique | même repository/ref | `LOCAL` | `src/ui/pause.rs`: `PauseGame`, `PauseState`, `PauseLayout`, `update_running`, `update_paused`, `update_resume_gate`, tests | Overlay bloquant, navigation clavier/gamepad/tactile, resume/quit et anti-fuite d'entrée |
| Consumers unitaires de contrats | même repository/ref | `LOCAL` | `src/ui/ordinal_identity_tests.rs`; `src/ui/tabs_contract_tests.rs`; tests de `toolkit.rs`, `virtual_pad.rs`, `control.rs` | Comportements limites observés et contournements, déterminisme headless des mécanismes |

## Décision Gate 0

**PASS.**

Les responsabilités existantes sont comprises, les forces sont étayées, les shortcomings sont évalués sans extrapolation, et les consumers nécessaires au gate ont été observés avec provenance. Le PASS ne recommande aucune architecture et n'autorise aucune suite dans le cadre de cette exécution limitée.

**STOP APRÈS GATE 0.** Aucun Gate 1, Gate 2, prior art, architecture, DAG, modèle de transaction, MFE ou travail d'implémentation n'est inclus dans ce document.
