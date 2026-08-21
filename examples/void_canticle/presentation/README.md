# Void Canticle presentation

This directory contains the current presentation layer.

Rules:

- use semantic module, type, function, constant and test names only;
- do not introduce or retain `vXX`, `VcXX`, `VCXX` or version-numbered identifiers here;
- historical version names belong only under `legacy/` while that code still exists;
- current presentation must compile outside the historical nested module graph;
- prefer a narrow gameplay-facing boundary over direct `game.game.game...` traversal.
