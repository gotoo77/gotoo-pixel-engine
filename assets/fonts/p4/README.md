# P4 font gallery assets

Twenty distinct families from https://github.com/google/fonts, distributed under
the SIL Open Font License. Each family directory includes its original OFL.txt.
Files are unmodified. `sources.json` records upstream revision, exact download
URL and SHA-256 for each file. Variable fonts use their default instance.

The assets are embedded only by the optional P4 gallery example. No installed
system fonts or network access are required to run it.

The maintainer import script is `scripts/fetch_p4_fonts.ps1`. Running it refreshes
assets from Google Fonts main and regenerates provenance; it is not a build step.

```powershell
rtk cargo run --features outline-fonts --example gpe_ui_p4_font_gallery
```

Click a family, use previous/next, or Left/Right to cycle all twenty families.
Up/Down selects a menu control; Space activates it. When SIZE has focus,
Left/Right adjusts size instead. SIZE also supports mouse click and drag.
The collection page follows the selected family. Escape closes the window.

Export one PNG per family without opening a window:

```powershell
rtk cargo run --features outline-fonts --example gpe_ui_p4_font_gallery -- target/p4-gallery
```
