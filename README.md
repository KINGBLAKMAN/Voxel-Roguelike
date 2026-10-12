# Emberwild

A voxel roguelike survival game for 1 to 4 players. Pick one of four classes, explore an endless generated world, loot ruins and camps, level up your abilities, and last as many nights as you can. When everyone falls, the run is over.

## Play

**Windows:** download `Emberwild-<version>-portable.exe` (no install) or `Emberwild-<version>-setup.exe` from the latest run of the **Build Windows exe** workflow (Actions tab, then the run, then *Artifacts*), or from a GitHub release.

**Browser:** open `index.html`. Single player works fully, and the browser can join a co-op game.

**From source:** `npm install` then `npm start`.

## Controls

| Key | Action |
|---|---|
| WASD | Move |
| Mouse | Aim |
| Left mouse | Attack (hold to draw the bow) |
| 1 to 4 | Abilities |
| Q or mouse wheel | Swap weapon |
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

Each level-up offers three picks. They mix general boons (damage, cooldowns, lifesteal and so on) with **upgrades for a specific ability or weapon**. Each ability has 3 upgrades and each weapon has 2, and most stack in ranks. Some examples:

- **Splitting Arrows / Splitting Orb / Splitting Light:** projectiles burst into more projectiles on impact.
- **Prayer of Healing, Aegis, Rallying Cry, Sanctuary:** heal, shield or buff every teammate nearby.
- **Meteor Shower, Chain Smite, Aftershock, Shatterburst:** one cast becomes several.
- **Cluster Fire, Explosive Snare, Vortex, Decoy:** change how an ability plays.

## Difficulty

Enemies get tougher every day. From day 2, **elites** appear with affixes: Swift, Brute, Splitting (bursts into more enemies), Vampiric (heals by hitting you) and Volatile (explodes when it dies). Every third night is a **Blood Moon** with more spawns and an Ancient Golem boss. Enemy health scales with the number of players.

To keep big fights smooth, the game caps enemies (56), projectiles (180), effects, pickups and traps. It also lowers view distance, shadows and particles automatically when the frame rate drops. You can lock the graphics level in the pause menu.

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
| `src/game.js` | Players, abilities and upgrades, enemies, co-op networking, HUD and game flow |
| `vendor/three.min.js` | three.js r128 |
| `desktop/` | Electron shell (`main.js`, `preload.js`) and the co-op relay (`relay.js`) |
| `.github/workflows/windows-build.yml` | Builds the portable exe and the installer on Windows |
