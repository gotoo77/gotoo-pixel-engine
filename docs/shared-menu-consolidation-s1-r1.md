# Shared Menu Consolidation S1-R1 - rapport de clôture technique

Date : 2026-11-25. Verdict : **GO WITH CONDITIONS pour ouvrir la PR ; NO-GO pour merge automatique**.

## Références et périmètre

- Base canonique Core #114 : `e50977593dabe47e5c324cc50d3fefa27efef2f8`.
- Candidat technique gelé et validé : `e864eec7c0f3a082994d0b34a55fe86bf66ba8ea`.
- Branche : `feat/shared-menu-consolidation-s1-r1`.
- Avant ce rapport, GitHub confirme main à la base, neuf commits en avance, zéro en retard, dix fichiers modifiés, 2522 ajouts et 54 suppressions.
- Rapport historique conservé : [S1](shared-menu-consolidation-s1.md). Le présent document le complète, sans effacer son NO-GO historique.
- Checkpoint préservé : `58dbd3affb86be9f33ba77cfedd2f4bd6cc9e0c7` ; reprise sur #114 : `9ddb029df04069567f0eb0434d62814968fee44e`.

S1 fournit MenuStack, PauseSettingsMenu, intentions, quatre propositions audio, labels, style, outline et snapshot. Le consommateur P6 conserve l'unique autorité modale ; les transitions et la frame de relâchement suspendent le monde. Aucun S2/remapping, migration de jeu, changement de pin, dépendance, CI, plateforme, renderer, backend audio ou stockage. L'outline_text partagé est modifié uniquement pour le centrage des glyphes. Aucun changement produit depuis le candidat validé pour publier ce rapport.

## Revue et corrections

| Finding | Résolution | État |
| --- | --- | --- |
| D1 captures parent persistantes | Invalidation pointeur/touch au push/back, focus conservé ; scénario mixte et réactivation testés | PASS |
| D2 Back et édition audio concurrente | Priorité de Back avant commit des propositions ; test audio et activation concurrente Settings | PASS |
| D3 second wrapping outline | Layout dans les bounds originales puis translation des glyphes ; comparaison pixels des quatre libellés | PASS |
| R1 supersampling | Référence raster haute résolution, comparaison alpha par pixel aux facteurs 2 et 4 | PASS |
| R2 clipping | Texte débordant, pixels hors contrôle intacts et présence de pixels peints, facteurs 1/2/4 | PASS |
| R3 documentation | Ce rapport distinct | DONE |
| R4 runtime | Voir matrice ci-dessous | PARTIAL |

Revue statique ChatGPT sur 5a1e010 (auteur des corrections, non indépendante), puis diff des deux tests supplémentaires et correction rustfmt. Aucun nouveau défaut fonctionnel majeur confirmé dans les portions examinées. Les tests R1/R2 n'ont pas nécessité de modification produit. Le gate outline préexistant du rapport S1 a été corrigé séparément par #114.

## Validation automatisée du candidat

Rapport terminal utilisateur, WSL, Rust/Cargo 1.97.1, wasm-bindgen 0.2.127, cog 7.0.0. Répertoire local de preuve : `~/gpe-validation/s1-r1/20261125-133550-FglagF/` (non archivé dans Git). Les 23 entrées du summary.tsv ont toutes exit 0 :

| Gates | Résultat |
| --- | --- |
| preflight, 00-web-dependencies, 01-diff-integrity, 02-protected-files, 03-conventional-commits, 04-format | PASS |
| 05-native-check, 06-canonical-native (A9, tests, Clippy), 07-native-tests-locked, 08-outline-tests, 09-doctests | PASS |
| 10-clippy-native, 11-clippy-native-outline, 12-clippy-wasm, 13-clippy-wasm-outline | PASS |
| 14-web-check, 15-web-unit, 16-web-build, 17-web-browser | PASS |
| 18-example-native, 19-example-wasm, 20-final-format, 21-final-diff | PASS |

