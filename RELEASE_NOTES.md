## Emberwild 0.7.0: the visual overhaul

**Download:** `Emberwild-0.7.0.exe` is the whole game in one file; no installer. Windows may show a SmartScreen warning because the exe isn't code-signed: click *More info*, then *Run anyway*.

### A new look
- **Lighting:**
  - Screen-space ambient occlusion.
  - SMAA anti-aliasing.
  - Softer cascaded sun shadows.
  - A thin haze around you that the sun turns into god rays through the trees.
  - Warm sun glow in the distance fog.
  - Filmic tone mapping with gentle colour grading.
- **Sky:** painted, drifting cumulus clouds that catch the sun and glow at their edges, replacing the box clouds.
- **Textured world:**
  - Every block has a pixel-art texture with surface relief and roughness, from grass with blades and leaf litter to cracked stone, bark, log rings, bricks, planks, magma and runes.
  - Grass and snow hang over the edges of cliffs.
- **Plants:**
  - Grass, flowers, ferns, reeds and lily pads are textured and sway in the wind.
  - **Bushes are bushes:** round clumps of leaves, with red berries on berry bushes.
  - Tree crowns have ragged, leafy edges instead of cube outlines.
  - Leaves cast leaf-shaped, dappled shadows.
- **Ambient life:** fireflies over the meadows at night, pollen glinting in sunbeams, and leaves drifting down in forests.

### Animation
- **Bodies:** characters have bevelled edges and real elbows and knees, and walk with bent knees, monsters included.
- **Every weapon, every attack:**
  - Each attack winds up, strikes at the moment the hit lands, follows through and recovers.
  - Melee weapons chain three different blows with a heavy finisher:
    - Greatsword: diagonal cut, rising backswing, overhead slam.
    - Sword and shield: forehand, backhand, lunging thrust.
    - Daggers: right slash, left slash, scissor cut.
    - Mace: overhead smash, side swing, uppercut.
  - Staffs thrust or cast overhead, the wand flicks and the scepter points.
  - The bow is raised to aim as you start drawing, the string hand pulls back as the charge builds, and the bow kicks on release.
- **Smooth:** every joint blends into its next pose, so nothing snaps, and blades leave ribbon trails. Attacks speed up with your attack-speed upgrades.

### New tools
- **Balance bot:** run `Emberwild.exe --bot` from a command prompt.
  - The game plays full runs by itself, faster than real time, and writes every run to `balance-<time>.csv` plus a per-class summary: win rate, survival time, kills, damage taken and healed per minute.
  - Options, for example: `--bot "runs=5;classes=warrior,mage;minutes=20;turbo=8;ember=3"`.
  - Your save is never touched.
- **Crash log:** if the game ever crashes, it writes `crash-<time>.log` to `%APPDATA%/Emberwild/`. Send that file with a bug report.

### Settings
- **New Graphics quality option** (Low, Medium, High, Ultra), which scales ambient occlusion, god rays and shadow resolution. If the new look runs slowly on your machine, drop it to Medium or Low.
