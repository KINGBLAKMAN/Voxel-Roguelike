# Emberwild

A voxel horde-survival roguelike in the spirit of Vampire Survivors. Pick one of four classes and survive 20 minutes against a horde that rises from the ground all around you and grows every minute. A boss arrives every five minutes, and the Ember King waits at 20:00; beat him to win the run (or carry on in endless mode). Level up, evolve relics, dodge and parry, take on events, and spend the gold you bring home at the Camp on permanent upgrades. Play alone or with up to three friends.

## Play

Emberwild is a native Windows game written in Rust on the [Bevy](https://bevyengine.org) engine. It opens fullscreen; set `EMBERWILD_WINDOWED=1` to run it in a window. The pause menu has **Quit to desktop**.

**Download:** each release is a pair of zips: `Emberwild-<version>-windows.zip` holds the game (a single exe, no installer), and `Emberwild-<version>-windows-dev.zip` holds the same game plus the developer tools (the balance bot and the scripted test hooks). To build them yourself, run `rust/build-windows.sh`, which cross-compiles both exes and writes the zips to `rust/dist/`.

**From source:** install Rust, then `cd rust && cargo run --release`. On Linux you need the usual Bevy packages (`libasound2-dev libudev-dev libxkbcommon-x11-0`).

The original browser version is still in `index.html` and `src/`, with its Electron wrapper in `desktop/`.

## Controls

| Key | Action |
|---|---|
| WASD | Move |
| Mouse | Aim |
| Left mouse | Attack (hold to draw the bow) |
| 1 to 4 | Abilities |
| Q | Swap weapon |
| 1 to 3, R | On level-up: pick an upgrade, reroll |
| Shift | Sprint |
| Space | Jump or swim |
| Ctrl or Alt | Dodge roll (perfectly timed: refunds stamina and guarantees crits briefly) |
| Right mouse | Block; raise it just before a hit to parry |
| E | Interact: chests, shrines, altars, totems, the merchant; hold to revive a teammate |
| F | Eat |
| Esc | Pause |
| M | Mute |
| Tab | Character panel: stats, abilities and every upgrade as icons (hover for details) |
| C, S, H, J | On the title screen: Camp, Settings, Host co-op, Join co-op |

Gamepads work too: left stick moves, right stick looks, RT attacks, LT blocks, A jumps, B dodges, X, Y, LB and RB cast, and D-pad right opens the character panel. Hover an ability slot whenever the cursor is free (pause, level-up, character panel) for a tooltip with its cooldown and upgrades. Sensitivity, volume, field of view, view distance, shadows, screen shake and more are under **Settings**.

## Runs, bosses and the Camp

- A run lasts 20 minutes. The **Ancient Golem**, **Bone Colossus** and **Broodmother** arrive at 5, 10 and 15 minutes, and the three-phase **Ember King** at 20:00. Killing him wins the run, after which you can keep going in endless mode.
- Something happens every 70 to 100 seconds: a timed challenge, a cursed shrine (a minute of a nastier horde in exchange for a relic), a war totem to defend, or a treasure goblin that drips gold as it flees. A **wandering merchant** visits every few minutes and sells relic caches, healing, rerolls and more for run gold.
- **Relic evolutions:** a max-rank relic plus the matching boon evolves into something much stronger (Storm of Swords, Storm Throne, Heart of the Sun, Choir of Spirits, Reaper's Cyclone, Inferno Cask, Absolute Winter).
- Runs pay out gold. At the **Camp** it buys permanent upgrades (health, damage, speed, cooldowns, armour, XP, pickup range, gold, rerolls and a Second Chance) and unlocks the Mage, the Priest and three relics.
- **Ember 1 to 10:** each win unlocks the next difficulty level, and each level adds a twist. Pick it on the title screen.
- Progress, settings and your last 30 runs are saved in `%APPDATA%/Emberwild/save.json`.

## Classes

| Class | Weapons | Abilities |
|---|---|---|
| Ranger | Longbow, Twin Daggers | Volley, Evasive Roll, Snare Trap, Arrow Storm |
| Mage | Fire Staff, Frost Wand | Blink, Frost Nova, Arcane Orb, Meteor |
| Warrior | Greatsword, Sword & Shield | Charge, Whirlwind, Ground Slam, Battle Cry |
| Priest | Holy Mace, Sacred Scepter | Mend, Smite, Divine Shield, Consecration |

Each level-up offers three picks (with a couple of rerolls per run) drawn from general boons, **relics**, and **upgrades for a specific ability or weapon**. Each ability has 3 upgrades plus a **Mastery** that unlocks once you've put 2 ranks into it, and each weapon has 3. Levelling up also blasts nearby enemies back. Some examples:

- **Splitting Arrows / Splitting Orb / Splitting Light:** projectiles burst into more projectiles on impact.
- **Meteor Shower, Chain Smite, Aftershock, Shatterburst:** one cast becomes several.
- **Cluster Fire, Explosive Snare, Vortex, Decoy:** change how an ability plays.
- **Masteries** transform an ability: Hail of Arrows (Volley fires three waves), Mirror Image, Absolute Zero, Singularity, Armageddon, Bladestorm, Fissures, Warlord, Lifebloom, Judgment, Holy March and more.

## Chaos augments

Chaos augments bend the rules of the whole run rather than strengthening one ability, and they stack into mayhem: **Chain Reaction** (kills explode), **Echo** (abilities recast themselves), **Arcane Surge** (every cast fires seeking bolts), **Stormcaller** (crits call lightning), **Frenzy** (kills cut cooldowns), **Gravity Well** (black holes swallow the horde), **Overload** (every 5th cast is a huge nova), **Ricochet** (projectiles bounce), **Phoenix Heart** (cheat death once) and **Berserker** (stronger at low health).

## Relics

Relics are auto-firing weapons any class can pick up. You can carry 4, and each has 5 ranks.

| Relic | What it does |
|---|---|
| Whirling Blades | Blades orbit you and cut what they touch |
| Thunder Call | Lightning strikes nearby enemies, later chaining |
| Ember Aura | A burning ring around you that sears and slows |
| Spirit Wisp | A companion that fires seeking bolts |
| Ricochet Axe | Boomerang axes that fly out through the horde and return |
| Firebomb Flask | Lobbed flasks that leave burning pools |
| Frost Halo | Periodic rings of ice shards that chill and freeze |

## The horde

Monsters don't wander: they hunt you. A director spawns packs in a ring just around the players (they claw up out of the ground), and both the headcount and their toughness climb with every minute you survive; the timer and threat level (Calm to Apocalypse) sit at the top of the screen. New kinds join as the run goes on: zombies and slimes first, then bats, skeletons, spiders, **bloaters** (they swell and burst; kill them in a crowd for chain reactions), cultists and club-swinging **brutes**, then monsters with jobs: **Gorehorn** chargers, **Shieldbearers** that block hits from the front, **Bone Shamans** that heal and haste the horde, **Ember Imps** that bomb from above and **Necromancers** that raise the dead. Deserts have **Sand Wurms** that tunnel and erupt under you, and the snow has **Frost Giants** whose slams chill. Stragglers that fall far behind are recycled to the front of the fight.

- Every 75 to 100 seconds there's an event: a full ring of the dead rising around you, a bat swarm from one direction, or an elite war band.
- Every third night is a **Blood Moon**, and from minute 6 telegraphed eruptions land where you're heading.
- **Elites** carry affixes: Swift, Brute, Splitting, Vampiric and Volatile, and drop a treasure chest.
- Enemies drop experience gems (blue, green and red by value). Breakable **urns** and rare drops hold hearts, a Lodestone that pulls in every gem, and a Holy Relic that scours the horde.
- **Cursed altars** (red diamonds on the minimap): awaken one and survive 35 seconds of waves near it to earn a relic.

Enemy health scales with the number of players. To keep big fights smooth, the game caps enemies (130 hunting at once on High graphics, fewer on lower settings), projectiles, effects and pickups, and draws each enemy with a handful of merged meshes. It also lowers view distance, shadows and particles automatically when the frame rate drops. You can lock the graphics level in the pause menu.

## Graphics

Everything is generated in code; there are no art files.

- **Renderer:** HDR with bloom, SMAA anti-aliasing, screen-space ambient occlusion, cascaded soft sun shadows, and a thin haze around the camera that the sun lights into god rays through the trees. Tone mapping is filmic (TonyMcMapface) with gentle colour grading.
- **Sky:** a painted sky with drifting cumulus clouds that catch the sun, plus stars and the moon at night.
- **Blocks:** every block face has a 32px pixel-art texture with a matching normal and roughness map, all painted at startup. That covers grass with blades, cracked stone, bark, leaves, bricks, planks, magma and more.
- **Plants:** grass, flowers, ferns, reeds and lily pads are textured cut-out cards that sway in the wind. Bushes are domes of leaf clusters (berry bushes carry berries). Tree crowns have leafy fringes instead of cube edges, and all foliage casts leaf-shaped shadows.
- **Ambient life:** fireflies drift over meadows at night, pollen glints in sunbeams, and leaves fall in forests.
- **Characters:**
  - Bevelled parts, with elbows and knees.
  - Every weapon has keyframed attacks: a wind-up, a strike timed to the hit, a follow-through and a recovery. Melee weapons chain three different attacks, with a heavy third blow.
  - The bow draws back as it charges and kicks on release. Staffs thrust, wands flick and the scepter points.
  - Joints blend smoothly into each pose, and blades leave ribbon trails.
  - Running: hips turn towards where you're going while the chest keeps facing your aim, knees lift and absorb, the body leans into speed and banks into turns, free arms pump, and landings squash.
- **Effects:** anime-style rather than spell-circle magic. That means impact flashes with starbursts of speed lines, crescent slash marks, speed lines on dashes, sparkle glints, cel-shaded energy, sparks that streak, and a lens punch on big hits.

**Settings → Graphics quality:**

| Level | Ambient occlusion | God rays | Shadows |
|---|---|---|---|
| Low | none | none | small shadow maps |
| Medium | light | none | standard |
| High (default) | yes | yes | standard |
| Ultra | higher quality | longer | 4K shadow maps |

## Tools

- **Crash log:** if the game crashes, it writes `crash-<time>.log` (the error and a backtrace) to `%APPDATA%/Emberwild/`. Send that file along with a bug report.
- **Balance bot** (dev build only): run `Emberwild-<version>-dev.exe --bot` from a command prompt.
  - The game plays complete runs on its own in a small window, simulating several steps per frame so runs go much faster than real time. It aims, kites, draws the bow, casts abilities, dodges, eats, opens chests and picks level-up cards.
  - Each run is appended to `balance-<time>.csv` in the current folder, and a per-class summary goes to `balance-<time>-summary.txt`. The summary covers win rate, survival time, level, kills per minute, damage taken and healed per minute, and the heal-to-damage ratio.
  - Your save is never touched.
  - Options go in one quoted string, for example: `Emberwild.exe --bot "runs=5;classes=warrior,mage;minutes=20;turbo=8;ember=3"`.
    - `runs`: runs per class.
    - `classes`: any of ranger, mage, warrior, priest.
    - `minutes`: the run cap.
    - `turbo`: steps per frame.
    - `ember`: difficulty.
    - `seed`: the base world seed. Each run's world is derived from it, so a change can be compared on the same worlds.

## The world

Gently rolling meadows, forests that open into glades, lakes with reeds and lily pads, deserts, tundra and rare mountain ranges. Ruins and camps hold chests, shrines bless you, and altars offer trials. The day and night cycle continues, and the music builds as the horde closes in.

## Co-op

1. **Host:** on the title screen, click **Host co-op** (or press H). The title screen shows your LAN address, such as `192.168.1.20:7777`. Windows asks once to allow Emberwild through the firewall; allow it. Then pick a class.
2. **Friends:** click **Join co-op** (or press J), type that address (the port can be left off) and press Enter. You load into the host's world; pick a class.
3. To play over the internet, forward TCP port 7777 on the host's router, or use a LAN tunnel such as Tailscale or ZeroTier.

Up to four players. The host runs the horde, the bosses, events and the clock; everyone sees each other's spells and shares the XP from kills. Gold, loot and Camp progress are your own. A downed player can be revived by a teammate holding E next to them; the run ends when everyone is down. The game doesn't pause during co-op.

## Layout

| Path | What |
|---|---|
| `rust/src/main.rs` | App setup, the game tick, run start/restart, crash log |
| `rust/src/scene.rs`, `rust/src/shaders/` | Camera and post-processing, sky, water, terrain and foliage materials (WGSL), day/night, chunk streaming |
| `rust/src/textures.rs`, `rust/src/mesher.rs`, `rust/src/world.rs` | Procedural block and foliage textures, chunk meshing (faces, bushes, canopy fringes, plants), world generation |
| `rust/src/models.rs`, `rust/src/player.rs`, `rust/src/anim.rs` | Box-built models, the player controller, camera, joint blending, weapon trails, and the keyframed weapon animations |
| `rust/src/combat.rs`, `abilities.rs`, `relics.rs`, `augments.rs` | Weapons, the 16 abilities, relics and evolutions, chaos augments |
| `rust/src/enemies.rs`, `horde.rs`, `director.rs`, `events.rs` | Monsters and their AI, bosses, the horde director, run events and the merchant |
| `rust/src/hud.rs`, `camp.rs`, `settings.rs`, `meta.rs` | Menus and HUD, the Camp, settings and gamepad, the save file |
| `rust/src/net.rs` | Co-op |
| `rust/src/bot.rs`, `testkit.rs` | The balance bot and the scripted screenshot tests |
| `rust/build-windows.sh` | Builds the player and dev Windows exes and zips them |
| `index.html`, `src/`, `desktop/` | The original browser version |