Les gates web existants ne testent pas le nouveau menu en navigateur. Le build WASM P6 ne démontre pas son lancement interactif. Avertissement npm : postinstall esbuild bloqué par allowScripts ; gates web néanmoins PASS, aucune autorisation ajoutée. Pas de test affaibli. Les résultats portent sur e864eec, non sur le futur commit documentaire ; CI distante de la PR à vérifier.

## Recette réelle et risques résiduels

| Contrôle | Résultat et limite |
| --- | --- |
| Windows natif P6 bitmap et outline, release | Affichage et navigation clavier observés ; environ 58 FPS, mesures ponctuelles seulement |
| Audio | Quatre propositions distinctes observées dans les captures ; aucun son joué par P6 ; pas de preuve de persistance disque ni d'effet audio réel |
| Retour/focus/release/Exit | Tests automatisés PASS ; checklist physique exhaustive non attestée |
| Manette physique | NOT CONFIRMED ; tests synthétiques du consommateur PASS |
| Souris/touch P6 | NOT IMPLEMENTED dans ce consommateur Standard ; API Core et tests synthétiques existent. Pas une recette physique réussie |
| Menu outline | Tests framebuffer PASS ; P6 utilise le menu bitmap même avec outline-fonts ; pas de recette visuelle interactive du menu outline |
| WSLg | BLOCKED : absence d'affichage sur candidat et baseline #114 ; cause non identifiée |
| Shared Shell Web | NOT RUN ; compilation et Chromium existants PASS, aucune preuve d'un entrypoint Web P6 utilisable |
| Troncature POINTER: MOVE HERE | Préexistante, observée sur #114 et candidat ; hors scope |

### Incident OOM Windows

Un panic wgpu Out of Memory (exit 101) est survenu en release avec outline-fonts pendant la manipulation des menus. Pas de backtrace exploitable de cet incident. Baseline #114 avec outline : aucun crash observé pendant resize/déplacement/pointeur, mais pas de menu S1, donc comparaison non équivalente. Candidat bitmap puis outline : navigation et modifications audio effectuées, crash non reproduit. Ne pas conclure à une correction, à une absence de fuite ou à une cause environnementale démontrée.

Sonde diagnostics_environment_probe avec diagnostics : NormalExit, 360 frames principales, trois incarnations de renderer détruites, Vulkan sur RTX 5060, 15 événements conservés sans perte. Champs d'erreurs Unknown, pas une preuve d'absence d'erreur. Cette sonde n'instrumente pas le crash P6. P6 appelle run et non run_with_diagnostics ; persistance/panic hook à la charge du consommateur. Conserver le besoin de crash reporting au backlog, sans nouveau chantier bloquant spéculatif.

## Expérimentation sans Codex

Reprise et corrections R1 via ChatGPT/GitHub ; checkpoint S1 initial issu du travail antérieur. Worker GPE-Agent : fmt validé, check bloqué par permissions du cache Cargo, test/clippy bloqués par pkg-config/ALSA dans le sandbox. Aucun PASS produit déduit de ces échecs infrastructure. Validation conventionnelle locale de substitution complète, sans assouplissement. CVR et FGH-UX restent indépendants ; pas de nouveau développement ici.

## Conditions de merge

- CI distante du head documentaire : vérifier les checks requis.
- Accepter explicitement le risque OOM intermittent non reproduit ; si récidive, collecter backtrace/diagnostics et réévaluer avant merge.
- Confirmer la recette physique manette et les points clavier restants, ou accepter explicitement leur report.
- Confirmer la recette Web du nouveau menu, ou accepter explicitement son report avec absence d'entrypoint P6 vérifié.
- Accepter les limites WSLg, souris/touch P6 et menu outline interactif ; ne pas les présenter comme validées.
- Aucune migration ni S2 autorisés par cette clôture. Pirates P1.2 non débloqué par S1 seul.

Décision : clôture technique automatisée et revue statique achevées, recette partielle. PR draft recommandée pour rendre visibles les conditions ; pas de merge, publication release ou déploiement. Le GO de création de PR ne vaut pas acceptation de toutes les exceptions de recette.
