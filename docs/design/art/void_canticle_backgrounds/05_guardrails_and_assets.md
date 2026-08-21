# 4. Version production — Exploitabilité gameplay

## 4.1 Ne pas utiliser le background comme une simple texture

Le vieux modèle :

```text
petite texture
     ↓
scroll
     ↓
boucle visible
```

doit être remplacé par une composition en couches.

### Stack recommandé

```text
FAR BACKGROUND HD
        ↓
DISTANT STRUCTURES
        ↓
NEBULA / DUST
        ↓
DEBRIS BACK
        ↓
=====================
      GAMEPLAY
=====================
        ↓
DEBRIS FRONT / FX
        ↓
HUD
```

Le background HD peut être quasiment statique ou défiler extrêmement lentement.

---

## 4.2 Parallaxe

Exemple de rapports de vitesse :

```text
background cosmique       0.02
planètes / structures     0.05
brumes / poussières       0.10
débris lointains          0.15
débris moyens             0.30
gros débris proches       0.60
poussières premier plan   1.00
```

Les valeurs exactes doivent être ajustées au gameplay ; c’est le rapport entre les couches qui compte.

### Bénéfice

Même si plusieurs couches bouclent, elles ne recommencent pas ensemble.

La répétition devient beaucoup moins perceptible.

---

## 4.3 Débris animés

Commencer petit :

- 6 à 12 sprites ;
- 3 profondeurs ;
- dérive verticale ;
- légère dérive latérale ;
- rotation lente ;
- vitesse différente selon la profondeur.

### Rotation

Si GPE ne possède pas encore une rotation de texture adaptée, ne pas créer immédiatement une abstraction lourde.

Solution minimale :

- pré-rendre 4 ou 8 orientations ;
- sélectionner l’image correspondant à l’angle.

### Événements rares

Quelques comportements peuvent renforcer fortement l’identité :

- un débris dérive vers une singularité ;
- une immense structure traverse lentement un bord ;
- une épave disparaît dans l’obscurité ;
- une poussière accélère ;
- un fragment passe derrière le gameplay puis devant une autre couche.

Ces événements doivent rester rares.

---

## 4.4 Background vivant

Le fond n’a pas besoin d’une vidéo complète.

Micro-animations possibles :

- halo respirant ;
- luminosité variant de quelques pourcents ;
- filaments se déplaçant lentement ;
- étoiles scintillant rarement ;
- anneau en rotation très lente ;
- brume dérivante ;
- colonne d’énergie pulsante ;
- éclair gravitationnel occasionnel.

### Principe

> Animation lente, faible amplitude, fréquence irrégulière.

Le fond doit sembler vivant sans détourner le regard du combat.

---

## 4.5 Separation HD / gameplay pixel

Approche recommandée :

- **background HD** indépendant ;
- **gameplay pixel-art** conservé ;
- **HUD** au-dessus ;
- effets additionnels séparés.

Cela permet de conserver :

- la lisibilité rétro du gameplay ;
- la richesse du monde ;
- une résolution de fond adaptée au 9:16 ;
- une identité visuelle nettement plus ambitieuse.

Le mélange peut devenir une signature de VC :

> **pixel SHMUP lisible au premier plan / space-opera gothique gigantesque derrière.**

---

# 5. Règles de production des futurs backgrounds

Pour toute nouvelle image destinée au gameplay :

## Composition

- format 9:16 ;
- centre relativement calme ;
- landmark principal plutôt haut ou latéral ;
- grandes formes plutôt que micro-détails ;
- profondeurs clairement distinctes.

## Contraste

- pas de blanc pur au milieu du combat ;
- petites sources lumineuses limitées ;
- luminosité maximale réservée aux landmarks ;
- fond plus sombre que les projectiles.

## Détail

- détails fins surtout en périphérie ;
- éviter les petites formes géométriques ressemblant à des tirs ;
- textures très lointaines floutées ou peu contrastées.

## Couleur

- une couleur dominante claire ;
- 1 à 2 accents secondaires ;
- zones brillantes localisées ;
- cohérence avec le stage.

## Animation

Prévoir dès la création ce qui peut être séparé :

- débris ;
- poussière ;
- filaments ;
- astéroïdes ;
- structures mobiles ;
- anneaux ;
- halos.

---
