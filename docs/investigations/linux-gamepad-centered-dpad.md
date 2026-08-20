# Linux — D-pad centré asymétriquement et polarité verticale

## Statut

Résolu dans le backend gamepad natif.

Le problème a été observé avec un contrôleur `NEXT SNES Controller` sous Linux : certaines directions du D-pad, notamment `Down`, n’étaient pas détectées correctement alors que `Up` semblait fonctionner.

La cause n’était pas un mapping de boutons manquant dans le jeu, mais la façon dont ce contrôleur expose son hat/D-pad via `gilrs`.

## Symptôme

Le probe gamepad recevait correctement de nombreux boutons, mais le D-pad vertical avait un comportement incohérent :

- `Up` pouvait être vu ;
- `Down` ne produisait pas toujours l’état attendu ;
- des transitions numériques pouvaient inventer une direction opposée ou entrer en conflit avec les valeurs analogiques du hat.

## Observation importante

Le D-pad n’était pas exposé comme un axe standard centré sur `0.0` avec des extrêmes `-1.0 / +1.0`.

Une valeur neutre proche de :

```text
0.431
```

était observée.

Cela signifie qu’une calibration standard :

```text
negative = -1.0
center   =  0.0
positive =  1.0
```

est fausse pour ce device.

## Deuxième particularité : axe vertical Linux

Pour ce hat vertical, la convention observée était :

```text
valeur basse  -> Up physique
valeur haute  -> Down physique
```

alors que GPE conserve comme convention canonique une valeur positive pour `Up`.

La calibration verticale doit donc inverser les extrêmes.

## Modèle introduit

GPE utilise maintenant `AxisCalibration` :

```rust
AxisCalibration::new(raw_negative, raw_center, raw_positive, dead_zone)
```

Une calibration peut donc représenter un centre non nul et une polarité inversée.

Exemple validé dans les tests :

```rust
AxisCalibration::new(1.0, 0.431, 0.0, 0.10)
```

pour le hat vertical Linux concerné.

## Détection dynamique du centre

Le backend surveille les `ButtonChanged` associés aux directions du D-pad.

Une valeur intermédiaire entre `0.2` et `0.8` est considérée comme un candidat de centre pour l’axe correspondant :

```text
CENTER_LOW  = 0.2
CENTER_HIGH = 0.8
```

Une fois le centre observé :

- axe horizontal : `0.0 -> center -> 1.0` ;
- axe vertical : `1.0 -> center -> 0.0`.

Le résultat est ensuite normalisé dans l’espace logique GPE `[-1, +1]` et transformé en boutons numériques via le `digital_threshold` du `GamepadProfile`.

## Pourquoi ignorer ensuite certains événements numériques gilrs

Une fois qu’un D-pad centré de cette façon a été identifié, les événements `ButtonPressed` / `ButtonReleased` numériques du même axe sont ignorés.

Raison : utiliser simultanément :

- les edges numériques ;
- les valeurs analogiques/hat centrées ;

créait deux sources concurrentes pour le même état et pouvait produire des faux edges, notamment une direction opposée au retour au centre.

Le backend préfère donc la représentation calibrée de l’axe dès qu’elle est disponible.

## Profil gamepad

Le profil standard actuel contient notamment :

```text
left stick dead zone : 0.20
D-pad dead zone      : 0.10
digital threshold    : 0.50
```

Le seuil numérique est ajustable par jeu via `GamepadProfile`, sans ajouter un service gamepad obligatoire dans `Frame`.

Cette décision est importante : la correction matérielle reste dans la couche input/gamepad et n’alourdit pas l’API de frame pour les jeux qui n’en ont pas besoin.

## Tests de régression

Les tests couvrent notamment :

- axe standard dans les deux directions ;
- inversion de polarité ;
- centre asymétrique `0.431` ;
- hat vertical Linux inversé ;
- conservation des autres axes lors d’un override de profil ;
- absence de direction opposée inventée au relâchement ;
- suppression des edges numériques après détection d’un D-pad centré.

## Validation matérielle

Après le correctif, le contrôleur physique a été retesté avec le probe puis dans les menus/jeux GPE :

- D-pad Up/Down/Left/Right ;
- boutons face ;
- shoulders ;
- Start/Select ;
- navigation menu ;
- gameplay.

Le comportement a été validé sur le matériel concerné.

## Leçon

Un « bouton » de D-pad peut ne pas être un vrai bouton binaire du point de vue du backend. Certains contrôleurs exposent un hat analogique ou pseudo-analogique avec :

- centre non nul ;
- extrêmes non symétriques ;
- polarité différente selon la plateforme ;
- événements numériques dérivés imparfaits.

Le backend doit donc normaliser le matériel vers le modèle logique du moteur au lieu d’imposer au jeu les particularités du périphérique.
