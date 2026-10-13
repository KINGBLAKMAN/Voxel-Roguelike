# Emberwild

A voxel horde-survival roguelike for 1 to 4 players, in the spirit of Vampire Survivors. Pick one of four classes and fight an endless horde that rises from the ground all around you and grows every minute. Level up, stack relics and ability masteries, crack open urns and altars, and see how long you last. When everyone falls, the run is over.

## Play

Emberwild is a Windows desktop game. It opens fullscreen (F11 toggles windowed), and the pause menu has **Quit to desktop**.

**Download:** `Emberwild-<version>-portable.exe` (runs without installing) or `Emberwild-<version>-setup.exe` (installer). Every push builds both: open the **Actions** tab, pick the latest **Build Windows exe** run, and download *Emberwild-windows* under *Artifacts*. Tagged versions (`v*`) also attach them to a GitHub release.

**From source:** `npm install` then `npm start`. To build the exe yourself on Windows: `npm run dist:win` (output in `dist/`).

`index.html` still opens in a browser for quick testing, but hosting co-op and the fullscreen and quit controls need the desktop app.

## Controls

| Key | Action |
|---|---|
| WASD | Move |
| Mouse | Aim |
| Left mouse | Attack (hold to draw the bow) |
| 1 to 4 | Abilities |
| Q or mouse wheel | Swap weapon |
| 1 to 3, R | On level-up: pick an upgrade, reroll |
| Shift | Sprint |
| Space | Jump or swim |
| E | Interact: chests, shrines, berries; hold to revive a teammate |
| F | Eat |
| Esc | Pause and graphics settings |
| F11 | Fullscreen (desktop app) |
| M | Mute |

## Classes

| Class | Weapons | Abilities |
|---|---|---|
| Ranger | Longbow, Twin Daggers | Volley, Evasive Roll, Snare Trap, Arrow Storm |
| Mage | Fire Staff, Frost Wand | Blink, Frost Nova, Arcane Orb, Meteor |
| Warrior | Greatsword, Sword & Shield | Charge, Whirlwind, Ground Slam, Battle Cry |
| Priest | Holy Mace, Sacred Scepter | Mend, Smite, Divine Shield, Consecration |

Each level-up offers three picks (with a couple of rerolls per run) drawn from general boons, **relics**, and **upgrades for a specific ability or weapon**. Each ability has 3 upgrades plus a **Mastery** that unlocks once you've put 2 ranks into it, and each weapon has 3. Levelling up also blasts nearby enemies back. Some examples:

- **Splitting Arrows / Splitting Orb / Splitting Light:** projectiles burst into more projectiles on impact.
- **Prayer of Healing, Aegis, Rallying Cry, Sanctuary:** heal, shield or buff every teammate nearby.
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

Monsters don't wander: they hunt you. A director spawns packs in a ring just around the players (they claw up out of the ground), and both the headcount and their toughness climb with every minute you survive; the timer and threat level (Calm to Apocalypse) sit at the top of the screen. New kinds join as the run goes on: zombies and slimes first, then bats, skeletons, spiders, **bloaters** (they swell and burst; kill them in a crowd for chain reactions), cultists and club-swinging **brutes**. Stragglers that fall far behind are recycled to the front of the fight.

- Every 75 to 100 seconds there's an event: a full ring of the dead rising around you, a bat swarm from one direction, or an elite war band.
- Every 5 minutes an **Ancient Golem** boss arrives, and every third night is a **Blood Moon**.
- **Elites** carry affixes: Swift, Brute, Splitting, Vampiric and Volatile, and drop a treasure chest.
- Enemies drop experience gems (blue, green and red by value). Breakable **urns** and rare drops hold hearts, a Lodestone that pulls in every gem, and a Holy Relic that scours the horde.
- **Cursed altars** (red diamonds on the minimap): awaken one and survive 35 seconds of waves near it to earn a relic.

Enemy health scales with the number of players. To keep big fights smooth, the game caps enemies (130 hunting at once on High graphics, fewer on lower settings), projectiles, effects and pickups, and draws each enemy with a handful of merged meshes. It also lowers view distance, shadows and particles automatically when the frame rate drops. You can lock the graphics level in the pause menu.

## Graphics

The scene renders into an HDR buffer with bloom, sun shafts, a filmic tone curve, colour grading and FXAA, so fire, magic and glowing blocks actually glow. Every ability has its own cast pose and effects: magic circles, sweeping weapon trails, scorch and crack decals, ice and rock spikes, light flares and afterimages. **Low** graphics turns post-processing off; **Medium** keeps bloom and anti-aliasing but drops the sun shafts.

## The world

Gently rolling meadows, forests that open into glades, lakes with reeds and lily pads, deserts, tundra and rare mountain ranges. Ruins and camps hold chests, shrines bless you, and altars offer trials. The day and night cycle continues, and the music builds as the horde closes in.

## Co-op

1. **Host:** in the desktop app, enter a name and click **Host a game**. The menu shows your LAN address, such as `192.168.1.20:7777`. Windows asks once to allow Emberwild through the firewall; allow it.
2. **Friends:** enter that address and click **Join**, then pick a class.
3. To play over the internet, forward TCP port 7777 on the host's router, or use a LAN tunnel such as Tailscale or ZeroTier.

The host runs the enemies and the world clock. Everyone shares the world, the XP from kills, and the day/night cycle; loot from chests is per player. A downed player can be revived by a teammate holding E, and everyone gets back up at dawn. You can also run a standalone relay on any machine with `npm run relay` and have everyone join it; the first player to connect becomes the host.

## Layout

| Path | What |
|---|---|
| `index.html` | Menus, HUD and styles |
| `src/core.js` | World generation, chunk meshing, audio, particles and effects, models, class data |
| `src/render.js` | Post-processing: HDR, bloom, sun shafts, tone mapping, grading, FXAA |
| `src/vfx.js` | Procedural effect textures and the ability effect primitives |
| `src/game.js` | Players, abilities and upgrades, enemies, pickups, co-op networking, HUD and game flow |
| `src/relics.js` | Auto-firing relic weapons |
| `src/augments.js` | Chaos augments |
| `src/director.js` | Horde spawning, events, bosses, urns, altar trials and the generative music |
| `vendor/three.min.js` | three.js r128 |
| `desktop/` | Electron shell (`main.js`, `preload.js`) and the co-op relay (`relay.js`) |
| `.github/workflows/windows-build.yml` | Builds the portable exe and the installer on Windows |
