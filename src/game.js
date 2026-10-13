'use strict';
/* Emberwild gameplay: players, abilities and their upgrades, enemies, co-op networking, HUD and game flow.
   Everything a player does goes through an "actor" (the local player P, or a remote player's proxy), so the
   same ability code drives local play and replays a teammate's casts. Only the local actor deals real damage;
   in co-op the host owns enemies and clients report their hits to it. */

/* ---------------- ability & weapon upgrades ---------------- */
const MODS={
  volley:[['more','Splinter Volley','+3 arrows per volley.',3],['split','Splitting Arrows','Every volley arrow splits into 3 when it hits.',1],['fire','Incendiary Volley','Volley arrows set enemies ablaze.',1],['hail','Hail of Arrows','Mastery: every Volley fires two more waves.',1,1]],
  roll:[['cd','Nimble','Roll recharges 22% faster.',2],['shot','Parting Shot','Fire a ring of 10 arrows as you roll.',1],['decoy','Decoy','Leave a decoy that draws enemies for 4s.',1],['dance','Blade Dance','Mastery: rolling through enemies slashes them, and each hit refunds 0.4s of cooldown.',1,1]],
  snare:[['twin','Twin Snares','Throw +1 trap at once.',2],['boom','Explosive Snare','Traps explode, hitting everything nearby.',1],['spiked','Serrated Jaws','+60% trap damage and +1s root.',2],['field','Minefield','Mastery: sprung traps scatter 4 mines around them.',1,1]],
  storm:[['wide','Wider Storm','+25% storm radius.',2],['long','Downpour','Storm lasts 1.5s longer.',2],['frost','Frost-tipped','Storm arrows slow enemies.',1],['eye','Eye of the Storm','Mastery: the storm follows you and lasts 2s longer.',1,1]],
  blink:[['far','Far Step','Blink 5m further.',1],['wake','Arcane Wake','Blink detonates where you leave and land.',1],['cd','Flicker','Blink recharges 22% faster.',2],['mirror','Mirror Image','Mastery: blinking leaves an image that fires arcane bolts for 4s.',1,1]],
  nova:[['wide','Wider Nova','+25% nova radius.',2],['deep','Deep Freeze','Freeze lasts 1.2s longer.',2],['shards','Shatterburst','Nova fires 10 ice shards outward.',2],['zero','Absolute Zero','Mastery: Frost Nova pulses two more times, each wider.',1,1]],
  orb:[['split','Splitting Orb','The orb bursts into 3 smaller orbs (5 at rank 2).',2],['twin','Twin Orbs','Cast two orbs at once.',1],['big','Greater Orb','The orb is 35% larger and stronger.',2],['sing','Singularity','Mastery: the orb drags enemies into its path and collapses in a bigger blast.',1,1]],
  meteor:[['shower','Meteor Shower','+2 smaller meteors follow the first.',2],['inferno','Inferno','Impact leaves burning ground for 5s.',1],['cata','Cataclysm','+40% meteor damage and +20% radius.',2],['arma','Armageddon','Mastery: meteors keep raining on enemies around you for 4s.',1,1]],
  charge:[['mom','Momentum','Charge travels 50% further.',1],['wave','Shockwave','Charge ends in a stunning shockwave.',1],['cd','Relentless','Charge recharges 22% faster.',2],['ruin','Trail of Ruin','Mastery: Charge leaves a blazing trail and ends in an eruption.',1,1]],
  whirl:[['long','Cyclone','Whirlwind lasts 1.5s longer.',2],['vortex','Vortex','Whirlwind drags enemies inward.',1],['wide','Wide Arc','+25% whirlwind radius.',2],['blades','Bladestorm','Mastery: Whirlwind hurls spinning blades outward.',1,1]],
  slam:[['after','Aftershock','+1 follow-up shockwave.',2],['wide','Earthsplitter','+30% slam radius.',2],['quake','Quake','Slam stuns for 1s longer.',1],['fissure','Fissures','Mastery: the slam tears 6 erupting fissures through the ground.',1,1]],
  cry:[['rally','Rallying Cry','Allies within 15m also gain Battle Cry and healing.',1],['terror','Terrify','Enemies nearby flee for 3s.',1],['rage','Bloodrage','+2s duration and +10% lifesteal.',2],['warlord','Warlord','Mastery: during Battle Cry every swing launches a shockwave.',1,1]],
  mend:[['prayer','Prayer of Healing','Mend also heals every ally within 18m.',1],['greater','Greater Mend','Mend heals 15% more.',2],['renew','Renew','Healing over time lasts twice as long.',1],['bloom','Lifebloom','Mastery: Mend plants a blossom that heals allies and burns foes for 6s.',1,1]],
  smite:[['chain','Chain Smite','+2 smites strike nearby enemies.',2],['wrath','Wrath','+40% smite damage.',2],['cd','Swift Judgment','Smite recharges 22% faster.',2],['judg','Judgment','Mastery: Smite strikes the same spot twice more.',1,1]],
  shield:[['aegis','Aegis','Also shields allies within 12m.',1],['retri','Retribution','The shield ends in a blast of light.',1],['long','Lasting Grace','Shield lasts 1.5s longer.',2],['thorns','Thorns of Light','Mastery: while shielded, you scorch every enemy within 4.5m.',1,1]],
  consecrate:[['wide','Hallowed Ground','+25% radius.',2],['purge','Purge','+50% damage.',2],['sanct','Sanctuary','Doubles healing and heals allies standing inside.',1],['march','Holy March','Mastery: Consecration follows you and lasts 3s longer.',1,1]],
};
const WMODS={
  longbow:[['multi','Multishot','Fire 2 extra arrows at half damage.',2],['split','Splinter Tips','Fully drawn arrows split into 3 on hit.',1],['blast','Hunter\'s Mark','Fully drawn arrows explode where they land.',1]],
  daggers:[['venom','Venom','Hits poison enemies.',1],['flurry','Flurry','Attack 20% faster.',2],['shadow','Shadow Strikes','Every third strike also hurls a spectral dagger.',1]],
  firestaff:[['cluster','Cluster Fire','Explosions scatter 3 bomblets (+2 per rank).',2],['big','Conflagration','+30% explosion radius.',2],['trail','Phoenix Fire','Fireballs leave burning ground.',1]],
  frostwand:[['fork','Forked Ice','Fire 2 extra shards.',2],['brittle','Brittle','Targets freeze after 3 hits instead of 5.',1],['lance','Glacial Lance','Shards pierce through every enemy.',1]],
  greatsword:[['wave','Sword Wave','Swings launch a piercing wave.',1],['exec','Executioner','+60% damage to enemies below 35% health.',1],['reach','Titan\'s Reach','+30% reach and a wider arc.',1]],
  swordshield:[['bash','Shield Bash','Every 3rd swing stuns.',1],['bulwark','Bulwark','The shield blocks 10% more damage.',2],['riposte','Riposte','Taking a hit blasts enemies around you (every 2s).',1]],
  mace:[['nova','Holy Nova','Every 4th hit bursts with light, healing you and allies.',1],['heavy','Heavy Blows','+25% mace damage.',2],['thunder','Thunder Mace','Every third hit calls lightning on the target.',1]],
  scepter:[['split','Splitting Light','Bolts split into 3 seekers on hit.',1],['twin','Twin Bolts','Fire +1 bolt.',2],['nova','Radiant Bolts','Bolts burst in holy light on impact.',1]],
};
const M=(a,k)=>(a.mods&&a.mods[k])||0;
const UP=new V3(0,1,0);
const AFFIX=[{k:'swift',n:'Swift',c:0x7fd4ff},{k:'brute',n:'Brute',c:0xff7a3a},{k:'splitter',n:'Splitting',c:0x9be07a},{k:'vampiric',n:'Vampiric',c:0xff3a5a},{k:'volatile',n:'Volatile',c:0xffd35a}];

/* ---------------- game state ---------------- */
let world=null;
const G={state:'menu',ui:null,time:0.06,day:1,night:false,blood:false,kills:0,runT:0,hitstop:0,shake:0,seed:0,noLock:false,bossAnnounced:-1,spawnT:0,lastNight:false,over:false,trial:null,flash:0};
const P={local:true,id:0,name:'You',pos:new V3(),vel:new V3(),w:0.3,h:1.8,onGround:false,yaw:0,pitch:-0.15,visY:0,inWater:false,hp:100,maxHp:100,ghostHp:100,stam:100,hunger:100,lvl:1,xp:0,
  cls:'ranger',widx:0,cds:[0,0,0,0],atkT:0,charge:0,charging:false,invuln:0,model:null,weaponMeshes:[],buffs:{},food:{meat:0,berries:0},kills:0,dash:null,leap:null,
  swingT:1,swingDur:0.3,swingSide:1,castKick:0,casting:0,whirl:0,hits:0,coyote:0,dead:false,deathT:0,regenAcc:0,hitT:0,firing:false,
  st:null,mods:{},perks:[],walkPh:0,aimPt:new V3(),hand:new V3(),moveDir:new V3(),cryLeech:0.1,revive:0,rerolls:2,rollT:0,rel:null};
function baseStats(){return{dmg:1,cd:1,speed:1,leech:0,crit:0.06,armor:0,regen:0,food:1,elite:0,magnet:1,area:1,extraProj:0,chargeMul:1,fireRad:1,shatter:0,wwHeal:0,radiance:0,atkSpd:1,stamMax:100,stamRegen:1,xpMul:1};}
let ents=[],projs=[],pickups=[],zones=[],traps=[],timers=[],stuck=[],chests=[],decoys=[];
let ACTOR=P;
function withActor(a,fn){const prev=ACTOR;ACTOR=a;try{return fn();}finally{ACTOR=prev;}}
function later(t,fn){timers.push({t,fn,a:ACTOR});}
const canAct=()=>G.state==='playing'&&!G.ui&&!P.dead;
const AUTH=()=>!NET.on||NET.host;

/* ---------------- networking (co-op) ---------------- */
const NET={on:false,host:false,id:0,hostId:0,ws:null,peers:new Map(),eid:1,psT:0,snapT:0,lastMods:'',addr:'',welcomed:false,hostIps:null};
const r2=v=>Math.round(v*100)/100;
const pk=v=>[r2(v.x),r2(v.y),r2(v.z)];
function send(m){if(NET.ws&&NET.ws.readyState===1)NET.ws.send(JSON.stringify(m));}
function netConnect(url){
  return new Promise((res,rej)=>{let ws;try{ws=new WebSocket(url);}catch(e){rej(e);return;}
    const to=setTimeout(()=>{try{ws.close();}catch(e){}rej(new Error('timeout'));},6000);
    ws.onopen=()=>{clearTimeout(to);NET.ws=ws;NET.on=true;res();};
    ws.onerror=()=>{clearTimeout(to);rej(new Error('connect failed'));};
    ws.onclose=()=>{if(NET.ws===ws)netLost();};
    ws.onmessage=e=>{let m;try{m=JSON.parse(e.data);}catch(err){return;}try{onNet(m);}catch(err){console.error('net',m.t,err);}};});
}
function netLost(){const was=NET.on;NET.on=false;NET.ws=null;for(const r of NET.peers.values())dropPeer(r);NET.peers.clear();coopStatus('');
  if(was&&G.state==='playing'){toast('Disconnected','The co-op session ended',3);if(!NET.host)for(const e of ents.filter(e=>e.net)){removeEnt(e);}ents=ents.filter(e=>!e.net);}
  NET.host=false;renderCoop();}
function netLeave(){if(NET.ws){const ws=NET.ws;NET.ws=null;try{ws.close();}catch(e){}}netLost();if(window.native&&window.native.stopHost)window.native.stopHost();}
function peerOf(id){let r=NET.peers.get(id);if(!r){r={local:false,id,name:'Player '+id,pos:new V3(),tpos:new V3(),vel:new V3(),w:0.3,h:1.8,onGround:true,yaw:0,tyaw:0,pitch:0,cls:null,widx:0,model:null,weaponMeshes:[],
    st:baseStats(),mods:{},buffs:{},aimPt:new V3(),hand:new V3(),moveDir:new V3(),dead:false,hp:100,maxHp:100,lvl:1,walkPh:0,swingT:1,swingDur:0.3,swingSide:1,castKick:0,casting:0,whirl:0,
    charging:false,charge:0,firing:false,hits:0,inGame:false,deathT:0,cryLeech:0.1,tag:null};NET.peers.set(id,r);}return r;}
function dropPeer(r){if(r.model)scene.remove(r.model.root);if(r.tag)r.tag.remove();clearRelics(r);}
function onNet(m){
  switch(m.t){
    case'id':NET.id=m.id;P.id=m.id;NET.hostId=m.host;NET.host=m.id===m.host;
      if(NET.host){NET.welcomed=true;coopStatus(hostLine());}else{send({t:'hello',n:P.name});coopStatus('Connected. Waiting for the host…');}renderCoop();break;
    case'join':if(NET.host)coopStatus(hostLine());break;
    case'hello':if(!NET.host)return;{const r=peerOf(m.from);r.name=cleanName(m.n);
      send({t:'welcome',to:m.from,seed:G.seed,time:G.time,day:G.day,hp:G.state==='playing'?pk(P.pos):null,hn:P.name});toast(r.name+' joined','',2);renderCoop();renderParty();}break;
    case'welcome':NET.welcomed=true;{const h=peerOf(m.from);h.name=cleanName(m.hn);}
      NET.spawnAt=m.hp?new V3().fromArray(m.hp):null;
      {const done=()=>{G.time=m.time;coopStatus(`Joined ${cleanName(m.hn)}'s world. Pick a class to drop in.`);renderCoop();};
        if(G.seed!==m.seed){NET.welcomed=false;const L=$('#loading');L.firstChild.textContent='Joining world…';L.hidden=false;
          setTimeout(()=>{newWorld(m.seed);L.hidden=true;NET.welcomed=true;done();},40);}else done();}
      break;
    case'newworld':if(!NET.host){newWorld(m.seed);G.time=0.06;G.over=false;NET.spawnAt=null;if(G.state!=='menu'){$('#death').hidden=true;toMenu(false);}coopStatus('The host started a new world. Pick a class.');}break;
    case'ps':onPeerState(m);break;
    case'mods':{const r=peerOf(m.from);r.mods=m.mods||{};Object.assign(r.st,m.st||{});}break;
    case'cast':case'prim':onRemoteCast(m);break;
    case'hit':if(NET.host){const e=entById(m.id);if(e)applyDamage(e,m.a,{kb:m.kb||0,up:m.up||0,src:m.s?new V3(m.s[0],0,m.s[1]):null,by:m.from});}break;
    case'st':if(NET.host){const e=entById(m.id);if(e)applyStatus(e,m.s);}break;
    case'hurt':if(!P.dead)hurtPlayer(m.a,m.s?{x:m.s[0],z:m.s[1]}:null,{kb:m.kb});break;
    case'snap':if(!NET.host&&G.state==='playing')applySnap(m);else if(!NET.host)G.time=m.time;break;
    case'kill':if(!NET.host&&G.state==='playing')netKill(m);break;
    case'eproj':if(!NET.host&&G.state==='playing')visualEnemyProj(m);break;
    case'efx':if(!NET.host&&G.state==='playing'&&EFX[m.k])EFX[m.k](m);break;
    case'block':world.set(m.x,m.y,m.z,m.b);break;
    case'revive':if(P.dead&&!G.over)revivePlayer(0.5,cleanName(peerOf(m.from).name));break;
    case'trial':if(NET.host){const p=world.poi(m.gx,m.gz);if(p&&p.type==='altar')beginTrial(p,cleanName(peerOf(m.from).name));}break;
    case'over':if(!NET.host&&!G.over&&G.state==='playing'){G.over=true;P.dead=true;setTimeout(showDeath,1200);}break;
    case'leave':{const r=NET.peers.get(m.id);if(r){toast(r.name+' left','',2);dropPeer(r);NET.peers.delete(m.id);renderParty();renderCoop();}}break;
    case'hostleft':toast('The host left','Co-op session ended',3);netLeave();break;
  }
}
function cleanName(n){return String(n||'Player').replace(/[<>&"]/g,'').slice(0,16)||'Player';}
function hostLine(){const n=NET.peers.size;const ips=NET.hostIps;return `Hosting${ips&&ips.length?' on '+ips.map(i=>i+':'+NET.port).join(' or '):''}. ${n} friend${n===1?'':'s'} connected.`;}
function sendState(dt){
  NET.psT-=dt;if(NET.psT>0||G.state!=='playing'||!P.model)return;NET.psT=0.05;
  send({t:'ps',p:pk(P.pos),y:r2(P.yaw),pi:r2(P.pitch),c:P.cls,w:P.widx,hp:Math.ceil(P.hp),mh:P.maxHp,l:P.lvl,d:P.dead?1:0,ch:P.charging?r2(P.charge):0,wh:P.whirl>0?1:0,f:P.firing?1:0,n:P.name});
  const ms=JSON.stringify(P.mods)+JSON.stringify(P.st);if(ms!==NET.lastMods){NET.lastMods=ms;send({t:'mods',mods:P.mods,st:{area:P.st.area,extraProj:P.st.extraProj,fireRad:P.st.fireRad,radiance:P.st.radiance,wwHeal:P.st.wwHeal}});}
}
function onPeerState(m){const r=peerOf(m.from);r.tpos.fromArray(m.p);r.tyaw=m.y;r.pitch=m.pi;r.hp=m.hp;r.maxHp=m.mh;r.lvl=m.l;r.name=cleanName(m.n);
  const wasDead=r.dead;r.dead=!!m.d;if(wasDead&&!r.dead&&r.model){r.deathT=0;r.model.body.rotation.x=0;r.model.body.position.y=0;}
  r.charging=m.ch>0;r.charge=m.ch;if(m.wh)r.whirl=0.2;r.firing=!!m.f;
  if(!r.inGame||r.cls!==m.c){r.cls=m.c;r.widx=m.w;r.pos.copy(r.tpos);buildAvatar(r);r.inGame=true;renderParty();}
  else if(r.widx!==m.w){r.widx=m.w;attachWeapons(r);}}
function onRemoteCast(m){const r=NET.peers.get(m.from);if(!r||!r.model)return;
  r.pos.fromArray(m.p);r.tpos.copy(r.pos);r.yaw=r.tyaw=m.y;r.pitch=m.pi;r.hand.fromArray(m.h);r.aimPt.fromArray(m.a);r.moveDir.set(m.md[0],0,m.md[1]);
  withActor(r,()=>{if(m.t==='cast'){const f=ABILITY_FN[m.ab];f&&f(r);}else primaryFire(r,m.w,m.c);});}
function castMsg(t,extra){return Object.assign({t,p:pk(P.pos),y:r2(P.yaw),pi:r2(P.pitch),h:pk(P.hand),a:pk(P.aimPt),md:[r2(P.moveDir.x),r2(P.moveDir.z)]},extra);}
const ETL=Object.keys({zombie:1,skeleton:1,slime:1,spider:1,cultist:1,golem:1,pig:1,sheep:1,chicken:1,bat:1,bloater:1,brute:1,urn:1});
function entById(id){for(const e of ents)if(e.id===id)return e;return null;}
function sendSnap(dt){
  NET.snapT-=dt;if(NET.snapT>0)return;NET.snapT=1/15;const arr=[];
  for(const e of ents){if(e.dead)continue;const f=(e.wind>0?1:0)|(e.frozen>0?2:0)|(e.stun>0?4:0)|(e.aiming?8:0)|(e.burn>0||e.poison>0?16:0);
    arr.push([e.id,ETL.indexOf(e.type),r2(e.pos.x),r2(e.pos.y),r2(e.pos.z),r2(e.yaw),Math.ceil(e.hp),e.maxHp,f,r2(e.size),e.color,e.affix?AFFIX.indexOf(e.affix):-1,e.boss?1:0]);}
  send({t:'snap',time:G.time,e:arr});
}
function applySnap(m){
  if(Math.abs(m.time-G.time)>0.002)G.time=m.time;const seen=new Set();
  for(const q of m.e){const[id,ti,x,y,z,yaw,hp,mh,f,size,color,af,boss]=q;seen.add(id);let e=entById(id);
    if(!e){if(ents.length>=CAP.ents+10)continue;e=spawnEnt(ETL[ti],x,y,z,{net:true,id,size,color,affix:af>=0?AFFIX[af]:null,boss:!!boss});if(!e)continue;}
    e.tpos.set(x,y,z);e.yaw=yaw;e.hp=hp;e.maxHp=mh;e.wind=f&1?1:0;e.frozenFlag=!!(f&2);e.stun=f&4?1:0;e.aiming=!!(f&8);e.burnFlag=!!(f&16);}
  for(let i=ents.length-1;i>=0;i--){const e=ents[i];if(e.net&&!seen.has(e.id)){removeEnt(e);ents.splice(i,1);}}
}
function netKill(m){const e=entById(m.id);if(e){e.hp=0;e.dead=true;deathVisual(e);augKill(e,new V3(m.x,m.y,m.z));if(e.T.prop||e.type==='bloater')removeEnt(e);else toCorpse(e);ents.splice(ents.indexOf(e),1);}
  const c=new V3(m.x,m.y,m.z);if(!P.dead&&m.xp)dropGems(c,m.xp);if(!m.pa&&m.ty!=='urn')P.kills++;
  if(m.pa){const k=(m.ty==='chicken'?1:2)+(P.st.food>1?1:0);for(let i=0;i<k;i++)dropFood(c,'meat');}
  if(m.ty==='urn')urnLoot(c);else if(!m.pa&&!P.dead)rareDrop(c,m.el);
  if(m.el){toast((m.nm||'Elite')+' slain','+'+m.xp+' experience',2.5);dropChestAt(c);}}
function efx(k,m){if(EFX[k])EFX[k](m);if(NET.on&&NET.host){m.t='efx';m.k=k;send(m);}}
const EFX={
  tele:m=>fx.telegraph(new V3().fromArray(m.p),m.c,m.r,m.l),
  arc:m=>fx.arc(new V3().fromArray(m.p),m.y,m.r,1.6,0xff6a5a,0.18),
  slam:m=>golemSlamFx(new V3().fromArray(m.p),m.r),
  bloat:m=>bloatFx(new V3().fromArray(m.p),m.r),
  bolt:m=>fx.lightning(new V3().fromArray(m.a),new V3().fromArray(m.b),m.c||0xd07aff,0.3,0.3),
  smoke:m=>fx.smoke(new V3().fromArray(m.p),0x1a1420,6,1,1.2,1.2),
  boom:m=>volatileFx(new V3().fromArray(m.p),m.r),
  rock:m=>rockBreakFx(new V3().fromArray(m.p)),
  toast:m=>toast(m.a,m.b,3),
};

/* ---------------- input ---------------- */
const keys={};const mouse={l:false,r:false};
addEventListener('keydown',e=>{if(e.target&&e.target.tagName==='INPUT')return;if(e.repeat&&!/^Key[WASDE]$/.test(e.code))return;keys[e.code]=true;AU.init();
  if(G.state==='playing'&&!G.ui){
    if(!P.dead){if(e.code==='Digit1')useAbility(0);if(e.code==='Digit2')useAbility(1);if(e.code==='Digit3')useAbility(2);if(e.code==='Digit4')useAbility(3);
      if(e.code==='KeyQ')swapWeapon();if(e.code==='KeyE'&&!e.repeat)interact();if(e.code==='KeyF')eat();}
    if(e.code==='Escape'&&G.noLock)pauseGame();
    if(e.code==='Space'||e.code==='Tab')e.preventDefault();}
  if(e.code==='KeyM'){AU.muted=!AU.muted;toast(AU.muted?'Sound off':'Sound on','',1);}
});
addEventListener('keyup',e=>{keys[e.code]=false;});
addEventListener('blur',()=>{for(const k in keys)keys[k]=false;mouse.l=mouse.r=false;});
canvas.addEventListener('mousedown',e=>{AU.init();if(G.state!=='playing'||G.ui)return;if(!G.noLock&&document.pointerLockElement!==canvas){lockPointer();return;}
  if(e.button===0)mouse.l=true;if(e.button===2)mouse.r=true;});
addEventListener('mouseup',e=>{if(e.button===0)mouse.l=false;if(e.button===2)mouse.r=false;});
canvas.addEventListener('contextmenu',e=>e.preventDefault());
/* pointer lock sometimes reports a huge bogus jump (right after locking, or when the OS cursor warps), which would whip the camera around: drop those */
let lockT=0;
addEventListener('mousemove',e=>{if(G.state!=='playing'||G.ui||P.dead)return;const locked=document.pointerLockElement===canvas;if(!locked&&!(G.noLock&&mouse.r))return;
  if(performance.now()-lockT<120)return;const mx=e.movementX,my=e.movementY;if(Math.abs(mx)>260||Math.abs(my)>200)return;
  P.yaw-=mx*0.0024;P.pitch=clamp(P.pitch-my*0.0022,-1.25,1.05);});
addEventListener('wheel',e=>{if(canAct()&&Math.abs(e.deltaY)>20)swapWeapon();},{passive:true});
function lockPointer(){try{let r;try{r=canvas.requestPointerLock({unadjustedMovement:true});}catch(err){r=null;}if(r&&r.catch)r=r.catch(()=>canvas.requestPointerLock());else if(!r)r=canvas.requestPointerLock();if(r&&r.catch)r.catch(()=>{G.noLock=true;$('#lockhint').hidden=true;});}catch(e){G.noLock=true;}}
document.addEventListener('pointerlockchange',()=>{const locked=document.pointerLockElement===canvas;if(locked)lockT=performance.now();$('#lockhint').hidden=locked||G.state!=='playing'||G.noLock||P.dead||!!G.ui;
  if(!locked&&G.state==='playing'&&!G.noLock&&!G.ui){mouse.l=false;pauseGame();}});
document.addEventListener('pointerlockerror',()=>{G.noLock=true;$('#lockhint').hidden=true;toast('Mouse capture unavailable','Hold right mouse to look around',3);});

/* ---------------- avatars (local and remote players) ---------------- */
function buildAvatar(a){if(a.model)scene.remove(a.model.root);a.model=playerModel(a.cls);scene.add(a.model.root);a.model.root.position.copy(a.pos);attachWeapons(a);
  if(!a.local&&!a.tag){a.tag=document.createElement('div');a.tag.className='ntag';a.tag.innerHTML='<b></b><i><s></s></i>';fxLayer.appendChild(a.tag);}}
function attachWeapons(a){
  const m=a.model;if(!m)return;for(const w of a.weaponMeshes)w.parent&&w.parent.remove(w);a.weaponMeshes=[];
  const id=CLASSES[a.cls].weapons[a.widx];const mats=m.mats;
  if(id==='longbow'){const b=WMODEL.longbow(mats);m.handL.add(b);a.weaponMeshes.push(b);a.tip=b.userData.tip;}
  else if(id==='daggers'){const x=WMODEL.daggers(mats),y=WMODEL.daggers(mats);m.handR.add(x);m.handL.add(y);a.weaponMeshes.push(x,y);a.tip=x.userData.tip;}
  else if(id==='swordshield'){const x=WMODEL.sword(mats),y=WMODEL.shield(mats);m.handR.add(x);m.handL.add(y);a.weaponMeshes.push(x,y);a.tip=x.userData.tip;}
  else{const x=WMODEL[id](mats);m.handR.add(x);a.weaponMeshes.push(x);a.tip=x.userData.tip;}
  for(const w of a.weaponMeshes)w.traverse(o=>{if(o.isMesh)o.castShadow=true;});
}
const wid=a=>CLASSES[a.cls].weapons[a.widx];
function weapon(){return WEAPONS[wid(P)];}
function swapWeapon(){if(P.dead||P.charging)return;P.widx=1-P.widx;attachWeapons(P);P.atkT=0.2;const w=weapon();toast(w.name,w.desc,1.4);sfx('pick');buildActionBar();
  fx.burst(handOf(P),CLASSES[P.cls].color,14,3,0.4,0.25);}
const _hp=new V3();
function handOf(a){if(a.tip&&a.model){a.tip.getWorldPosition(_hp);return _hp.clone();}return new V3(a.pos.x,a.pos.y+1.3,a.pos.z);}
function fwdOf(a,v){return v.set(Math.sin(a.yaw),0,Math.cos(a.yaw));}
function lookOf(a,v){return v.set(Math.sin(a.yaw)*Math.cos(a.pitch),Math.sin(a.pitch),Math.cos(a.yaw)*Math.cos(a.pitch));}
function rightOf(a,v){return v.set(-Math.cos(a.yaw),0,Math.sin(a.yaw));}
function avatars(){const out=[];if(P.model&&G.state==='playing')out.push(P);for(const r of NET.peers.values())if(r.inGame&&r.model)out.push(r);return out;}
function alliesNear(a,r){if(a.local||P.dead||!NET.on||G.state!=='playing')return[];return Math.hypot(P.pos.x-a.pos.x,P.pos.z-a.pos.z)<r?[P]:[];}
function healActor(a,amt){if(a.local)healPlayer(amt);}
function animateAvatar(a,dt){
  const m=a.model;if(!m)return;
  if(a.dead){a.deathT+=dt;m.body.rotation.x=-Math.min(Math.PI/2,a.deathT*3);m.body.position.y=Math.min(0.3,a.deathT);return;}
  const hs=Math.hypot(a.vel.x,a.vel.z);a.walkPh+=dt*hs*1.7;const amt=Math.min(1,hs/6)*(a.onGround?1:0.3);a.idlePh=(a.idlePh||0)+dt;
  m.legL.rotation.x=Math.sin(a.walkPh)*0.75*amt;m.legR.rotation.x=-Math.sin(a.walkPh)*0.75*amt;
  m.body.position.y=Math.abs(Math.sin(a.walkPh))*0.06*amt+Math.sin(a.idlePh*2.2)*0.012*(1-amt);m.body.position.z=0;m.body.rotation.x=0;m.torso.rotation.x=amt*0.08;m.head.rotation.x=0;
  if(!a.onGround&&!a.leap&&!a.dash){m.legL.rotation.x=-0.5;m.legR.rotation.x=0.3;}
  if(a.swingT>=1)m.torso.rotation.y*=0.8;
  poseArms(a,dt,amt);
  if(a.anim){const A=a.anim;A.t+=dt;const k=A.t/A.d;if(k>=1)a.anim=null;else POSES[A.k](m,k,Math.min(1,k/0.12,(1-k)/0.22),a);}
  if(m.cape)m.cape.rotation.x=lerp(m.cape.rotation.x,0.08+amt*0.7+(a.onGround?0:0.5)+Math.sin(a.idlePh*(3+amt*6))*(0.04+amt*0.06),Math.min(1,dt*8));
  // evasive roll: a tucked somersault around the hips
  if(a.rollT>0){a.rollT-=dt;const k=1-Math.max(0,a.rollT)/0.3,th=k*TAU;m.body.rotation.x=th;m.body.position.y=0.85-0.85*Math.cos(th)-0.3*Math.sin(k*Math.PI);m.body.position.z=-0.85*Math.sin(th);
    m.legL.rotation.x=m.legR.rotation.x=-1.4;m.armL.rotation.x=m.armR.rotation.x=-1.2;}
  // slam: arms raised overhead on the way up, legs tucked
  if(a.leap){const k=Math.min(1,a.leap.t/a.leap.dur);m.armL.rotation.set(-2.9+k*1.2,0,0.3);m.armR.rotation.set(-2.9+k*1.2,0,-0.3);m.legL.rotation.x=m.legR.rotation.x=-0.9*Math.sin(k*Math.PI);m.torso.rotation.x=-0.3+k*0.7;}
  if(a.whirl>0){a.whirl-=dt;m.body.rotation.y+=dt*22;}else m.body.rotation.y=0;
  const near=a.local||a.pos.distanceTo(camera.position)<45;
  if(near&&a.buffs.cry&&Math.random()<0.5)PA.emit(a.pos.x+rand(-0.5,0.5),a.pos.y+rand(0.2,1.8),a.pos.z+rand(-0.5,0.5),0,2.5,0,0.5,0.35,0.05,col(0xff4a2a),0,0,0.9);
  if(near&&a.buffs.hot&&Math.random()<0.4)PA.emit(a.pos.x+rand(-0.5,0.5),a.pos.y+rand(0,1.6),a.pos.z+rand(-0.5,0.5),0,1.6,0,0.7,0.3,0.05,col(0x8dff9e),0,0,0.9);
  const w=WEAPONS[wid(a)];if(near&&w.type==='cast'&&a.tip&&Math.random()<0.6){const p=handOf(a);const id=wid(a);const c=id==='firestaff'?0xff7a22:id==='frostwand'?0x8fe6ff:0xffe48a;
    PA.emit(p.x,p.y,p.z,rand(-0.3,0.3),rand(0.3,1),rand(-0.3,0.3),0.4,0.28,0.04,col(c),0,1,0.9);}
  for(const k in a.buffs){a.buffs[k]-=dt;if(a.buffs[k]<=0)delete a.buffs[k];}
}
/* cast poses: each ability plays a short full-body gesture, blended over the walk cycle by weight w */
const rotTo=(o,x,y,z,w)=>{o.rotation.x=lerp(o.rotation.x,x,w);o.rotation.y=lerp(o.rotation.y,y,w);o.rotation.z=lerp(o.rotation.z,z,w);};
const POSES={
  push:(m,k,w)=>{const e=k<0.3?k/0.3:1;rotTo(m.armL,-1.2-0.4*e,0.25,0.25,w);rotTo(m.armR,-1.2-0.4*e,-0.25,-0.25,w);m.torso.rotation.x=lerp(m.torso.rotation.x,0.2*e-0.1,w);},
  raise:(m,k,w)=>{rotTo(m.armL,-2.95,0,0.4,w);rotTo(m.armR,-2.95,0,-0.4,w);m.head.rotation.x=lerp(m.head.rotation.x,-0.4,w);m.torso.rotation.x=lerp(m.torso.rotation.x,-0.2,w);},
  throw:(m,k,w)=>{const f=k<0.35?0:Math.min(1,(k-0.35)/0.25);rotTo(m.armR,lerp(-2.9,-0.8,f),0,-0.2,w);m.torso.rotation.y=lerp(m.torso.rotation.y,lerp(0.45,-0.35,f),w);rotTo(m.armL,-0.6,0,0.5,w);},
  shout:(m,k,w)=>{rotTo(m.armL,-0.5,0,1.35,w);rotTo(m.armR,-0.5,0,-1.35,w);m.head.rotation.x=lerp(m.head.rotation.x,-0.5,w);m.torso.rotation.x=lerp(m.torso.rotation.x,-0.3,w);},
  pray:(m,k,w)=>{rotTo(m.armL,-2.0,0.62,0,w);rotTo(m.armR,-2.0,-0.62,0,w);m.head.rotation.x=lerp(m.head.rotation.x,-0.35+0.6*Math.min(1,k*2),w);},
  point:(m,k,w)=>{const f=k<0.4?0:Math.min(1,(k-0.4)/0.15);rotTo(m.armR,lerp(-3.0,-1.3,f),0,-0.15,w);rotTo(m.armL,-0.3,0,0.3,w);m.torso.rotation.x=lerp(m.torso.rotation.x,-0.15+0.35*f,w);},
  lunge:(m,k,w)=>{m.torso.rotation.x=lerp(m.torso.rotation.x,0.55,w);rotTo(m.armR,-1.5,0,-0.1,w);rotTo(m.armL,-0.2,0,0.5,w);m.legL.rotation.x=lerp(m.legL.rotation.x,-0.9,w);m.legR.rotation.x=lerp(m.legR.rotation.x,0.7,w);},
  bowup:(m,k,w)=>{rotTo(m.armL,-2.7,0,0.2,w);rotTo(m.armR,-2.4,-0.4,0,w);m.head.rotation.x=lerp(m.head.rotation.x,-0.55,w);m.torso.rotation.x=lerp(m.torso.rotation.x,-0.25,w);},
  kneel:(m,k,w)=>{const f=k<0.35?0:1;m.body.position.y-=0.35*w*(1-f*0.5);rotTo(m.armL,f?-1.0:-0.2,0,f?1.4:0.9,w);rotTo(m.armR,f?-1.0:-0.2,0,f?-1.4:-0.9,w);m.legL.rotation.x=lerp(m.legL.rotation.x,-1.0,w*(1-f*0.5));m.legR.rotation.x=lerp(m.legR.rotation.x,0.6,w*(1-f*0.5));m.torso.rotation.x=lerp(m.torso.rotation.x,0.3-f*0.4,w);},
};
function castAnim(a,k,d){a.anim={k,t:0,d};}
function poseArms(a,dt,amt){
  const m=a.model,id=wid(a),sw=a.swingT<1?a.swingT:1;a.swingT=Math.min(1,a.swingT+dt/a.swingDur);
  const walkA=Math.sin(a.walkPh)*0.6*amt;
  m.armL.rotation.set(-walkA,0,0.08);m.armR.rotation.set(walkA,0,-0.08);a.castKick=Math.max(0,a.castKick-dt*5);
  if(id==='longbow'){m.armL.rotation.set(-Math.PI/2+0.05-a.pitch*0.6,0.12,0);const c=a.charging?a.charge:0;m.armR.rotation.set(-Math.PI/2+0.1-a.pitch*0.6,-0.5-c*0.35,0);
    const bw=a.weaponMeshes[0];if(bw&&bw.userData.draw)bw.userData.draw(c);}
  else if(WEAPONS[id].type==='cast'){if(a.castKick>0||a.firing){m.armR.rotation.set(-Math.PI/2-a.pitch*0.7+a.castKick*0.5,0,0);}else m.armR.rotation.set(-0.6+walkA*0.3,0,-0.1);
    if(a.casting>0){a.casting-=dt;m.armL.rotation.set(-Math.PI/2-0.3,0.3,0);m.armR.rotation.set(-Math.PI/2-0.3,-0.3,0);}}
  else{
    if(sw<1){const e=1-Math.pow(1-sw,2.4);
      if(id==='daggers'){const arm=a.swingSide>0?m.armR:m.armL;arm.rotation.set(-Math.PI/2+lerp(-0.6,0.5,e),a.swingSide*lerp(0.9,-0.9,e),0);}
      else if(id==='greatsword'){m.armR.rotation.set(lerp(-2.8,-0.2,e),lerp(0.4,-0.6,e),0);m.armL.rotation.set(lerp(-2.6,-0.3,e),lerp(0.8,-0.2,e),0);m.torso.rotation.y=lerp(0.5,-0.5,e);}
      else{m.armR.rotation.set(-Math.PI/2+lerp(-1.1,0.7,e)*0.6,a.swingSide*lerp(1.2,-1.0,e),0);m.torso.rotation.y=a.swingSide*lerp(0.35,-0.35,e);}
    }else{m.torso.rotation.y*=0.8;if(id==='greatsword'){m.armR.rotation.set(-0.5,0,-0.1);m.armL.rotation.set(-0.6,0.5,0.1);}else m.armR.rotation.x=-0.45+walkA*0.4;}
    if(id==='swordshield')m.armL.rotation.set(-0.9,0.5,0);
  }
  if(a.buffs.shield>0){m.armL.rotation.set(-2.6,0,0.3);m.armR.rotation.set(-2.6,0,-0.3);}
}
function updateRemotes(dt){
  for(const r of NET.peers.values()){if(!r.inGame||!r.model)continue;ACTOR=r;
    if(r.dash){const d=r.dash;d.t-=dt;r.vel.set(d.dir.x*d.speed,Math.max(r.vel.y,-2),d.dir.z*d.speed);moveEnt(r,dt);if(d.onTick)d.onTick(dt);if(d.t<=0){r.dash=null;if(d.onEnd)d.onEnd();}}
    else if(r.leap){const L=r.leap;L.t+=dt;const k=Math.min(1,L.t/L.dur);r.pos.lerpVectors(L.from,L.to,k);r.pos.y+=Math.sin(k*Math.PI)*L.height;if(k>=1){r.leap=null;L.onLand();}}
    else{const px=r.pos.x,pz=r.pos.z;if(r.pos.distanceTo(r.tpos)>10)r.pos.copy(r.tpos);else r.pos.lerp(r.tpos,Math.min(1,dt*12));r.vel.set((r.pos.x-px)/Math.max(dt,1e-3),0,(r.pos.z-pz)/Math.max(dt,1e-3));r.onGround=true;}
    let dy=((r.tyaw-r.yaw)%TAU+TAU*1.5)%TAU-Math.PI;r.yaw+=dy*Math.min(1,dt*14);
    r.model.root.position.copy(r.pos);r.model.root.rotation.y=r.yaw;animateAvatar(r,dt);ACTOR=P;
    if(r.tag){_pv.set(r.pos.x,r.pos.y+2.35,r.pos.z).project(camera);const d=r.pos.distanceTo(camera.position);if(_pv.z>1||d>80){r.tag.style.display='none';continue;}
      r.tag.style.display='block';r.tag.style.transform=`translate(${(_pv.x+1)/2*innerWidth}px,${(1-_pv.y)/2*innerHeight}px) translate(-50%,-100%)`;
      r.tag.firstChild.textContent=(r.dead?'Downed · ':'')+r.name;r.tag.style.setProperty('--c',CLASSES[r.cls].color);r.tag.querySelector('s').style.transform=`scaleX(${clamp(r.hp/r.maxHp,0,1)})`;}}
}

/* ---------------- player ---------------- */
const aim={point:new V3(),ent:null,dist:0,block:null};
const _cf=new V3(),_t1=new V3(),_t2=new V3(),_t3=new V3();
function updateAim(){
  lookOf(P,_cf);const o=camera.position;let best=60,be=null;
  for(const e of ents){if(e.dead)continue;const cx=e.pos.x-o.x,cy=e.pos.y+e.h*0.5-o.y,cz=e.pos.z-o.z;const t=cx*_cf.x+cy*_cf.y+cz*_cf.z;if(t<2.5||t>best)continue;
    const px=cx-_cf.x*t,py=cy-_cf.y*t,pz=cz-_cf.z*t;const rr=Math.max(e.w+0.2,e.h*0.45);if(px*px+py*py+pz*pz<rr*rr){best=t;be=e;}}
  const hit=voxelRay(o,_cf,best);if(hit&&hit.t<best){best=hit.t;be=null;}
  aim.dist=best;aim.ent=be;aim.point.copy(o).addScaledVector(_cf,best);aim.block=hit&&hit.t<=best+0.01?hit:null;
}
function groundAt(x,z,yHint){const fx=Math.floor(x),fz=Math.floor(z);let y=Math.min(CH-2,Math.floor(yHint)+3);for(;y>0;y--){if(world.get(fx,y,fz)&&!world.get(fx,y+1,fz))return y+1;}return world.topAt(fx,fz)+1;}
function resetRun(cls){
  P.cls=cls;const C=CLASSES[cls];P.st=baseStats();P.mods={};P.maxHp=C.hp;P.hp=P.ghostHp=C.hp;P.stam=100;P.hunger=100;P.lvl=1;P.xp=0;P.widx=0;P.cds=[0,0,0,0];P.atkT=0;P.charge=0;P.charging=false;
  P.invuln=0;P.buffs={};P.food={meat:1,berries:3};P.kills=0;P.dash=null;P.leap=null;P.dead=false;P.deathT=0;P.vel.set(0,0,0);P.swingT=1;P.hits=0;P.perks=[];P.whirl=0;P.casting=0;P.regenAcc=0;P.hitT=0;P.revive=0;
  P.rerolls=2;P.rollT=0;P.ripT=0;P.maceN=0;P.phoenixUsed=false;P.castN=0;P.wellT=3;P.stormT=0;P.anim=null;clearRelics(P);clearCorpses();resetDirector();
  let sp=world.spawnPoint();if(NET.on&&!NET.host&&NET.spawnAt){const s=surfaceSpot(NET.spawnAt.x+rand(-3,3),NET.spawnAt.z+rand(-3,3));if(s)sp=s;}
  P.pos.copy(sp);P.visY=sp.y;P.yaw=0;P.pitch=-0.15;buildAvatar(P);NET.lastMods='';
  if(!NET.on||NET.host){for(const e of ents)removeEnt(e);ents=[];}
  for(const p of projs)p.mesh&&scene.remove(p.mesh);projs=[];for(const p of pickups)scene.remove(p.mesh);pickups=[];
  for(const z of zones)z.cleanup&&z.cleanup();zones=[];for(const t of traps)scene.remove(t.mesh);traps=[];timers=[];for(const s of stuck)scene.remove(s.m);stuck=[];for(const c of chests)scene.remove(c.mesh);chests=[];
  for(const d of decoys)scene.remove(d.model.root);decoys=[];
  PA.clear();PN.clear();DB.clear();
  const keepTime=NET.on&&!NET.host;Object.assign(G,{ui:null,kills:0,runT:0,hitstop:0,shake:0,spawnT:2,over:false,trial:null,flash:0});
  if(!keepTime){Object.assign(G,{time:0.06,day:1,night:false,blood:false,bossAnnounced:-1,lastNight:false});}else{G.lastNight=G.night;}
  for(const p of world.poiCache.values())if(p){p.chestOut=false;p.guarded=false;p.trial=null;if(!keepTime){p.used=false;p.looted=false;p.cleared=false;}}
}
function hurtPlayer(amount,src,opts={}){
  if(P.dead||G.state!=='playing')return;
  if(P.invuln>0||P.buffs.shield>0){popText(_t1.set(P.pos.x,P.pos.y+2,P.pos.z),'Blocked','info');return;}
  let a=amount*(1-P.st.armor);if(wid(P)==='swordshield')a*=0.7-0.1*M(P,'w.swordshield.bulwark');
  a=Math.max(1,Math.round(a));P.hp-=a;P.hitT=0.35;G.shake=Math.min(1,G.shake+0.35+a/60);sfx('hurt');
  if(wid(P)==='swordshield'&&M(P,'w.swordshield.riposte')&&G.runT>(P.ripT||0)){P.ripT=G.runT+2;ACTOR=P;const c=P.pos.clone();fx.shock(c,0xe8f4ff,0.5,4,0.35,1.2);fx.ring(c,0xe8f4ff,0.5,4,0.35);sfx('crit');
    aoeEnemies(c,4,e=>{damageEnt(e,28,{kb:9,src:c,col:0xe8f4ff});status(e,{stun:0.6});});}
  popText(_t1.set(P.pos.x,P.pos.y+2,P.pos.z),'-'+a,'hurt');fx.burst(_t1.set(P.pos.x,P.pos.y+1.1,P.pos.z),0xff4a3a,10,4,0.4,0.25,{g:9});
  for(const m of P.model.mats)if(m.emissive)m.emissive.setHex(0x661111);
  if(src&&opts.kb!==0){_t2.set(P.pos.x-src.x,0,P.pos.z-src.z);if(_t2.lengthSq()>1e-4){_t2.normalize().multiplyScalar(opts.kb||5);P.vel.x+=_t2.x;P.vel.z+=_t2.z;P.vel.y=Math.max(P.vel.y,3);}}
  if(P.hp<=0)killPlayer();
}
function healPlayer(a,quiet){if(P.dead)return;const before=P.hp;P.hp=Math.min(P.maxHp,P.hp+a);const d=Math.round(P.hp-before);if(d>=2&&!quiet)popText(_t1.set(P.pos.x,P.pos.y+2.1,P.pos.z),'+'+d,'heal');}
function killPlayer(){if(augPhoenix())return;P.hp=0;P.dead=true;P.deathT=0;sfx('die');G.hitstop=0.25;fx.burst(_t1.set(P.pos.x,P.pos.y+1,P.pos.z),0xff4a3a,40,6,0.9,0.4,{g:6});mouse.l=false;P.charging=false;
  if(NET.on&&!G.over){toast('You are down','A teammate can revive you (hold E), or wait for dawn',4);return;}
  setTimeout(()=>{if(document.pointerLockElement)document.exitPointerLock();},900);setTimeout(showDeath,1900);}
function revivePlayer(frac,by){P.dead=false;P.deathT=0;P.hp=Math.max(1,Math.round(P.maxHp*frac));P.invuln=2;if(P.model){P.model.body.rotation.x=0;P.model.body.position.y=0;}
  fx.pillar(P.pos.clone(),0x8dff9e,1.2,10,1);sfx('heal');toast('Back on your feet',by?'Revived by '+by:'Dawn has come',2.5);}
function gainXp(n){P.xp+=n*P.st.xpMul;while(P.xp>=xpNeed()){P.xp-=xpNeed();P.lvl++;levelUpBlast();later(0.45,()=>openPerks('Level '+P.lvl,'Choose one upgrade. It lasts until you die.'));}}
/* levelling up clears breathing room: a bright wave that hurls back and stuns everything close */
function levelUpBlast(){sfx('level');const c=P.pos.clone();ACTOR=P;
  fx.ring(c,0x79d0ff,0.5,8,0.7);fx.shock(c,0x79d0ff,0.6,8,0.5,2.2);fx.circle(c,0x79d0ff,5,1.2,{style:'arcane',spin:3});fx.flare(_t1.set(c.x,c.y+1.2,c.z),0xbfe8ff,6,0.5,true);fx.burst(_t1.set(c.x,c.y+1,c.z),0x79d0ff,60,8,0.9,0.4,{up:2});fx.pillar(c.clone(),0x79d0ff,1.2,16,1.2);flashLight(_t1.set(c.x,c.y+2,c.z),0x79d0ff,4,18,0.6);
  P.hp=Math.min(P.maxHp,P.hp+P.maxHp*0.25);aoeEnemies(c,7,e=>{if(e.passive||e.T.prop)return;damageEnt(e,10,{kb:11,src:c,up:5,col:0x79d0ff,quiet:true});status(e,{stun:0.8});});}
function xpNeed(){const l=P.lvl-1;return Math.round(16+l*11+l*l*0.55);}

const _mv=new V3(),_rt=new V3();
function updatePlayer(dt){
  const m=P.model;ACTOR=P;
  if(P.dead){P.deathT+=dt;animateAvatar(P,dt);moveEnt(P,dt);m.root.position.set(P.pos.x,P.pos.y,P.pos.z);
    return;}
  const st=P.st;
  for(let i=0;i<4;i++)P.cds[i]=Math.max(0,P.cds[i]-dt);P.invuln=Math.max(0,P.invuln-dt);P.hitT=Math.max(0,P.hitT-dt);
  if(P.hitT<=0)for(const mm of m.mats)if(mm.emissive)mm.emissive.setHex(P.buffs.cry?0x401008:0);
  P.hunger=Math.max(0,P.hunger-dt*(G.night?0.26:0.21)*((keys.ShiftLeft||keys.ShiftRight)&&P.stam>0?1.6:1));
  if(P.hunger<=0)P.regenAcc-=dt*1.5;else if(P.hunger>75)P.regenAcc+=dt*0.8;
  P.regenAcc+=dt*st.regen;if(P.buffs.hot)P.regenAcc+=dt*6;
  if(P.regenAcc>=1){const a=Math.floor(P.regenAcc);healPlayer(a,true);P.regenAcc-=a;}
  if(P.regenAcc<=-1){P.regenAcc+=1;P.hp-=1;if(P.hp<=0)killPlayer();if(Math.random()<0.3)popText(_t1.set(P.pos.x,P.pos.y+2,P.pos.z),'Starving','hurt');}
  P.ghostHp=P.ghostHp>P.hp?Math.max(P.hp,P.ghostHp-dt*40):P.hp;
  const fwd=fwdOf(P,_t1),right=rightOf(P,_rt);_mv.set(0,0,0);
  if(!G.ui){if(keys.KeyW)_mv.add(fwd);if(keys.KeyS)_mv.sub(fwd);if(keys.KeyD)_mv.add(right);if(keys.KeyA)_mv.sub(right);}
  const moving=_mv.lengthSq()>0;if(moving)_mv.normalize();P.moveDir.copy(_mv);
  const sprint=(keys.ShiftLeft||keys.ShiftRight)&&moving&&P.stam>1&&!P.charging&&!G.ui;
  if(sprint)P.stam=Math.max(0,P.stam-dt*22);else P.stam=Math.min(st.stamMax,P.stam+dt*20*st.stamRegen);
  const spd=6.2*CLASSES[P.cls].speed*st.speed*(sprint?1.55:1)*(P.charging?0.55:1)*(P.inWater?0.6:1);
  if(P.dash){const d=P.dash;d.t-=dt;P.vel.x=d.dir.x*d.speed;P.vel.z=d.dir.z*d.speed;P.vel.y=Math.max(P.vel.y,-2);if(d.onTick)d.onTick(dt);
    if(Math.random()<0.8)fx.smoke(P.pos,0xd8d0c0,1,0.6,0.6,0.6);if(d.t<=0){P.dash=null;if(d.onEnd)d.onEnd();}}
  else if(P.leap){const L=P.leap;L.t+=dt;const k=Math.min(1,L.t/L.dur);P.pos.lerpVectors(L.from,L.to,k);P.pos.y+=Math.sin(k*Math.PI)*L.height;P.vel.set(0,0,0);
    PA.emit(P.pos.x,P.pos.y+1,P.pos.z,0,0,0,0.4,0.5,0.1,col(0xffb070),0,0,0.6);if(k>=1){P.leap=null;L.onLand();}}
  else{const acc=P.onGround?14:4;P.vel.x=lerp(P.vel.x,_mv.x*spd,Math.min(1,acc*dt));P.vel.z=lerp(P.vel.z,_mv.z*spd,Math.min(1,acc*dt));}
  P.coyote=P.onGround?0.12:P.coyote-dt;
  if(keys.Space&&!G.ui){if(P.inWater)P.vel.y=Math.max(P.vel.y,4.5);else if(P.coyote>0&&!P.leap){P.vel.y=9.2;P.coyote=0;P.onGround=false;}}
  if(!P.leap){const wasAir=!P.onGround,vy=P.vel.y;moveEnt(P,dt);if(wasAir&&P.onGround&&vy<-12)fx.smoke(P.pos,0xc8bca8,6,1.5,0.6,0.7);}
  P.firing=mouse.l&&!G.ui;updatePrimary(dt);
  P.visY=Math.abs(P.visY-P.pos.y)>2?P.pos.y:lerp(P.visY,P.pos.y,Math.min(1,dt*16));
  m.root.position.set(P.pos.x,P.visY,P.pos.z);
  let dyaw=((P.yaw-m.root.rotation.y)%TAU+TAU*1.5)%TAU-Math.PI;m.root.rotation.y+=dyaw*Math.min(1,dt*18);
  animateAvatar(P,dt);if(sprint)m.torso.rotation.x+=0.12;
  updateRevive(dt);
}
function updatePrimary(dt){
  const id=wid(P),w=WEAPONS[id];P.atkT-=dt*augAtk(P)*P.st.atkSpd*(id==='daggers'?1+0.2*M(P,'w.daggers.flurry'):1);
  if(P.dash||P.leap||G.ui)return;
  if(w.type==='bow'){
    if(mouse.l&&P.atkT<=0){if(!P.charging){P.charging=true;P.charge=0;}P.charge=Math.min(1,P.charge+dt*1.55*P.st.chargeMul);
      if(P.charge>=1&&Math.random()<0.5){const p=handOf(P);PA.emit(p.x,p.y,p.z,rand(-1,1),rand(-1,1),rand(-1,1),0.3,0.3,0.05,col(0xb8ff8a),0,3,1);}}
    else if(P.charging){P.charging=false;localPrimary(id,Math.max(0.15,P.charge));P.charge=0;P.atkT=w.rate;}
    return;}
  if(!mouse.l||P.atkT>0)return;P.atkT=w.rate;localPrimary(id,0);
}
function localPrimary(id,charge){P.hand=handOf(P);P.aimPt.copy(aim.point);ACTOR=P;primaryFire(P,id,charge);if(NET.on)send(castMsg('prim',{w:id,c:r2(charge)}));}
function primaryFire(a,id,charge){const w=WEAPONS[id];if(w.type==='bow')fireArrow(a,charge);else if(w.type==='melee')meleeSwing(a,id,w);else castPrimary(a,id);}
function dmgMul(a){return augDmg(a)*a.st.dmg*(a.buffs.cry?1.4:1)*(a.buffs.shrine?1.25:1);}
function meleeSwing(a,id,w){
  a.swingT=0;a.swingDur=w.rate*0.85;a.swingSide*=-1;a.hits=(a.hits||0)+1;const yaw=a.yaw,n=a.hits;sfx(w.heavy?'heavy':'swing');
  const reach=id==='greatsword'&&M(a,'w.greatsword.reach')?1.3:1,range=w.range*reach,arc=w.arc*(reach>1?1.15:1);
  const c=new V3(a.pos.x,a.pos.y+1.0,a.pos.z);fx.arc(c,yaw,range,arc,w.color||0xffffff,w.heavy?0.26:0.18,id==='greatsword'?0:a.swingSide*0.25);
  if(w.heavy)fx.arc(_t1.set(c.x,c.y+0.15,c.z),yaw,range*0.8,arc*0.9,0xffffff,0.16,0);
  const wave=(wc,dmg,big)=>{const d=fwdOf(a,new V3());const m=new THREE.Mesh(arcGeo(big?2.6:2.2,2.2),cmAdd(wc,0.7));
    spawnProj({pos:c.clone().addScaledVector(d,1),vel:d.multiplyScalar(24),grav:0,r:big?1.7:1.4,life:0.6,mesh:m,owner:'p',pierce:true,kind:'wave',noWorld:true,
      trail:p=>{m.rotation.set(0,Math.atan2(p.vel.x,p.vel.z),0);PA.emit(p.pos.x+rand(-1,1),p.pos.y,p.pos.z+rand(-1,1),0,0,0,0.25,0.35,0.05,col(wc),0,0,0.8);},
      onHit:(e,p)=>damageEnt(e,dmg,{kb:5,src:p.pos,col:wc})});};
  if(id==='greatsword'&&M(a,'w.greatsword.wave'))wave(0xfff1d6,18,false);
  if(a.buffs.cry&&M(a,'cry.warlord'))wave(0xff5b4c,22,true);
  if(id==='daggers'&&n%3===0&&M(a,'w.daggers.shadow')){const d=lookOf(a,new V3());const m=new THREE.Group();mk(m,0.08,0.05,0.7,cmAdd(0xa98aff,0.9),0,0,0).castShadow=false;
    spawnProj({pos:handOf(a),vel:d.multiplyScalar(40),grav:0,r:0.35,life:0.8,mesh:m,owner:'p',pierce:true,kind:'arrow',
      trail:p=>PA.emit(p.pos.x,p.pos.y,p.pos.z,0,0,0,0.25,0.3,0.05,col(0xa98aff),0,0,0.9),onHit:(e,p)=>{damageEnt(e,16,{kb:2,src:p.pos,col:0xa98aff});fx.sparks(p.pos,0xa98aff,6,6);}});}
  later(w.rate*0.22,()=>{if(a.dead)return;let hitN=0;const f=fwdOf(a,new V3());
    for(const e of ents){if(e.dead)continue;const dx=e.pos.x-a.pos.x,dz=e.pos.z-a.pos.z,dy=e.pos.y-a.pos.y;const d=Math.hypot(dx,dz)-e.w;if(d>range||Math.abs(dy)>2.6)continue;
      const ang=Math.acos(clamp((dx*f.x+dz*f.z)/Math.max(0.01,Math.hypot(dx,dz)),-1,1));if(ang>arc/2&&d>0.6)continue;
      let dm=w.dmg;if(id==='daggers'&&n%3===0)dm*=2;if(id==='mace')dm*=1+0.25*M(a,'w.mace.heavy');
      if(id==='greatsword'&&M(a,'w.greatsword.exec')&&e.hp/e.maxHp<0.35)dm*=1.6;
      damageEnt(e,dm,{kb:w.heavy?9:4,src:a.pos,col:w.color});hitN++;fx.sparks(_t3.set(e.pos.x,e.pos.y+e.h*0.6,e.pos.z),w.color||0xffffff,w.heavy?8:4,w.heavy?10:7);
      if(id==='daggers'&&M(a,'w.daggers.venom'))status(e,{poison:3});
      if(id==='swordshield'&&M(a,'w.swordshield.bash')&&n%3===0)status(e,{stun:1});
      if(id==='mace'&&M(a,'w.mace.thunder')&&n%3===0&&hitN===1)lightningStrike(a,e.pos.clone(),26,1.6);}
    if(hitN&&id==='mace'){healActor(a,2);if(M(a,'w.mace.nova')){a.maceN=(a.maceN||0)+1;if(a.maceN%4===0)holyNova(a);}}
    if(hitN&&a.local){G.hitstop=Math.max(G.hitstop,w.heavy?0.07:0.035);G.shake=Math.min(1,G.shake+(w.heavy?0.25:0.1));}});
}
function holyNova(a){const c=new V3(a.pos.x,a.pos.y,a.pos.z);sfx('holy');fx.ring(c,0xffe48a,0.5,4.5,0.4);fx.burst(_t1.set(c.x,c.y+1,c.z),0xffe48a,30,6,0.5,0.35);
  aoeEnemies(c,4.5,e=>damageEnt(e,20,{kb:3,src:c,col:0xffe48a}));healActor(a,8);for(const p of alliesNear(a,8))healPlayer(8);}
function spawnProj(o){if(projs.length>=CAP.projs){const q=projs.shift();if(q.mesh)scene.remove(q.mesh);}
  o.a=o.a||ACTOR;o.hit=new Set();o.age=0;if(o.owner==='p'&&o.bnc==null)o.bnc=AK(o.a,'bounce');if(o.mesh){scene.add(o.mesh);o.mesh.position.copy(o.pos);}projs.push(o);return o;}
function aimDirFrom(from,target,out){return out.copy(target).sub(from).normalize();}
const MC={};function cm(c,basic){const k=(basic?'b':'l')+c;if(!MC[k]){MC[k]=basic?new THREE.MeshBasicMaterial({color:c}):new THREE.MeshLambertMaterial({color:c});MC[k].userData.shared=true;}return MC[k];}
function cmAdd(c,op){const k='a'+c+'_'+op;if(!MC[k]){MC[k]=basicAdd(c,op);MC[k].userData.shared=true;}return MC[k];}

function arrowMesh(color=0x6a4528,tip=0xd8dde4){const g=new THREE.Group();mk(g,0.04,0.04,0.75,cm(color),0,0,0);mk(g,0.08,0.08,0.12,cm(tip),0,0,0.4);mk(g,0.14,0.02,0.12,cm(0xf0f0f0),0,0,-0.32);
  g.children.forEach(c=>c.castShadow=false);return g;}
function glowBall(c,r){const g=new THREE.Group();const core=new THREE.Mesh(SPH,cm(0xffffff,true));core.scale.setScalar(r*0.55);const sh=new THREE.Mesh(SPH,cmAdd(c,0.55));sh.scale.setScalar(r);g.add(core,sh);return g;}
function playerArrow(a,from,vel,o){
  return spawnProj({a,pos:from,vel,grav:o.grav??9,r:o.r||0.25,dmg:o.dmg,pierce:!!o.pierce,life:o.life||3,mesh:arrowMesh(o.shaft||0x6a4528,o.tip||0xd8dde4),owner:'p',kind:'arrow',split:o.split||0,fire:o.fire||0,blast:o.blast||0,
    trail:p=>{if(o.glow)PA.emit(p.pos.x,p.pos.y,p.pos.z,0,0,0,0.3,0.32,0.05,col(o.glow),0,0,0.85);else if(Math.random()<0.5)PA.emit(p.pos.x,p.pos.y,p.pos.z,0,0,0,0.25,0.18,0.03,col(0xffffff),0,0,0.5);},
    onHit:(e,p)=>{damageEnt(e,p.dmg,{kb:3,src:p.pos});fx.sparks(p.pos,o.glow||0xffe0b0,4,6,0.22);if(p.fire){status(e,{burn:3});fx.burst(p.pos,0xff7a22,10,3,0.3,0.3);}if(p.split)splitArrows(p,e);if(p.blast&&!p.pierce)arrowBlast(p.pos);},
    onWorld:p=>{if(p.blast)arrowBlast(p.pos);stickArrow(p);},onExpire:p=>{if(p.blast)arrowBlast(p.pos);}});
}
function arrowBlast(pos){const c=pos.clone();sfx('fire');fx.burst(c,0xb8ff8a,30,7,0.45,0.45);fx.ring(_t3.copy(c).setY(c.y-0.4),0xb8ff8a,0.4,3,0.35);fx.sparks(c,0xeaffd0,10,10);flashLight(c,0xb8ff8a,2.5,10,0.3);
  aoeEnemies(c,3,e=>damageEnt(e,26,{kb:6,src:c,col:0xb8ff8a}));}
function splitArrows(p,e){const n=p.split;p.split=0;const dir=p.vel.clone().setY(0).normalize();fx.burst(p.pos,0xb8ff8a,12,4,0.3,0.25);
  for(let i=0;i<n;i++){const d=dir.clone().applyAxisAngle(UP,(i-(n-1)/2)*0.55+rand(-0.1,0.1));d.y=0.08;d.normalize();
    const c=playerArrow(p.a,p.pos.clone().addScaledVector(d,0.5),d.multiplyScalar(38),{dmg:Math.max(4,Math.round(p.dmg*0.5)),grav:6,life:0.9,shaft:0x4a7a3a,tip:0xb8ff8a,glow:0xb8ff8a});c.hit.add(e);}}
function fireArrow(a,charge){
  const from=a.hand.clone();const dir=aimDirFrom(from,a.aimPt,new V3());const full=charge>=0.99;
  const sp=lerp(32,72,charge);dir.y+=0.02;dir.normalize();sfx('bow');a.castKick=1;const dmg=Math.round(lerp(10,40,charge));
  playerArrow(a,from,dir.clone().multiplyScalar(sp),{dmg,pierce:full,grav:full?3:9,glow:full?0xb8ff8a:0,split:full&&M(a,'w.longbow.split')?3:0,blast:full&&M(a,'w.longbow.blast')});
  if(full){fx.burst(from,0xb8ff8a,12,4,0.3,0.3);if(a.local)G.shake=Math.min(1,G.shake+0.08);}
  const extra=2*M(a,'w.longbow.multi');for(let k=1;k<=extra;k++){const d=dir.clone().applyAxisAngle(UP,(k%2?1:-1)*Math.ceil(k/2)*0.07);playerArrow(a,from.clone(),d.multiplyScalar(sp),{dmg:Math.round(dmg*0.5),grav:full?3:9});}
}
function stickArrow(p){if(p.mesh){if(stuck.length>=CAP.stuck){scene.remove(stuck.shift().m);}stuck.push({m:p.mesh,t:4});p.keepMesh=true;}fx.burst(p.pos,0xc8b89a,5,2,0.3,0.15,{g:8});}
function castPrimary(a,id){
  const from=a.hand.clone();const dir=aimDirFrom(from,a.aimPt,new V3());a.castKick=1;
  if(id==='firestaff'){sfx('fire');const rad=3*a.st.fireRad*a.st.area*(1+0.3*M(a,'w.firestaff.big'));const cl=M(a,'w.firestaff.cluster');const tr=M(a,'w.firestaff.trail');
    const boom=p=>{explodeFire(p.pos,rad,20,cl);if(tr){const g=p.pos.clone();g.y=groundAt(g.x,g.z,g.y+1);firePool(g,rad*0.75,3,5,false);}};
    spawnProj({pos:from,vel:dir.multiplyScalar(30),grav:0,r:0.35,life:2.2,mesh:glowBall(0xff6a1a,0.32),owner:'p',kind:'fire',
      trail:p=>{for(let i=0;i<3;i++)PA.emit(p.pos.x+rand(-0.1,0.1),p.pos.y+rand(-0.1,0.1),p.pos.z+rand(-0.1,0.1),rand(-0.6,0.6),rand(0,1.2),rand(-0.6,0.6),0.45,0.6,0.08,col(pick([0xff6a1a,0xffb03a,0xff3a10])),-1,1,1);
        if(Math.random()<0.3)PN.emit(p.pos.x,p.pos.y,p.pos.z,0,0.8,0,0.8,0.3,0.9,col(0x3a3330),-0.5,1,0.4);},
      onHit:(e,p)=>boom(p),onWorld:boom,onExpire:boom});
    fx.burst(from,0xff8a2a,6,2,0.25,0.3);}
  if(id==='frostwand'){sfx('ice');const n=1+2*M(a,'w.frostwand.fork');const thr=M(a,'w.frostwand.brittle')?3:5;
    for(let k=0;k<n;k++){const d=dir.clone();if(k)d.applyAxisAngle(UP,(k%2?1:-1)*Math.ceil(k/2)*0.08);d.x+=rand(-0.02,0.02);d.y+=rand(-0.02,0.02);d.normalize();
      const sh=new THREE.Mesh(OCT,cm(0xbff4ff,true));sh.scale.set(0.7,0.7,2.2);
      spawnProj({pos:from.clone(),vel:d.multiplyScalar(46),grav:0,r:0.25,life:1.4,mesh:sh,owner:'p',kind:'ice',pierce:M(a,'w.frostwand.lance')>0,
        trail:p=>PA.emit(p.pos.x,p.pos.y,p.pos.z,rand(-0.3,0.3),rand(-0.3,0.3),rand(-0.3,0.3),0.3,0.3,0.04,col(0x8fe6ff),0,1,0.8),
        onHit:(e,p)=>{damageEnt(e,6,{kb:1,src:p.pos,col:0x8fe6ff});status(e,{slow:2,chill:1,thr});fx.burst(p.pos,0xbff4ff,6,3,0.3,0.2);},
        onWorld:p=>fx.burst(p.pos,0xbff4ff,6,3,0.3,0.2)});}}
  if(id==='scepter'){sfx('holy');const n=1+M(a,'w.scepter.twin');const split=M(a,'w.scepter.split');const nova=M(a,'w.scepter.nova');
    for(let k=0;k<n;k++){const d=dir.clone();if(k)d.applyAxisAngle(UP,(k%2?1:-1)*Math.ceil(k/2)*0.12);const b=holyBolt(a,from.clone(),d.multiplyScalar(30),14,0.24,split);b.nova=nova;}}
}
function holyBolt(a,from,vel,dmg,size,split){
  return spawnProj({a,pos:from,vel,grav:0,r:0.35,life:2,mesh:glowBall(0xffd35a,size),owner:'p',kind:'holy',homing:3.2,
    trail:p=>PA.emit(p.pos.x,p.pos.y,p.pos.z,rand(-0.4,0.4),rand(-0.4,0.4),rand(-0.4,0.4),0.5,size*1.8,0.05,col(0xffe48a),0,1.5,0.9),
    onHit:(e,p)=>{damageEnt(e,dmg,{kb:2,src:p.pos,col:0xffe48a});fx.burst(p.pos,0xffe48a,14,4,0.4,0.3);
      if(p.nova){const c=p.pos.clone();fx.ring(_t3.copy(c).setY(c.y-0.5),0xffe48a,0.4,2.6,0.3);fx.sparks(c,0xfff3b8,8,8);aoeEnemies(c,2.6,q=>{if(q!==e)damageEnt(q,10,{kb:3,src:c,col:0xffe48a,quiet:true});});}
      if(split)for(let i=0;i<3;i++){const an=rand(0,TAU);const c=holyBolt(p.a,p.pos.clone(),new V3(Math.cos(an),0.3,Math.sin(an)).multiplyScalar(18),7,0.14,0);c.hit.add(e);c.life=1.2;}},
    onWorld:p=>fx.burst(p.pos,0xffe48a,10,3,0.35,0.3)});
}
function explodeFire(pos,rad,dmg,cluster,small){sfx('fire');fx.flare(pos,0xffa050,small?2.5:5,0.3);if(!small){const g=pos.clone();g.y=groundAt(g.x,g.z,g.y+1);if(Math.abs(g.y-pos.y)<3)fx.decal(g,'scorch',rad*0.7,6);}fx.burst(pos,0xff7a22,small?16:40,rad*2.6,0.55,small?0.5:0.7,{s1:0.05,drag:3});fx.burst(pos,0xffd27a,small?6:16,rad*1.5,0.35,0.5);fx.smoke(pos,0x3a3330,small?3:8,1.4,1.4,1.2);
  fx.ring(_t3.copy(pos).setY(pos.y-0.4),0xff8a3a,0.4,rad,0.35);if(!small)flashLight(pos,0xff7a2a,4,rad*4,0.35);if(ACTOR.local)G.shake=Math.min(1,G.shake+(small?0.04:0.12));
  const c=pos.clone();for(const e of ents){if(e.dead)continue;if(dist3(e,c)<rad+e.w){damageEnt(e,dmg,{kb:small?3:6,src:c,col:0xff8a3a});status(e,{burn:3*ACTOR.st.fireRad});}}
  if(cluster){const n=1+2*cluster;for(let i=0;i<n;i++){const an=i/n*TAU+rand(-0.3,0.3);
    spawnProj({pos:c.clone().setY(c.y+0.4),vel:new V3(Math.cos(an)*rand(5,8),rand(5,8),Math.sin(an)*rand(5,8)),grav:18,r:0.3,life:1.6,mesh:glowBall(0xff8a2a,0.16),owner:'p',kind:'fire',
      trail:p=>PA.emit(p.pos.x,p.pos.y,p.pos.z,0,0.5,0,0.3,0.3,0.05,col(0xffb03a),0,0,1),
      onHit:(e,p)=>explodeFire(p.pos,rad*0.55,9,0,true),onWorld:p=>explodeFire(p.pos,rad*0.55,9,0,true),onExpire:p=>explodeFire(p.pos,rad*0.55,9,0,true)});}}}
function dist3(e,p){return Math.hypot(e.pos.x-p.x,(e.pos.y+e.h*0.5)-p.y,e.pos.z-p.z);}

/* ---------------- abilities ---------------- */
function cdOf(a,id){return ABIL[id].cd*a.st.cd*(1-0.22*M(a,id+'.cd'));}
function useAbility(i){if(!canAct()||P.cds[i]>0||P.leap)return;const id=CLASSES[P.cls].abilities[i];
  P.hand=handOf(P);P.aimPt.copy(aim.point);ACTOR=P;const ok=ABILITY_FN[id](P);if(ok===false)return;
  P.cds[i]=cdOf(P,id);if(NET.on)send(castMsg('cast',{ab:id}));augCast(id);
  const s=document.querySelector(`.slot[data-i="${i}"]`);if(s){s.classList.remove('flash');void s.offsetWidth;s.classList.add('flash');}}
function aoeEnemies(center,r,fn){let n=0;for(const e of ents){if(e.dead)continue;const d=Math.hypot(e.pos.x-center.x,e.pos.z-center.z);if(d<r+e.w&&Math.abs(e.pos.y-center.y)<r+2){fn(e,d);n++;}}return n;}
function foesNear(pos,r){const out=[];for(const e of ents){if(e.dead||e.passive||e.T.prop)continue;const d=Math.hypot(e.pos.x-pos.x,e.pos.z-pos.z);if(d<r)out.push([d,e]);}out.sort((p,q)=>p[0]-q[0]);return out.map(q=>q[1]);}
function zone(o){if(zones.length>=CAP.zones){const z=zones.shift();z.cleanup&&z.cleanup();}o.t=0;o.tick=0;o.a=ACTOR;zones.push(o);return o;}
function spawnDecoy(a,pos){if(decoys.length>=CAP.decoys){const d=decoys.shift();scene.remove(d.model.root);}
  const m=ghostModel(a,0x1a3a1a);m.root.position.copy(pos);m.root.rotation.y=a.yaw;scene.add(m.root);
  decoys.push({isDecoy:true,pos:pos.clone(),w:0.3,h:1.8,hp:80,t:4,model:m,dead:false});fx.smoke(pos,0xd8d0c0,8,1.5,0.8,0.9);}
function ghostModel(a,glow){const m=playerModel(a.cls);m.mats.forEach(x=>{x.transparent=true;x.opacity=0.45;x.depthWrite=false;x.emissive&&x.emissive.setHex(glow);});m.root.traverse(o=>{if(o.isMesh)o.castShadow=false;});return m;}
function makeTrap(a,to,mini){if(traps.length>=CAP.traps){const t=traps.shift();scene.remove(t.mesh);}
  const g=new THREE.Group();mk(g,0.9,0.08,0.9,cm(0x4a3a2a),0,0.04,0);
  for(let i=0;i<8;i++){const an=i/8*TAU;const t=mk(g,0.08,0.3,0.08,cm(0x9aa0a8),Math.cos(an)*0.4,0.18,Math.sin(an)*0.4);t.rotation.set(Math.sin(an)*0.5,0,-Math.cos(an)*0.5);}
  if(mini)g.scale.setScalar(0.55);
  g.position.copy(to);scene.add(g);traps.push({a,mesh:g,pos:to,life:30,armed:mini?0.35:0.4,spiked:M(a,'snare.spiked'),boom:M(a,'snare.boom'),field:!mini&&M(a,'snare.field'),mini:!!mini});
  fx.burst(_t1.copy(to).setY(to.y+0.3),0xd9b46a,mini?8:16,3,0.4,0.25);fx.ring(to,0xd9b46a,0.3,mini?1:1.6,0.4);}
function callMeteor(a,c,r,dmg,big,inferno){
  fx.telegraph(c,0xff6a1a,r,1.1);const m=new THREE.Group();const rock=new THREE.Mesh(new THREE.DodecahedronGeometry(big?1.1:0.6),cm(0xffa040,true));m.add(rock,glowBall(0xff5a10,big?1.8:1));scene.add(m);
  const from=new V3(c.x-14,c.y+42,c.z-8);const T=1.1;let t=0;
  zone({pos:c,r,life:T,onFrame:(z,dt)=>{t+=dt;const k=Math.min(1,t/T);m.position.lerpVectors(from,c,k*k);rock.rotation.x+=dt*5;
    for(let i=0;i<(big?6:3);i++)PA.emit(m.position.x+rand(-0.8,0.8),m.position.y+rand(-0.8,0.8),m.position.z+rand(-0.8,0.8),rand(-1,1),rand(1,3),rand(-1,1),0.7,big?1.1:0.7,0.1,col(pick([0xff6a1a,0xffb03a,0xff3a10])),-1,1,1);
    PN.emit(m.position.x,m.position.y,m.position.z,0,1,0,1.4,0.8,2.2,col(0x2a2420),-0.5,0.5,0.5);},
    end:()=>{scene.remove(m);rock.geometry.dispose();meteorImpact(c,r,dmg,big,inferno);},cleanup:()=>{scene.remove(m);rock.geometry.dispose();}});
}
function smiteAt(a,c,r,dmg){fx.telegraph(c,0xffe48a,r,0.35);
  later(0.35,()=>{fx.flare(_t1.copy(c).setY(c.y+1.5),0xfff3b8,7,0.4,true);fx.decal(c,'crack',r*0.9,4,0xffe48a);fx.pillar(c.clone(),0xffe48a,r*0.8,26,0.7);fx.pillar(c.clone(),0xffffff,r*0.35,26,0.45);fx.ring(c,0xffe48a,0.4,r*1.6,0.45);fx.shock(c,0xffe48a,0.4,r*1.4,0.35,1.4);flashLight(_t1.copy(c).setY(c.y+2),0xffe48a,5,18,0.5);
    fx.burst(_t1.copy(c).setY(c.y+0.5),0xffe48a,40,7,0.6,0.4,{up:3});fx.sparks(_t1,0xffffff,12,11);sfx('boom');if(a.local)G.shake=Math.min(1,G.shake+0.3);
    aoeEnemies(c,r,e=>{damageEnt(e,dmg,{kb:3,src:c,col:0xffe48a,up:5});status(e,{stun:0.9});});});}
function shieldOn(o,dur){fx.bubble(o,0xffe48a,1.6,dur);}
/* lightning from the sky onto a spot: used by Thunder Mace and the Thunder Call relic */
function lightningStrike(a,pos,dmg,r=1.8,c=0xa8dcff){const g=pos.clone();g.y=groundAt(g.x,g.z,g.y+1.5);const top=new V3(g.x+rand(-3,3),g.y+24,g.z+rand(-3,3));
  fx.lightning(top,_t3.copy(g).setY(g.y+0.1),c,0.28,0.32);fx.flare(_t3.copy(g).setY(g.y+0.6),c,4,0.25,true);fx.decal(g,'scorch',1.4,4);fx.ring(g,c,0.3,r*1.7,0.3);fx.sparks(_t3.copy(g).setY(g.y+0.3),0xffffff,10,9);fx.burst(_t3,c,14,4,0.35,0.35);
  flashLight(_t3.copy(g).setY(g.y+2),c,5,16,0.25);sfx('zap');if(a.local)G.shake=Math.min(1,G.shake+0.1);
  aoeEnemies(g,r,e=>{damageEnt(e,dmg,{kb:2,src:g,col:c,up:3});status(e,{stun:0.35});});return g;}
/* burning ground: Phoenix Fire, the Firebomb relic and Trail of Ruin */
function firePool(pos,r,life,dmg,slow){fx.disk(pos,0xff5a1a,r,life,0.26);fx.ring(pos,0xff8a3a,0.4,r,0.3);
  return zone({pos:pos.clone(),r,life,every:0.4,onTick:z=>aoeEnemies(z.pos,z.r,e=>{damageEnt(e,dmg,{quiet:true,col:0xff7a22});status(e,slow?{burn:2,slow:1}:{burn:2});}),
    onFrame:z=>{for(let i=0;i<2;i++){const an=rand(0,TAU),rr=Math.sqrt(Math.random())*z.r;PA.emit(z.pos.x+Math.cos(an)*rr,z.pos.y+0.15,z.pos.z+Math.sin(an)*rr,rand(-0.3,0.3),rand(1.2,3),rand(-0.3,0.3),0.55,0.5,0.06,col(pick([0xff6a1a,0xffb03a,0xff3a10])),-1,0.5,1);}}});}
/* arcane bolt that seeks the nearest foe: Mirror Image and the Spirit Wisp relic */
function seekBolt(a,from,dir,dmg,c,o={}){
  return spawnProj({a,pos:from,vel:dir.clone().normalize().multiplyScalar(o.speed||26),grav:0,r:0.35,life:1.8,mesh:glowBall(c,o.size||0.2),owner:'p',kind:'holy',homing:o.homing||4,pierce:!!o.pierce,
    trail:p=>PA.emit(p.pos.x,p.pos.y,p.pos.z,rand(-0.3,0.3),rand(-0.3,0.3),rand(-0.3,0.3),0.4,(o.size||0.2)*2,0.04,col(c),0,1.5,0.9),
    onHit:(e,p)=>{damageEnt(e,dmg,{kb:2,src:p.pos,col:c});fx.burst(p.pos,c,10,4,0.3,0.28);},onWorld:p=>fx.burst(p.pos,c,8,3,0.3,0.25)});}
function mirrorImage(a,pos){const m=ghostModel(a,0x3a1a5a);m.root.position.copy(pos);m.root.rotation.y=a.yaw;scene.add(m.root);fx.ring(pos,0xd08aff,0.4,2.2,0.4);
  zone({pos:pos.clone(),r:0,life:4,every:0.4,onTick:z=>{const f=foesNear(z.pos,18)[0];if(!f)return;const from=_t1.set(z.pos.x,z.pos.y+1.4,z.pos.z).clone();sfx('arcane');
      seekBolt(z.a,from,_t2.set(f.pos.x-from.x,f.pos.y+f.h*0.5-from.y,f.pos.z-from.z),16,0xd08aff);m.armR.rotation.x=-Math.PI/2;},
    onFrame:(z,dt)=>{m.root.rotation.y+=dt*0.8;m.armR.rotation.x=lerp(m.armR.rotation.x,-0.4,dt*5);if(Math.random()<0.4)PA.emit(z.pos.x+rand(-0.4,0.4),z.pos.y+rand(0,1.8),z.pos.z+rand(-0.4,0.4),0,1,0,0.5,0.3,0.04,col(0xd08aff),0,0,0.8);},
    end:()=>{scene.remove(m.root);m.mats.forEach(x=>x.dispose());fx.burst(_t1.set(pos.x,pos.y+1,pos.z),0xd08aff,20,4,0.4,0.3);},cleanup:()=>scene.remove(m.root)});}
function novaPulse(a,p0,r,fr,dmg){sfx('ice');fx.sphere(_t1.set(p0.x,p0.y+0.8,p0.z).clone(),0xbff4ff,0.5,r,0.4);
  fx.ring(p0,0x8fe6ff,0.5,r,0.5);fx.ring(p0,0xffffff,0.5,r*0.8,0.35);fx.shock(p0,0xbff4ff,0.6,r,0.45,1.8);fx.disk(p0,0x8fe6ff,r,0.6,0.25);flashLight(_t1.set(p0.x,p0.y+1,p0.z),0x8fe6ff,3,r*3,0.5);
  for(let i=0;i<60;i++){const an=rand(0,TAU),s=rand(4,r*2.4);PA.emit(p0.x,p0.y+0.5,p0.z,Math.cos(an)*s,rand(0,1.5),Math.sin(an)*s,0.5,0.45,0.05,col(pick([0x8fe6ff,0xffffff])),0,3,1);}
  for(let i=0;i<14;i++){const an=i/14*TAU;DB.emit(p0.x+Math.cos(an)*2,p0.y+0.3,p0.z+Math.sin(an)*2,pick([0xbff4ff,0x8fe6ff,0xe8fbff]),0.25,Math.cos(an)*7,5,Math.sin(an)*7,1.2);}
  aoeEnemies(p0,r,e=>{damageEnt(e,dmg,{kb:3,src:p0,col:0x8fe6ff});status(e,{freeze:fr});});if(a.local)G.shake=Math.min(1,G.shake+0.2);}
function spinBlade(a,from,dir,dmg,c=0xffe0b0){const m=new THREE.Group();mk(m,0.7,0.05,0.16,cmAdd(c,0.9),0,0,0);mk(m,0.16,0.05,0.7,cmAdd(c,0.9),0,0,0);m.children.forEach(q=>q.castShadow=false);
  return spawnProj({a,pos:from,vel:dir.multiplyScalar(19),grav:0,r:0.6,life:0.75,mesh:m,owner:'p',pierce:true,kind:'blade',noWorld:true,
    trail:p=>{m.rotation.y+=0.6;if(Math.random()<0.6)PA.emit(p.pos.x,p.pos.y,p.pos.z,0,0,0,0.2,0.35,0.05,col(c),0,0,0.8);},onHit:(e,p)=>{damageEnt(e,dmg,{kb:3,src:p.pos,col:c});fx.sparks(p.pos,c,4,6);}});}
const ABILITY_FN={
  volley(a){const n=9+a.st.extraProj+3*M(a,'volley.more');const fire=M(a,'volley.fire'),split=M(a,'volley.split')?3:0;
    const shoot=(first)=>{if(a.dead)return;const from=first?a.hand.clone():handOf(a);const base=aimDirFrom(from,a.aimPt,new V3());const step=Math.min(0.07,1.25/n);sfx('bow');a.castKick=1;
      for(let k=0;k<n;k++){const d=base.clone().applyAxisAngle(UP,(k-(n-1)/2)*step);d.y+=0.03;
        playerArrow(a,from.clone(),d.normalize().multiplyScalar(58),{dmg:15,grav:4,life:2.2,pierce:true,shaft:0x4a7a3a,tip:0xb8ff8a,glow:fire?0xff8a3a:0x9be07a,split,fire});}
      fx.burst(from,fire?0xff8a3a:0x9be07a,22,5,0.4,0.3);fx.arc(_t1.set(a.pos.x,a.pos.y+1.3,a.pos.z),a.yaw,3.2,n*step+0.4,fire?0xffb070:0xc8ffa0,0.22);};
    shoot(true);if(M(a,'volley.hail'))for(let i=1;i<=2;i++)later(0.22*i,()=>shoot(false));},
  roll(a){const d=a.moveDir.clone();if(d.lengthSq()<0.01)fwdOf(a,d).negate();d.y=0;d.normalize();const dance=M(a,'roll.dance');const hitSet=new Set();
    a.dash={dir:d,speed:25,t:0.3,onTick:dance?()=>{for(const e of ents){if(e.dead||e.passive||hitSet.has(e))continue;if(Math.hypot(e.pos.x-a.pos.x,e.pos.z-a.pos.z)<1.6+e.w&&Math.abs(e.pos.y-a.pos.y)<2.2){hitSet.add(e);
        damageEnt(e,35,{kb:5,src:a.pos,col:0xd8f0ff});fx.arc(_t3.set(a.pos.x,a.pos.y+1,a.pos.z),Math.atan2(e.pos.x-a.pos.x,e.pos.z-a.pos.z),2.2,2.4,0xd8f0ff,0.18,rand(-0.5,0.5));fx.sparks(_t3.set(e.pos.x,e.pos.y+1,e.pos.z),0xd8f0ff,6,8);
        if(a.local){const i=CLASSES[a.cls].abilities.indexOf('roll');P.cds[i]=Math.max(0,P.cds[i]-0.4);}}}}:null};
    a.rollT=0.3;if(a.local)a.invuln=0.45;
    sfx('dash');fx.smoke(a.pos,0xd8d0c0,10,2,0.7,0.9);fx.burst(_t1.set(a.pos.x,a.pos.y+0.8,a.pos.z),0xc9e6b0,16,3,0.4,0.25);
    if(M(a,'roll.decoy'))spawnDecoy(a,a.pos);
    if(M(a,'roll.shot')){const c=new V3(a.pos.x,a.pos.y+1.2,a.pos.z);for(let i=0;i<10;i++){const an=i/10*TAU;playerArrow(a,c.clone(),new V3(Math.cos(an),0.05,Math.sin(an)).multiplyScalar(44),{dmg:12,grav:3,life:1.2,shaft:0x4a7a3a,tip:0xb8ff8a,glow:0x9be07a});}}},
  snare(a){const cnt=1+M(a,'snare.twin');const fl=fwdOf(a,new V3());const base=a.aimPt.clone();if(a.pos.distanceTo(base)>22)base.copy(a.pos).addScaledVector(fl,22);const rt=rightOf(a,new V3());
    for(let i=0;i<cnt;i++){const off=i===0?0:(i%2?1:-1)*Math.ceil(i/2)*2.4;const to=base.clone().addScaledVector(rt,off);to.y=groundAt(to.x,to.z,to.y+1);makeTrap(a,to);}sfx('pick');},
  storm(a){const eye=M(a,'storm.eye');const c=eye?a.pos.clone():a.aimPt.clone();c.y=groundAt(c.x,c.z,c.y+1);const r=6*a.st.area*(1+0.25*M(a,'storm.wide'));const life=4+1.5*M(a,'storm.long')+(eye?2:0);const frost=M(a,'storm.frost');
    const tint=frost?0x9fe6ff:0x7fd46a;const rg=fx.ring(c,tint,r,r,life,0.15,0.6),dk=fx.disk(c,tint,r,life,0.15);sfx('bow');
    zone({follow:eye?a:null,pos:c,r,life,every:0.2,onTick:z=>{sfx('bow');aoeEnemies(z.pos,z.r,e=>{damageEnt(e,9,{kb:0.5,src:z.pos,quiet:true});if(frost)status(e,{slow:1.5});});},
      onFrame:z=>{if(eye){rg.mesh.position.set(z.pos.x,z.pos.y+0.15,z.pos.z);dk.mesh.position.set(z.pos.x,z.pos.y+0.1,z.pos.z);}
        for(let k=0;k<5;k++){const an=rand(0,TAU),rr=Math.sqrt(Math.random())*z.r;const x=z.pos.x+Math.cos(an)*rr,zz=z.pos.z+Math.sin(an)*rr;
          PA.emit(x,z.pos.y+14,zz,0,-52,0,0.27,0.32,0.32,col(frost?0xd8f8ff:0xd8ffb8),0,0,1);if(Math.random()<0.35)PA.emit(x,z.pos.y+0.2,zz,rand(-2,2),rand(1,3),rand(-2,2),0.3,0.25,0.02,col(frost?0x9fe6ff:0xb8ff8a),8,0,1);}}});},
  blink(a){const f=lookOf(a,new V3()).setY(0).normalize();const from=a.pos.clone();let ok=null;const far=11+5*M(a,'blink.far');
    for(let d=far;d>=1;d-=0.5){const x=a.pos.x+f.x*d,z=a.pos.z+f.z*d;const gy=groundAt(x,z,a.pos.y+2);if(Math.abs(gy-a.pos.y)>6)continue;if(!collides(x,gy,z,0.3,1.8)){ok=new V3(x,gy,z);break;}}
    if(!ok)return false;sfx('arcane');const bp=p=>{fx.burst(_t2.set(p.x,p.y+1,p.z),0xc4a8ff,30,5,0.5,0.35);fx.ring(p,0xc4a8ff,0.3,2.5,0.4);fx.pillar(p.clone(),0xc4a8ff,0.7,4,0.35);};bp(from);
    if(M(a,'blink.mirror'))mirrorImage(a,from);
    a.pos.copy(ok);if(a.local){a.visY=ok.y;a.vel.set(0,0,0);a.invuln=0.25;}else a.tpos.copy(ok);bp(ok);
    if(M(a,'blink.wake'))for(const p of[from,ok]){fx.ring(p,0xd08aff,0.5,3.5,0.4);fx.shock(p,0xd08aff,0.4,3.5,0.35,1.4);fx.burst(_t2.set(p.x,p.y+1,p.z),0xd08aff,30,6,0.5,0.4);aoeEnemies(p,3.5,e=>damageEnt(e,30,{kb:6,src:p,col:0xd08aff}));}
    for(let i=0;i<20;i++){const t=i/20;PA.emit(lerp(from.x,ok.x,t),lerp(from.y,ok.y,t)+1,lerp(from.z,ok.z,t),0,0,0,0.5,0.4,0.05,col(0xa183ff),0,0,0.9);}},
  nova(a){const r=7*a.st.area*(1+0.25*M(a,'nova.wide'));const fr=2.6+1.2*M(a,'nova.deep');const p0=a.pos.clone();novaPulse(a,p0,r,fr,15);
    if(M(a,'nova.zero'))for(let i=1;i<=2;i++)later(0.45*i,()=>{if(!a.dead)novaPulse(a,a.pos.clone(),r*(1+0.25*i),fr*0.6,12);});
    const ns=10*M(a,'nova.shards');for(let i=0;i<ns;i++){const an=i/ns*TAU;const sh=new THREE.Mesh(OCT,cm(0xbff4ff,true));sh.scale.set(0.9,0.9,2.6);
      spawnProj({pos:new V3(p0.x,p0.y+1,p0.z),vel:new V3(Math.cos(an),0,Math.sin(an)).multiplyScalar(30),grav:0,r:0.3,life:0.8,mesh:sh,owner:'p',kind:'ice',pierce:true,
        trail:p=>PA.emit(p.pos.x,p.pos.y,p.pos.z,0,0,0,0.25,0.3,0.04,col(0x8fe6ff),0,0,0.8),onHit:(e,p)=>{damageEnt(e,9,{kb:2,src:p.pos,col:0x8fe6ff});status(e,{slow:2});}});}},
  orb(a){const from=a.hand.clone();const dir=aimDirFrom(from,a.aimPt,new V3());sfx('arcane');a.castKick=1;const big=M(a,'orb.big'),sc=1+0.35*big;const n=1+M(a,'orb.twin');
    for(let k=0;k<n;k++){const d=dir.clone();if(n>1)d.applyAxisAngle(UP,k?0.14:-0.14);makeOrb(a,from.clone(),d,sc,Math.round(26*sc),M(a,'orb.split'),M(a,'orb.sing'));}
    fx.burst(from,0xd08aff,20,4,0.4,0.35);},
  meteor(a){const c=a.aimPt.clone();if(a.pos.distanceTo(c)>40)return false;c.y=groundAt(c.x,c.z,c.y+1);const cata=M(a,'meteor.cata');const r=5.5*a.st.area*(1+0.4*cata*0.5);
    const dmg=Math.round(95*(1+0.4*cata));const inf=M(a,'meteor.inferno');sfx('arcane');a.casting=0.6;callMeteor(a,c,r,dmg,true,inf);
    const sh=2*M(a,'meteor.shower');for(let i=0;i<sh;i++){const an=(i/sh)*TAU+rand(-0.3,0.3);const off=new V3(c.x+Math.cos(an)*r*rand(0.8,1.4),0,c.z+Math.sin(an)*r*rand(0.8,1.4));
      later(0.3+i*0.28,()=>{off.y=groundAt(off.x,off.z,c.y+2);callMeteor(a,off,r*0.55,Math.round(dmg*0.4),false,0);});}
    if(M(a,'meteor.arma'))later(1.2,()=>zone({follow:a,pos:a.pos,r:0,life:4,every:0.38,onTick:z=>{const fs=foesNear(z.pos,16);const t=fs.length?pick(fs.slice(0,8)).pos.clone():new V3(z.pos.x+rand(-10,10),0,z.pos.z+rand(-10,10));
      t.y=groundAt(t.x,t.z,z.pos.y+3);callMeteor(z.a,t,r*0.5,Math.round(dmg*0.35),false,0);}}));},
  charge(a){const d=fwdOf(a,new V3());const hitSet=new Set();sfx('dash');sfx('shout');const T=0.38*(1+0.5*M(a,'charge.mom'));const ruin=M(a,'charge.ruin');let drop=0;
    a.dash={dir:d,speed:30,t:T,onTick:dt=>{for(const e of ents){if(e.dead||hitSet.has(e))continue;if(Math.hypot(e.pos.x-a.pos.x,e.pos.z-a.pos.z)<1.6+e.w&&Math.abs(e.pos.y-a.pos.y)<2.5){hitSet.add(e);
        damageEnt(e,26,{kb:10,src:a.pos,col:0xff8a6a});status(e,{stun:1.3});fx.burst(_t3.set(e.pos.x,e.pos.y+1,e.pos.z),0xffd0a0,16,5,0.4,0.3);fx.sparks(_t3,0xffd0a0,8,10);if(a.local)G.hitstop=0.05;}}
      PA.emit(a.pos.x+rand(-0.4,0.4),a.pos.y+rand(0.3,1.6),a.pos.z+rand(-0.4,0.4),-d.x*6,0,-d.z*6,0.3,0.4,0.05,col(0xff8a6a),0,2,0.9);
      if(ruin){drop-=dt||0.016;if(drop<=0){drop=0.07;const g=a.pos.clone();g.y=groundAt(g.x,g.z,g.y+1);firePool(g,1.5,4,6,false);}}},
      onEnd:()=>{fx.ring(a.pos,0xff8a6a,0.3,3,0.3);fx.shock(a.pos,0xff8a6a,0.3,3,0.3,1);if(a.local)G.shake=Math.min(1,G.shake+0.2);
        if(M(a,'charge.wave')){const c=a.pos.clone();sfx('boom');fx.ring(c,0xffd0a0,0.5,4.5,0.45);fx.shock(c,0xffd0a0,0.5,4.5,0.4,1.6);fx.smoke(c,0xb8a888,10,2.5,1,1.2);aoeEnemies(c,4.5,e=>{damageEnt(e,30,{kb:8,src:c,up:6});status(e,{stun:1});});}
        if(ruin){const c=a.pos.clone();explodeFire(_t1.set(c.x,c.y+0.6,c.z),4.5,40,0);fx.pillar(c,0xff7a2a,1.6,10,0.6);}}};
    if(a.local)a.invuln=0.3;},
  whirl(a){const life=2.5+1.5*M(a,'whirl.long');a.whirl=life;sfx('heavy');const r=4.2*a.st.area*(1+0.25*M(a,'whirl.wide'));const vortex=M(a,'whirl.vortex'),storm=M(a,'whirl.blades');let tk=0;
    zone({follow:a,pos:a.pos,r,life,every:0.25,onTick:z=>{sfx('swing');const c=a.pos;tk++;const n=aoeEnemies(c,r,e=>{damageEnt(e,11,{kb:vortex?0:3,src:c,col:0xffd0a0,quiet:true});
          if(vortex)status(e,{push:[(c.x-e.pos.x)*1.6,0,(c.z-e.pos.z)*1.6]});});if(n&&a.st.wwHeal)healActor(a,n*a.st.wwHeal);
        fx.arc(_t1.set(c.x,c.y+1,c.z),rand(0,TAU),r,2.6,0xffe0b0,0.2);
        if(storm&&tk%2===0){const off=rand(0,TAU);for(let i=0;i<3;i++){const an=off+i/3*TAU;spinBlade(a,new V3(c.x,c.y+1,c.z),new V3(Math.sin(an),0,Math.cos(an)),14);}}},
      onFrame:()=>{const c=a.pos;for(let i=0;i<3;i++){const an=rand(0,TAU);PA.emit(c.x+Math.cos(an)*r*0.8,c.y+rand(0.4,1.4),c.z+Math.sin(an)*r*0.8,-Math.sin(an)*8,0,Math.cos(an)*8,0.25,0.35,0.05,col(vortex?0xffb070:0xffe0b0),0,2,0.8);}}});},
  slam(a){let to=a.aimPt.clone();const f=fwdOf(a,new V3());const dd=Math.hypot(to.x-a.pos.x,to.z-a.pos.z);if(dd>13)to.copy(a.pos).addScaledVector(f,13);if(dd<2)to.copy(a.pos).addScaledVector(f,3);
    to.y=groundAt(to.x,to.z,to.y+2);if(collides(to.x,to.y,to.z,0.3,1.8))to.y+=1;
    const r=6*a.st.area*(1+0.3*M(a,'slam.wide'));const stun=1+M(a,'slam.quake');const after=M(a,'slam.after');const fis=M(a,'slam.fissure');fx.telegraph(to,0xd9a46a,r,0.55);sfx('heavy');if(a.local)a.invuln=0.6;
    a.leap={from:a.pos.clone(),to,t:0,dur:0.55,height:5,onLand:()=>{sfx('boom');fx.decal(to,'crack',r*0.85,9,0xff9a4a);fx.flare(_t1.copy(to).setY(to.y+0.8),0xffd0a0,8,0.35,true);
      fx.spikes(to,0x8a7a6a,12,r*0.55,0.9,1.4,cm(0x8a7a66));if(a.local){G.shake=1;G.hitstop=0.08;}fx.ring(to,0xffd0a0,0.5,r,0.5);fx.ring(to,0xffffff,0.5,r*0.7,0.3);fx.shock(to,0xffd0a0,0.6,r,0.45,2);
      fx.smoke(to,0xb8a888,16,3,1.2,1.4);flashLight(_t1.copy(to).setY(to.y+1),0xffb070,3,14,0.4);
      const b=world.get(Math.floor(to.x),Math.floor(to.y)-1,Math.floor(to.z));const cols=[b&&BL[b]?BL[b].c.getHex():0x8a5b34,0x8a5b34,0x8c9097];
      for(let i=0;i<26;i++){const an=rand(0,TAU),s=rand(3,9);DB.emit(to.x+Math.cos(an),to.y+0.3,to.z+Math.sin(an),pick(cols),rand(0.2,0.45),Math.cos(an)*s,rand(5,11),Math.sin(an)*s,rand(1,2));}
      aoeEnemies(to,r,e=>{damageEnt(e,42,{kb:9,src:to,up:8,col:0xffd0a0});status(e,{stun});});
      for(let i=1;i<=after;i++)later(0.45*i,()=>{const rr=r*(1+0.25*i);sfx('heavy');fx.ring(to,0xffb070,0.5,rr,0.45);fx.shock(to,0xffb070,0.5,rr,0.4,1.2);fx.smoke(to,0xb8a888,8,2.5,1,1.2);if(a.local)G.shake=Math.min(1,G.shake+0.4);
        aoeEnemies(to,rr,e=>{damageEnt(e,25,{kb:6,src:to,up:5,col:0xffb070});status(e,{stun:0.6});});});
      if(fis){const off=rand(0,TAU);for(let j=0;j<6;j++){const an=off+j/6*TAU,dx=Math.cos(an),dz=Math.sin(an);for(let k=1;k<=5;k++)later(0.07*k,()=>{
        const p=new V3(to.x+dx*(r*0.5+k*1.7),0,to.z+dz*(r*0.5+k*1.7));p.y=groundAt(p.x,p.z,to.y+3);sfx('heavy');fx.smoke(p,0x8a7458,3,1.5,0.7,0.9);fx.burst(_t1.set(p.x,p.y+0.3,p.z),0xff9a4a,8,4,0.35,0.35,{up:4});
        for(let q=0;q<3;q++)DB.emit(p.x+rand(-0.4,0.4),p.y+0.2,p.z+rand(-0.4,0.4),pick(cols),rand(0.25,0.45),rand(-1.5,1.5),rand(7,11),rand(-1.5,1.5),rand(1,1.6));
        aoeEnemies(p,1.7,e=>{damageEnt(e,22,{kb:2,src:p,up:7,col:0xffb070,quiet:true});status(e,{stun:0.4});});});}}}};},
  cry(a){const dur=8+2*M(a,'cry.rage');a.buffs.cry=dur;a.cryLeech=0.1+0.1*M(a,'cry.rage');healActor(a,20);sfx('shout');if(a.local)G.shake=Math.min(1,G.shake+0.3);
    fx.ring(a.pos,0xff5b4c,0.5,9,0.6);fx.ring(a.pos,0xffb03a,0.5,6,0.45);fx.shock(a.pos,0xff5b4c,0.5,9,0.5,2.4);fx.burst(_t1.set(a.pos.x,a.pos.y+1.2,a.pos.z),0xff5b4c,40,7,0.6,0.4,{up:2});
    const terror=M(a,'cry.terror'),wl=M(a,'cry.warlord');aoeEnemies(a.pos,(terror?11:9)*(wl?1.5:1),e=>status(e,terror?{fear:3}:{stun:0.8,fear:1.5}));
    if(M(a,'cry.rally')){for(const o of avatars())if(o!==a&&o.pos.distanceTo(a.pos)<15){o.buffs.cry=dur;fx.ring(o.pos,0xff5b4c,0.4,2.5,0.5);}
      for(const p of alliesNear(a,15)){p.buffs.cry=dur;healPlayer(20);}}},
  mend(a){const amt=a.maxHp*0.35*(1+0.15*M(a,'mend.greater'));const hot=4*(1+M(a,'mend.renew'));healActor(a,amt);a.buffs.hot=hot;sfx('heal');healFx(a.pos);
    if(M(a,'mend.prayer')){for(const o of avatars())if(o!==a&&!o.dead&&o.pos.distanceTo(a.pos)<18){healFx(o.pos);o.buffs.hot=hot;beam(a.pos,o.pos,0x8dff9e);}
      for(const p of alliesNear(a,18)){healPlayer(p.maxHp*0.35*(1+0.15*M(a,'mend.greater')));}}
    if(a.st.radiance){aoeEnemies(a.pos,6,e=>damageEnt(e,40,{kb:4,src:a.pos,col:0xffe48a}));fx.ring(a.pos,0xffe48a,0.5,6,0.4);fx.shock(a.pos,0xffe48a,0.5,6,0.4,1.6);}
    if(M(a,'mend.bloom')){const c=a.pos.clone();c.y=groundAt(c.x,c.z,c.y+1);const R=5*a.st.area;fx.disk(c,0x8dff9e,R,6,0.16);fx.ring(c,0x8dff9e,R,R,6,0.13,0.6);
      zone({pos:c,r:R,life:6,every:0.5,onTick:z=>{aoeEnemies(z.pos,z.r,e=>damageEnt(e,9,{kb:0,src:z.pos,col:0xffe48a,quiet:true}));
          if(!P.dead&&Math.hypot(P.pos.x-z.pos.x,P.pos.z-z.pos.z)<z.r&&(z.a.local||NET.on))healPlayer(3,true);},
        onFrame:z=>{for(let i=0;i<2;i++){const an=rand(0,TAU),rr=Math.sqrt(Math.random())*z.r;PA.emit(z.pos.x+Math.cos(an)*rr,z.pos.y+0.2,z.pos.z+Math.sin(an)*rr,rand(-0.3,0.3),rand(0.8,2),rand(-0.3,0.3),1.1,0.3,0.05,col(pick([0x8dff9e,0xffb8e0,0xffffff])),0,0.5,0.9);}}});}},
  smite(a){const c=a.aimPt.clone();if(a.pos.distanceTo(c)>38)return false;c.y=groundAt(c.x,c.z,c.y+1);const r=2.9*a.st.area;const dmg=Math.round(48*(1+0.4*M(a,'smite.wrath')));sfx('holy');a.castKick=1;
    smiteAt(a,c,r,dmg);const chain=2*M(a,'smite.chain');
    if(M(a,'smite.judg'))for(let i=1;i<=2;i++)later(0.5*i,()=>smiteAt(a,c,r*(1+0.15*i),Math.round(dmg*0.7)));
    if(chain)later(0.45,()=>{const cand=ents.filter(e=>!e.dead&&!e.passive&&Math.hypot(e.pos.x-c.x,e.pos.z-c.z)<12&&Math.hypot(e.pos.x-c.x,e.pos.z-c.z)>r).sort((p,q)=>p.pos.distanceTo(c)-q.pos.distanceTo(c)).slice(0,chain);
      cand.forEach((e,i)=>{const p=e.pos.clone();later(0.12*i,()=>{beam(c,p,0xffe48a);smiteAt(a,p,r*0.8,Math.round(dmg*0.7));});});});},
  shield(a){const dur=3+1.5*M(a,'shield.long');a.buffs.shield=dur;sfx('holy');shieldOn(a,dur);fx.ring(a.pos,0xfff3b8,0.5,5,0.5);fx.shock(a.pos,0xfff3b8,0.5,5,0.4,2);
    aoeEnemies(a.pos,4.5,e=>status(e,{push:[(e.pos.x-a.pos.x)*3,5,(e.pos.z-a.pos.z)*3]}));
    if(M(a,'shield.aegis')){for(const o of avatars())if(o!==a&&!o.dead&&o.pos.distanceTo(a.pos)<12){o.buffs.shield=dur;shieldOn(o,dur);}for(const p of alliesNear(a,12))p.buffs.shield=dur;}
    if(M(a,'shield.thorns'))zone({follow:a,pos:a.pos,r:4.5,life:dur,every:0.5,onTick:z=>{aoeEnemies(a.pos,4.5,e=>{damageEnt(e,14,{kb:4,src:a.pos,col:0xfff3b8,quiet:true});});fx.ring(a.pos,0xfff3b8,0.5,4.5,0.3);},
      onFrame:()=>{const an=rand(0,TAU);PA.emit(a.pos.x+Math.cos(an)*1.7,a.pos.y+rand(0.2,1.8),a.pos.z+Math.sin(an)*1.7,Math.cos(an)*4,0.5,Math.sin(an)*4,0.35,0.3,0.05,col(0xfff3b8),0,1,1);}});
    if(M(a,'shield.retri'))later(dur,()=>{const c=a.pos.clone();sfx('boom');fx.ring(c,0xffe48a,0.5,6.5,0.5);fx.shock(c,0xffe48a,0.5,6.5,0.45,2.2);fx.pillar(c,0xffe48a,1.5,12,0.6);fx.burst(_t1.set(c.x,c.y+1,c.z),0xffe48a,50,8,0.6,0.4);
      aoeEnemies(c,6.5,e=>{damageEnt(e,50,{kb:8,src:c,up:5,col:0xffe48a});status(e,{stun:0.8});});});},
  consecrate(a){const march=M(a,'consecrate.march');const c=a.pos.clone();const r=6*a.st.area*(1+0.25*M(a,'consecrate.wide'));const dmg=Math.round(9*(1+0.5*M(a,'consecrate.purge')));const sanct=M(a,'consecrate.sanct');const life=6+(march?3:0);
    sfx('holy');fx.ring(c,0xffd35a,0.5,r,0.5);fx.shock(c,0xffd35a,0.5,r,0.4,1.4);const dk=fx.disk(c,0xffd35a,r,life,0.22),rg=fx.ring(c,0xffe48a,r,r,life,0.13,0.8);
    zone({follow:march?a:null,pos:c,r,life,every:0.5,onTick:z=>{aoeEnemies(z.pos,z.r,e=>damageEnt(e,dmg,{kb:0,src:z.pos,col:0xffd35a,quiet:true}));
        if(a.local&&Math.hypot(P.pos.x-z.pos.x,P.pos.z-z.pos.z)<z.r)healPlayer(4*(1+sanct));
        if(!a.local&&sanct&&!P.dead&&Math.hypot(P.pos.x-z.pos.x,P.pos.z-z.pos.z)<z.r)healPlayer(6);},
      onFrame:z=>{if(march){dk.mesh.position.set(z.pos.x,z.pos.y+0.1,z.pos.z);rg.mesh.position.set(z.pos.x,z.pos.y+0.13,z.pos.z);}
        for(let i=0;i<4;i++){const an=rand(0,TAU),rr=Math.sqrt(Math.random())*z.r;PA.emit(z.pos.x+Math.cos(an)*rr,z.pos.y+0.2,z.pos.z+Math.sin(an)*rr,0,rand(1,3),0,0.9,0.35,0.05,col(pick([0xffd35a,0xffffff,0xffb03a])),0,0.5,0.9);}}});},
};
/* a translucent copy of an actor frozen mid-pose that fades out: dashes, blinks and charges leave a trail of these */
function afterimage(a,c,life=0.4){if(!a.model)return;const g=ghostModel(a,c);const src=a.model;g.root.position.copy(src.root.position);g.root.rotation.copy(src.root.rotation);
  for(const k of['body','torso','head','armL','armR','legL','legR'])if(src[k]&&g[k]){g[k].rotation.copy(src[k].rotation);g[k].position.copy(src[k].position);}
  g.mats.forEach(x=>{x.blending=THREE.AdditiveBlending;x.color.set(c);x.opacity=0.5;});
  addFx(g.root,life,(f,t)=>{for(const x of g.mats)x.opacity=0.5*(1-t);g.root.scale.setScalar(1+t*0.08);});}
const _lk=new V3();
/* cast flourishes for every ability: a body pose plus magic circles, flares and trails. They wrap ABILITY_FN,
   so they also play when a teammate's cast is replayed in co-op. Impacts get their own effects where they land. */
const CAST_FX={
  volley:a=>{const h=a.hand.clone();fx.circle(h,0x9be07a,1.1,0.45,{style:'nature',dir:lookOf(a,_lk),y:0,spin:3});fx.flare(h,0xc8ffa0,2.2,0.25,true);},
  roll:a=>{for(let i=0;i<3;i++)later(0.05+i*0.07,()=>afterimage(a,0x9be07a,0.35));},
  snare:a=>{castAnim(a,'throw',0.45);const c=a.aimPt.clone();c.y=groundAt(c.x,c.z,c.y+1);fx.circle(c,0xd9b46a,1.6,0.7,{style:'war',spin:2});},
  storm:a=>{castAnim(a,'bowup',0.6);const h=a.hand.clone();fx.streak(h,h.clone().setY(h.y+20),0xb8ff8a,0.35,0.22);fx.flare(h,0xb8ff8a,2.5,0.3,true);
    const eye=M(a,'storm.eye');const c=eye?a.pos.clone():a.aimPt.clone();c.y=groundAt(c.x,c.z,c.y+1);const r=6*a.st.area*(1+0.25*M(a,'storm.wide'));
    fx.circle(c,M(a,'storm.frost')?0x9fe6ff:0x7fd46a,r,4+1.5*M(a,'storm.long')+(eye?2:0),{style:'nature',op:0.55,spin:0.4,follow:eye?a:null});},
  blink:a=>{fx.flare(_t1.set(a.pos.x,a.pos.y+1,a.pos.z),0xd08aff,4,0.35,true);fx.circle(a.pos,0xc4a8ff,2.4,0.6,{style:'arcane',spin:4});},
  nova:a=>{castAnim(a,'kneel',0.55);const p=a.pos.clone(),r=7*a.st.area*(1+0.25*M(a,'nova.wide'));fx.circle(p,0x8fe6ff,r*0.9,0.9,{style:'frost',spin:2});
    fx.spikes(p,0xbff4ff,14,r*0.45,1.1,1.7);later(0.08,()=>fx.spikes(p,0x8fe6ff,20,r*0.8,1.0,1.2));fx.decal(p,'frost',r,5,0x5ab8ff);fx.flare(_t1.set(p.x,p.y+1,p.z),0xdff8ff,7,0.4);},
  orb:a=>{castAnim(a,'push',0.4);const h=a.hand.clone();fx.circle(h,0xd08aff,1.3,0.5,{style:'arcane',dir:lookOf(a,_lk),y:0,spin:4});fx.flare(h,0xd08aff,3,0.3);},
  meteor:a=>{castAnim(a,'raise',0.9);const c=a.aimPt.clone();c.y=groundAt(c.x,c.z,c.y+1);const r=5.5*a.st.area;fx.circle(c,0xff6a1a,r*1.1,1.25,{style:'fire',spin:1.5});
    fx.circle(_t1.set(a.pos.x,a.pos.y+3.4,a.pos.z),0xff8a3a,1.6,0.9,{style:'fire',spin:5,y:0});fx.flare(_t1,0xffb070,3,0.6);},
  charge:a=>{castAnim(a,'lunge',0.42*(1+0.5*M(a,'charge.mom')));for(let i=0;i<5;i++)later(i*0.07,()=>{afterimage(a,0xff8a6a,0.3);const p=_t1.set(a.pos.x+rand(-0.6,0.6),a.pos.y+rand(0.4,1.6),a.pos.z+rand(-0.6,0.6));
    const f=fwdOf(a,_lk);fx.streak(p.clone(),p.clone().addScaledVector(f,-3.5),0xffd0a0,0.2,0.07);});},
  whirl:a=>{fx.circle(a.pos,0xffd0a0,4.2*a.st.area*(1+0.25*M(a,'whirl.wide')),2.5+1.5*M(a,'whirl.long'),{style:'war',spin:-6,op:0.5,follow:a});},
  cry:a=>{castAnim(a,'shout',0.75);const p=_t1.set(a.pos.x,a.pos.y+1.2,a.pos.z).clone();fx.sphere(p,0xff5b4c,0.5,9,0.5);fx.circle(a.pos,0xff5b4c,6,0.8,{style:'war',spin:3});fx.flare(p,0xff8a6a,5,0.4,true);},
  mend:a=>{castAnim(a,'pray',0.8);fx.circle(a.pos,0x8dff9e,3.2,1.4,{style:'nature',spin:1.5});fx.flare(_t1.set(a.pos.x,a.pos.y+2.2,a.pos.z),0xb8ffc8,3.5,0.6,true);},
  smite:a=>{castAnim(a,'point',0.55);const c=a.aimPt.clone();c.y=groundAt(c.x,c.z,c.y+1);fx.circle(c,0xffe48a,2.9*a.st.area*1.3,0.9,{style:'holy',spin:3});
    fx.circle(_t1.set(c.x,c.y+9,c.z),0xffe48a,2.2,0.7,{style:'holy',spin:-4,y:0});},
  shield:a=>{fx.circle(a.pos,0xfff3b8,2.6,3+1.5*M(a,'shield.long'),{style:'holy',spin:1.2,op:0.7,follow:a});fx.flare(_t1.set(a.pos.x,a.pos.y+1.2,a.pos.z),0xfff3b8,5,0.4,true);},
  consecrate:a=>{castAnim(a,'pray',0.6);const r=6*a.st.area*(1+0.25*M(a,'consecrate.wide'));const march=M(a,'consecrate.march');
    fx.circle(a.pos,0xffd35a,r,6+(march?3:0),{style:'holy',spin:0.5,op:0.7,follow:march?a:null});},
};
for(const id in CAST_FX){const f=ABILITY_FN[id];ABILITY_FN[id]=a=>{const r=f(a);if(r!==false)CAST_FX[id](a);return r;};}
function healFx(pos){fx.pillar(pos.clone(),0x8dff9e,1.1,6,0.9);for(let i=0;i<30;i++){const an=i/30*TAU*3;PA.emit(pos.x+Math.cos(an)*1.1,pos.y+i*0.06,pos.z+Math.sin(an)*1.1,-Math.sin(an)*1.5,2.5,Math.cos(an)*1.5,0.9,0.4,0.05,col(pick([0x8dff9e,0xffffff,0xffe48a])),0,0.5,1);}}
function beam(a,b,c){const n=Math.min(30,Math.ceil(a.distanceTo(b)*1.5));for(let i=0;i<=n;i++){const t=i/n;PA.emit(lerp(a.x,b.x,t),lerp(a.y,b.y,t)+1.2+Math.sin(t*Math.PI)*1.5,lerp(a.z,b.z,t),0,0,0,0.5,0.35,0.05,col(c),0,0,1);}}
function makeOrb(a,from,dir,sc,dmg,split,sing){let pull=0;
  spawnProj({a,pos:from,vel:dir.multiplyScalar(12),grav:0,r:0.9*sc,life:2.4,mesh:glowBall(0xb46aff,0.6*sc),owner:'p',pierce:true,kind:'orb',
    trail:p=>{p.mesh.rotation.y+=0.2;for(let i=0;i<3;i++){const an=rand(0,TAU),rr=rand(0.8,1.6)*sc*(sing?2.2:1);PA.emit(p.pos.x+Math.cos(an)*rr,p.pos.y+rand(-0.8,0.8)*sc,p.pos.z+Math.sin(an)*rr,-Math.cos(an)*3*(sing?2:1),0,-Math.sin(an)*3*(sing?2:1),0.35,0.3,0.05,col(pick([0xd08aff,0x8a5aff,0xffffff])),0,0,1);}
      if(sing&&p.a.local&&(pull-=1/60)<=0){pull=0.2;let n=0;for(const e of ents){if(e.dead||e.passive||e.T.prop||n>=10)continue;const dx=p.pos.x-e.pos.x,dz=p.pos.z-e.pos.z,d=Math.hypot(dx,dz);if(d<6&&d>0.8){n++;status(e,{push:[dx/d*7,1.5,dz/d*7]});}}}},
    onHit:(e,p)=>{damageEnt(e,dmg,{kb:2,src:p.pos,col:0xd08aff});fx.burst(_t3.set(e.pos.x,e.pos.y+e.h*0.5,e.pos.z),0xd08aff,14,4,0.4,0.3);},
    onWorld:p=>orbBurst(p.pos,sc*(sing?1.5:1),split),onExpire:p=>orbBurst(p.pos,sc*(sing?1.5:1),split)});
}
function orbBurst(pos,sc,split){sfx('arcane');const c=pos.clone();fx.flare(c,0xd08aff,6*sc,0.35,true);fx.sphere(c,0xb46aff,0.3,3.5*sc,0.35);const r=3.5*sc;fx.burst(c,0xd08aff,40,6,0.5,0.45);fx.ring(_t3.copy(c).setY(c.y-0.5),0xd08aff,0.5,r,0.4);fx.shock(_t3.copy(c).setY(c.y-0.8),0xd08aff,0.4,r,0.35,1.4);flashLight(c,0xb46aff,3,12,0.35);
  aoeEnemies(c,r,e=>damageEnt(e,Math.round(20*sc),{kb:6,src:c,col:0xd08aff}));
  if(split){const n=1+2*split;for(let i=0;i<n;i++){const an=i/n*TAU;
    spawnProj({pos:c.clone(),vel:new V3(Math.cos(an),0.1,Math.sin(an)).multiplyScalar(15),grav:0,r:0.55,life:0.75,mesh:glowBall(0xd08aff,0.3),owner:'p',pierce:true,kind:'orb',
      trail:p=>PA.emit(p.pos.x,p.pos.y,p.pos.z,0,0,0,0.3,0.4,0.05,col(0xd08aff),0,0,1),onHit:(e,p)=>damageEnt(e,12,{kb:3,src:p.pos,col:0xd08aff}),
      onExpire:p=>{fx.burst(p.pos,0xd08aff,14,4,0.35,0.3);aoeEnemies(p.pos,2,e=>damageEnt(e,10,{kb:3,src:p.pos,col:0xd08aff}));}});}}}
function meteorImpact(c,r,dmg,big,inferno){sfx('boom');if(ACTOR.local){G.shake=Math.min(1,G.shake+(big?1:0.4));G.hitstop=big?0.1:0;}
  fx.burst(_t1.copy(c).setY(c.y+0.5),0xff7a22,big?90:35,r*3,0.8,big?0.9:0.6,{up:3,drag:2.5});fx.burst(_t1,0xffe0a0,big?30:10,r*2,0.5,0.6);
  fx.decal(c,'scorch',r*(big?0.95:0.6),big?14:7);if(big){fx.decal(c,'crack',r*0.8,10,0xff7a2a);fx.pillar(c.clone(),0xff7a2a,r*0.45,18,0.7);}fx.flare(_t1.copy(c).setY(c.y+1),0xffb070,big?14:6,0.45,true);fx.sphere(_t1.clone(),0xff8a3a,0.5,r*1.1,0.4);
  fx.smoke(c,0x2a2420,big?24:8,3.5,2.2,2);fx.ring(c,0xff8a3a,0.5,r*1.5,0.6);fx.shock(c,0xff8a3a,0.5,r*1.3,0.5,big?3:1.5);fx.sparks(_t1.copy(c).setY(c.y+0.5),0xffd27a,big?24:8,14,0.5);fx.ring(c,0xffffff,0.5,r,0.35);flashLight(_t1.copy(c).setY(c.y+2),0xff7a2a,big?7:3,r*6,0.8);
  if(big){const cr=2.4,cx=Math.floor(c.x),cy=Math.floor(c.y)-1,cz=Math.floor(c.z);
    for(let dy=-3;dy<=3;dy++)for(let dz=-3;dz<=3;dz++)for(let dx=-3;dx<=3;dx++){if(dx*dx+dy*dy*1.5+dz*dz>cr*cr)continue;const y=cy+dy;if(y<2)continue;const b=world.get(cx+dx,y,cz+dz);if(!b)continue;
      world.set(cx+dx,y,cz+dz,0);if(Math.random()<0.5){const s=rand(5,12),an=rand(0,TAU);DB.emit(cx+dx+0.5,y+0.5,cz+dz+0.5,BL[b].c.getHex(),rand(0.25,0.5),Math.cos(an)*s*0.6,rand(6,13),Math.sin(an)*s*0.6,rand(1.5,2.5));}}
    for(let dz=-3;dz<=3;dz++)for(let dx=-3;dx<=3;dx++){if(dx*dx+dz*dz>9||hash2(cx+dx,cz+dz,world.seed+77)>0.4)continue;const t=world.topAt(cx+dx,cz+dz);const b=world.get(cx+dx,t,cz+dz);if(b&&b!==B.EMBER)world.set(cx+dx,t,cz+dz,B.DSTONE);}}
  aoeEnemies(c,r,e=>{damageEnt(e,dmg,{kb:big?12:6,src:c,col:0xff8a3a,up:big?10:5});status(e,{burn:3});});
  if(inferno){zone({pos:c.clone(),r:r*0.8,life:5,every:0.5,onTick:z=>aoeEnemies(z.pos,z.r,e=>{damageEnt(e,6,{quiet:true,col:0xff7a22});status(e,{burn:2});}),
    onFrame:z=>{for(let i=0;i<5;i++){const an=rand(0,TAU),rr=Math.sqrt(Math.random())*z.r;PA.emit(z.pos.x+Math.cos(an)*rr,z.pos.y+0.2,z.pos.z+Math.sin(an)*rr,rand(-0.3,0.3),rand(1.5,3.5),rand(-0.3,0.3),0.6,0.6,0.08,col(pick([0xff6a1a,0xffb03a,0xff3a10])),-1,0.5,1);}}});}
  if(ACTOR.local&&Math.hypot(P.pos.x-c.x,P.pos.z-c.z)<r*0.6&&big)P.vel.y=8;}

/* ---------------- damage & status (authority aware) ---------------- */
function damageEnt(e,amount,o={}){
  if(e.dead)return;const A=ACTOR||P;const hp=_t3.set(e.pos.x,e.pos.y+e.h*0.6,e.pos.z);
  if(!A.local){PA.emit(hp.x,hp.y,hp.z,0,0,0,0.12,1.2,0.2,col(o.col||0xffffff),0,0,0.8);return;}
  const crit=Math.random()<A.st.crit;let a=amount*dmgMul(A)*(e.elite?1+A.st.elite:1)*((e.frozen>0||e.frozenFlag)?1.25+A.st.shatter:1)*(crit?2:1);
  if(e.affix&&e.affix.k==='brute')a*=0.8;a=Math.max(1,Math.round(a*rand(0.92,1.08)));
  if(!o.quiet||Math.random()<0.5)popText(_t3.set(e.pos.x,e.pos.y+e.h+0.2,e.pos.z),String(a),crit?'crit':'');
  if(crit){sfx('crit');if(A===P)augCrit(e);if(G.runT>(damageEnt.cf||0)){damageEnt.cf=G.runT+0.08;fx.flare(_t3.set(e.pos.x,e.pos.y+e.h*0.6,e.pos.z),0xffe8a0,1.8,0.18,true);}}else sfx('hit');
  const bl=e.T.blood;hp.set(e.pos.x,e.pos.y+e.h*0.6,e.pos.z);
  PA.emit(hp.x,hp.y,hp.z,0,0,0,0.12,crit?2.2:1.4,0.2,col(o.col||0xffffff),0,0,1);
  for(let i=0;i<(crit?10:5);i++){PN.emit(hp.x,hp.y,hp.z,rand(-4,4),rand(1,5),rand(-4,4),rand(0.3,0.6),0.25,0.08,col(pick(bl)),14,1,1);}
  const leech=A.st.leech+(A.buffs.cry?A.cryLeech:0);if(leech)healPlayer(a*leech,true);
  e.flash=0.1;e.hitP=0.14;$('#cross').classList.add('hit');clearTimeout(damageEnt._t);damageEnt._t=setTimeout(()=>$('#cross').classList.remove('hit'),110);
  if(NET.on&&!NET.host){send({t:'hit',to:NET.hostId,id:e.id,a,kb:o.kb||0,up:o.up||0,s:o.src?[r2(o.src.x),r2(o.src.z)]:null});e.hp=Math.max(1,e.hp-a);return;}
  applyDamage(e,a,o);
}
function applyDamage(e,a,o){if(e.dead)return;e.hp-=a;e.flash=0.1;e.hitP=0.14;e.alert=true;if(e.passive)e.flee=4;
  if(o.src&&o.kb&&!e.elite&&!e.T.prop){const kr=e.T.kbRes||0;o=Object.assign({},o,{kb:o.kb*(1-kr),up:(o.up||0)*(1-kr)});_t2.set(e.pos.x-o.src.x,0,e.pos.z-o.src.z);if(_t2.lengthSq()<0.001)_t2.set(rand(-1,1),0,rand(-1,1));_t2.normalize().multiplyScalar(o.kb);e.vel.x+=_t2.x;e.vel.z+=_t2.z;e.vel.y=Math.max(e.vel.y,o.up||Math.min(5,o.kb*0.6));}
  if(o.by&&NET.on){const r=NET.peers.get(o.by);if(r&&!e.passive)e.aggroOn=r;}
  if(e.hp<=0)killEnt(e);}
function status(e,s){if(e.dead||!(ACTOR||P).local)return;if(s.freeze&&!AUTH())freezeFx(e);if(NET.on&&!NET.host){send({t:'st',to:NET.hostId,id:e.id,s});return;}applyStatus(e,s);}
function applyStatus(e,s){if(e.dead)return;
  if(e.T.prop)return;if(s.stun)e.stun=Math.max(e.stun||0,e.elite?s.stun*0.5:s.stun);if(s.fear)e.fear=Math.max(e.fear||0,s.fear);if(s.slow)e.slow=Math.max(e.slow||0,s.slow);
  if(s.burn)e.burn=Math.max(e.burn||0,s.burn);if(s.poison)e.poison=Math.max(e.poison||0,s.poison);
  if(s.freeze)freeze(e,e.elite?s.freeze*0.5:s.freeze);if(s.chill){e.chill=(e.chill||0)+1;if(e.chill>=(s.thr||5)){e.chill=0;freeze(e,1.6);}}
  if(s.push&&!e.elite&&!e.T.prop){e.vel.x+=s.push[0];e.vel.y=Math.max(e.vel.y,s.push[1]);e.vel.z+=s.push[2];}}
function freeze(e,t){e.frozen=Math.max(e.frozen||0,t);freezeFx(e);}
function freezeFx(e){for(const m of e.mats)if(m.emissive)m.emissive.setHex(0x2a6a8a);fx.burst(_t3.set(e.pos.x,e.pos.y+e.h*0.5,e.pos.z),0xbff4ff,16,4,0.5,0.3);
  DB.burst(_t3.set(e.pos.x,e.pos.y+0.3,e.pos.z),[0xbff4ff,0x8fe6ff,0xffffff],6,3,0.14,4);}

/* ---------------- enemies ----------------
   Hostiles always hunt the nearest player, like a survivors-style horde: no wandering, no idling.
   The director (src/director.js) decides what spawns and when; this section is how each kind behaves. */
const ETYPES={
  zombie:{name:'Zombie',hp:40,dmg:9,speed:3.3,reach:1.7,cd:1.2,wind:0.38,xp:3,w:0.32,h:1.85,blood:[0x5f9a4a,0x3a6ea5,0x2a4a2a]},
  skeleton:{name:'Skeleton',hp:30,dmg:8,speed:3.5,ranged:true,range:20,cd:2.7,wind:0.65,xp:4,w:0.28,h:1.85,blood:[0xe8e4d8,0xc8c2b0]},
  slime:{name:'Slime',hp:30,dmg:7,speed:4.4,xp:2,w:0.45,h:0.9,blood:[0x6fd46a,0x3fa83a]},
  spider:{name:'Spider',hp:26,dmg:8,speed:5.8,reach:1.6,cd:1.0,wind:0.25,xp:3,w:0.5,h:0.8,blood:[0x2a2228,0x6a2a2a]},
  cultist:{name:'Cultist',hp:40,dmg:11,speed:3.1,ranged:true,range:16,cd:3.4,wind:0.8,xp:5,w:0.32,h:1.85,blood:[0x4a2a6a,0x9a4aff]},
  golem:{name:'Ancient Golem',hp:620,dmg:26,speed:2.8,reach:3.2,cd:3,wind:1.0,xp:90,w:1.0,h:3.7,elite:true,kbRes:1,blood:[0x7d8088,0x5c7a42,0x6ff3ff]},
  bat:{name:'Bat',hp:12,dmg:5,speed:7.4,fly:true,cd:1.1,xp:1,w:0.35,h:0.6,blood:[0x3a2a40,0x8a2a3a]},
  bloater:{name:'Bloater',hp:55,dmg:22,speed:2.7,wind:0.75,xp:4,w:0.45,h:1.9,blood:[0x9ab860,0xc8ff5a,0x5a7a32]},
  brute:{name:'Brute',hp:230,dmg:20,speed:3.0,reach:2.8,cd:2.3,wind:0.7,xp:16,w:0.55,h:2.9,kbRes:0.7,blood:[0x9a7a62,0x6a4a34,0xb03a2a]},
  urn:{name:'Ancient Urn',hp:14,prop:true,xp:0,w:0.4,h:1.1,blood:[0xb0683e,0x7a4428,0xe0b84a]},
  pig:{name:'Pig',hp:20,speed:2.6,passive:true,xp:2,w:0.4,h:0.9,blood:[0xf0a0a8,0xd88088]},
  sheep:{name:'Sheep',hp:22,speed:2.4,passive:true,xp:2,w:0.42,h:1.0,blood:[0xf2f0ea,0xd0ccc4]},
  chicken:{name:'Chicken',hp:8,speed:3.0,passive:true,xp:1,w:0.22,h:0.6,blood:[0xffffff,0xf0e0c0]},
};
function makeEnemyModel(type,o){
  let m;switch(type){
    case'zombie':{m=humanoid({skin:pick([0x6aa050,0x7aa860,0x5e9448]),shirt:pick([0x3a6ea5,0x7a4a3a,0x4a5a3a]),pants:0x3b3355,eye:0x101010,eyeGlow:Math.random()<0.3?0xd8ff5a:null});
      const tr=matL(m.mats,0x2a3a22);mk(m.torso,0.2,0.18,0.04,tr,0.12,0.18,0.16);mk(m.head,0.2,0.12,0.04,matL(m.mats,0x4a7a38),-0.1,0.42,0.2);break;}
    case'skeleton':{m=humanoid({skin:0xe8e4d8,shirt:0xd8d4c6,pants:0xd0ccbc,thin:true,eyeGlow:0xff5a3a});const dk=matL(m.mats,0x8a8478);for(let i=0;i<3;i++)mk(m.torso,0.42,0.04,0.04,dk,0,0.5-i*0.13,0.12);
      const b=WMODEL.longbow(m.mats);m.handL.add(b);break;}
    case'cultist':{m=humanoid({skin:0x2a1f33,shirt:0x3a2350,pants:0x241634,robe:0x2e1c44,belt:0x9a4aff,eyeGlow:0xd07aff});
      const hm=matL(m.mats,0x2e1c44);mk(m.head,0.5,0.14,0.5,hm,0,0.47,-0.02);mk(m.head,0.5,0.46,0.1,hm,0,0.24,-0.25);mk(m.head,0.12,0.12,0.4,hm,0,0.58,-0.18);
      mk(m.torso,0.1,0.1,0.02,matB(m.mats,0xd07aff),0,0.42,0.16);break;}
    case'slime':return slimeModel(o.color,o.size);
    case'spider':m=spiderModel();break;
    case'golem':m=golemModel();break;
    case'bat':m=batModel();break;
    case'bloater':m=bloaterModel();break;
    case'brute':m=bruteModel();break;
    case'urn':m=urnModel();break;
    case'pig':m=quadModel({body:0xf0a0a8,head:0xf2adb4,snout:0xd88088,leg:0xe090a0,bw:0.7,bh:0.55,bl:1.0,hw:0.5,hh:0.45,hl:0.42,legH:0.35,lw:0.18});break;
    case'sheep':m=quadModel({body:0xf4f2ec,head:0x6a625a,leg:0x5a524a,bw:0.8,bh:0.62,bl:1.0,hw:0.38,hh:0.4,hl:0.42,legH:0.42,lw:0.16});break;
    case'chicken':m=quadModel({body:0xffffff,head:0xffffff,snout:0xf2b030,leg:0xf2b030,bw:0.34,bh:0.32,bl:0.42,hw:0.22,hh:0.26,hl:0.22,legH:0.25,lw:0.06,comb:true,two:true});break;
  }
  return bakeModel(m,o.shadow!==false);
}
function playerCount(){let n=1;for(const r of NET.peers.values())if(r.inGame)n++;return n;}
function spawnEnt(type,x,y,z,o={}){
  if(ents.length>=CAP.ents+(o.net?20:0)+(o.force?24:0))return null;
  const T=ETYPES[type];const mi=minutes();const diff=1+mi*0.11+(G.day-1)*0.08+(P.lvl-1)*0.02;const quiet=T.passive||T.prop;
  const e={type,T,id:o.id||NET.eid++,net:!!o.net,pos:new V3(x,y,z),tpos:new V3(x,y,z),vel:new V3(),w:T.w,h:T.h,onGround:false,yaw:rand(0,TAU),
    hp:0,maxHp:0,atkT:rand(0.5,1.5),wind:0,slow:0,frozen:0,stun:0,burn:0,poison:0,flash:0,ph:rand(0,6),wanderT:0,wdir:new V3(),dead:false,alert:false,elite:!!T.elite,passive:!!T.passive,poi:o.poi||null,
    size:o.size||1,color:o.color||0x6fd46a,leapT:2,hopT:0,throwT:4,boss:!!o.boss,affix:o.affix||null,speedMul:1,fly:!!T.fly,rise:o.net||quiet||o.noRise||T.fly?0:0.7,hitP:0,sc:1,swell:0};
  if(type==='slime'){e.w=0.45*e.size;e.h=0.9*e.size;}
  const pc=1+0.6*(playerCount()-1);
  e.maxHp=e.hp=Math.round(T.hp*(quiet?1:diff*pc)*(type==='slime'?e.size:1)*(o.boss?1.5:1));e.dmg=(T.dmg||0)*(0.72+Math.min(0.28,mi*0.1)+mi*0.045+(G.day-1)*0.06);
  if(!quiet)e.speedMul=1+Math.min(0.3,mi*0.02);
  if(e.affix){e.elite=true;e.maxHp=e.hp=Math.round(e.hp*2.6);e.dmg*=1.3;const k=e.affix.k;if(k==='swift')e.speedMul*=1.5;if(k==='brute'){e.maxHp=e.hp=Math.round(e.hp*1.5);}}
  const shadow=e.elite||type==='brute'||type==='golem'||ents.length<45;
  const m=makeEnemyModel(type,{color:e.color,size:e.size,shadow});e.m=m;e.mats=m.mats;m.root.position.copy(e.pos);
  e.sc=type==='brute'?1.55:1;
  if(e.affix){const s=e.affix.k==='brute'?1.4:1.2;e.sc*=s;e.w*=s;e.h*=s;}
  m.root.scale.setScalar(e.sc);
  scene.add(m.root);
  if(e.rise>0){const g=_t1.set(x,y+0.1,z);DB.burst(g,[0x6a4a2c,0x8a5b34,0x4a3a2a],e.elite?10:4,3,0.18,5);if(Math.random()<0.3)fx.smoke(g,0x6a5a48,3,1,0.8,0.8);}
  if(!e.passive){const bar=document.createElement('div');bar.className='ebar'+(e.elite?' elite':'');bar.innerHTML=(e.affix?`<em>${e.affix.n} ${T.name}</em>`:'')+'<i></i>';bar.style.display='none';fxLayer.appendChild(bar);e.bar=bar;e.barI=bar.querySelector('i');}
  ents.push(e);return e;
}
function disposeModel(root){root.traverse(o=>{if(o.geometry&&o.geometry.userData.own)o.geometry.dispose();if(o.material&&o.material.dispose&&!o.material.userData.shared)o.material.dispose();});}
function removeEnt(e){scene.remove(e.m.root);disposeModel(e.m.root);if(e.bar){e.bar.remove();e.bar=null;}}
/* the fallen keel over and shrink away instead of vanishing */
let corpses=[];
function toCorpse(e){if(e.bar){e.bar.remove();e.bar=null;}if(corpses.length>=40){const c=corpses.shift();scene.remove(c.m.root);disposeModel(c.m.root);}
  corpses.push({m:e.m,t:0,sc:e.sc,side:Math.random()<0.5?-1:1,fly:e.fly});}
function updateCorpses(dt){for(let i=corpses.length-1;i>=0;i--){const c=corpses[i];c.t+=dt;const k=Math.min(1,c.t/0.45),r=c.m.root;
  r.rotation.z=c.side*k*1.3;r.rotation.x=-k*0.3;if(c.fly)r.position.y-=dt*6;const s=c.sc*(k<0.6?1:1-(k-0.6)/0.4*0.95);r.scale.set(s,s*(1-k*0.3),s);
  if(c.t>=0.45){scene.remove(r);disposeModel(r);corpses.splice(i,1);}}}
function clearCorpses(){for(const c of corpses){scene.remove(c.m.root);disposeModel(c.m.root);}corpses=[];}
function deathVisual(e){const c=_t3.set(e.pos.x,e.pos.y+e.h*0.5,e.pos.z);
  DB.burst(c,e.T.blood,e.elite?40:9,e.elite?8:5,e.elite?0.35:0.2,6);fx.smoke(c,0x4a4448,e.elite?12:3,1.2,0.9,e.elite?1.6:0.8);fx.burst(c,0xffffff,8,4,0.3,0.3);fx.sparks(c,e.T.blood[0],6,7);if(!e.passive)sfx(e.T.prop?'glass':'die');
  if(e.elite){if(ACTOR.local)G.shake=Math.min(1,G.shake+0.6);flashLight(c,e.affix?e.affix.c:0x6ff3ff,5,20,1);fx.ring(_t1.set(e.pos.x,e.pos.y,e.pos.z),e.affix?e.affix.c:0x6ff3ff,0.5,5,0.5);}}
function killEnt(e,o={}){
  e.dead=true;const c=new V3(e.pos.x,e.pos.y+e.h*0.5,e.pos.z);deathVisual(e);const prop=!!e.T.prop;
  if(!e.passive&&!prop){P.kills++;G.kills++;}augKill(e,c);
  const xp=o.noXp?0:Math.round(e.T.xp*(1+minutes()*0.05)*(e.affix?4:1)*(e.type==='slime'?e.size:1));
  if(xp&&!P.dead)dropGems(c,xp);
  if(e.passive){const k=(e.type==='chicken'?1:2)+(P.st&&P.st.food>1?1:0);for(let i=0;i<k;i++)dropFood(c,'meat');}
  else if(prop)urnLoot(c);
  else{if(Math.random()<0.04)dropFood(c,Math.random()<0.5?'meat':'berries');if(!P.dead&&!o.noXp)rareDrop(c,e.elite);}
  const nm=e.affix?e.affix.n+' '+e.T.name:e.T.name;
  if(NET.on&&NET.host)send({t:'kill',id:e.id,xp,x:r2(c.x),y:r2(c.y),z:r2(c.z),ty:e.type,el:e.elite?1:0,pa:e.passive?1:0,nm});
  if(e.type==='slime'&&e.size>1.1){for(let i=0;i<2;i++){const s=spawnEnt('slime',e.pos.x+rand(-0.5,0.5),e.pos.y+0.3,e.pos.z+rand(-0.5,0.5),{size:e.size*0.55,color:e.color,noRise:true,force:true});if(s){s.vel.set(rand(-4,4),6,rand(-4,4));s.alert=true;}}}
  if(e.affix&&e.affix.k==='splitter'){for(let i=0;i<3;i++){const s=spawnEnt(e.type==='golem'?'zombie':e.type,e.pos.x+rand(-1,1),e.pos.y+0.3,e.pos.z+rand(-1,1),{size:0.8,color:e.color,noRise:true,force:true});if(s){s.vel.set(rand(-5,5),6,rand(-5,5));s.alert=true;}}}
  if(e.affix&&e.affix.k==='volatile'){const p=e.pos.clone();efx('tele',{p:pk(p),c:0xffd35a,r:4,l:1});later(1,()=>{efx('boom',{p:pk(p),r:4});for(const t of targets())if(Math.hypot(t.pos.x-p.x,t.pos.z-p.z)<4)hurtTarget(t,22,p,{kb:8});});}
  if(e.type==='bloater')bloaterPop(e,o.self);
  if(e.elite){toast(nm+' slain','+'+xp+' experience',2.5);dropChestAt(e.pos.clone());}
  if(prop||e.type==='bloater')removeEnt(e);else toCorpse(e);
}
/* a bloater bursts in a cloud of bile: it always hurts nearby monsters (chain reactions!), and hurts players only when it detonates on its own */
function bloaterPop(e,self){const p=e.pos.clone();const r=3.8;efx('bloat',{p:pk(p),r});
  for(const q of ents){if(q===e||q.dead||q.passive||q.T.prop)continue;if(Math.hypot(q.pos.x-p.x,q.pos.z-p.z)<r+q.w)withActor(P,()=>applyDamage(q,Math.round(30*(1+minutes()*0.08)),{kb:7,src:p,up:5}));}
  if(self)for(const t of targets())if(Math.hypot(t.pos.x-p.x,t.pos.z-p.z)<r&&Math.abs(t.pos.y-p.y)<3)hurtTarget(t,e.dmg,p,{kb:9});}
function bloatFx(p,r){sfx('squelch');fx.burst(_t1.copy(p).setY(p.y+0.8),0xb8ff4a,50,r*2.4,0.6,0.6,{up:2,g:6});fx.ring(p,0x9aff4a,0.5,r,0.4);fx.shock(p,0x9aff4a,0.4,r,0.35,1.4);
  fx.smoke(p,0x6a8a3a,10,2.2,1.2,1.4);DB.burst(_t1.copy(p).setY(p.y+0.5),[0x9ab860,0x5a7a32,0xc8ff5a],14,6,0.22,6);flashLight(_t1,0xb8ff4a,3,r*4,0.35);if(P.pos.distanceTo(p)<20)G.shake=Math.min(1,G.shake+0.25);}
function volatileFx(p,r){sfx('boom');fx.burst(_t1.copy(p).setY(p.y+0.6),0xffd35a,60,r*2.5,0.6,0.6,{up:2});fx.ring(p,0xffd35a,0.5,r,0.4);fx.smoke(p,0x3a3330,10,2,1.2,1.4);flashLight(_t1,0xffd35a,4,r*4,0.4);G.shake=Math.min(1,G.shake+0.3);}

/* ---------------- pickups: experience gems, food, and rare treasures ---------------- */
const GEM=[{v:0,c:0x5ab8ff,s:0.17},{v:4,c:0x5aff8a,s:0.22},{v:12,c:0xff5a7a,s:0.3}];
function gemMesh(v){const g=v>=GEM[2].v?GEM[2]:v>=GEM[1].v?GEM[1]:GEM[0];const m=new THREE.Mesh(OCT,cm(g.c,true));m.scale.set(g.s/0.14,g.s/0.14*1.5,g.s/0.14);m.userData.c=g.c;return m;}
function dropGems(pos,xp){const n=Math.min(6,Math.max(1,Math.ceil(xp/9)));for(let i=0;i<n;i++)dropXp(pos,xp/n);}
function dropXp(pos,v){if(pickups.length>=CAP.pickups){const o=pickups.find(p=>p.kind==='xp');if(o){o.v+=v;return;}}
  const m=gemMesh(v);m.castShadow=false;scene.add(m);pickups.push({kind:'xp',v,mesh:m,pos:pos.clone(),vel:new V3(rand(-3,3),rand(3,6),rand(-3,3)),t:0,ph:rand(0,6)});}
function dropFood(pos,kind){if(pickups.length>=CAP.pickups)return;const g=new THREE.Group();if(kind==='meat'){mk(g,0.32,0.18,0.24,cm(0xd06a6a),0,0,0);mk(g,0.08,0.08,0.22,cm(0xf0e8d8),0,0,0.2);}
  else{for(let i=0;i<3;i++)mk(g,0.12,0.12,0.12,cm(0xd23a4e),rand(-0.1,0.1),rand(0,0.1),rand(-0.1,0.1));}
  scene.add(g);pickups.push({kind,mesh:g,pos:pos.clone(),vel:new V3(rand(-2,2),5,rand(-2,2)),t:0,ph:rand(0,6)});}
const SPECIAL={heart:{n:'Heart',c:0xff4a5a},magnet:{n:'Lodestone',c:0x7fd4ff},bomb:{n:'Holy Relic',c:0xfff0a0}};
function dropSpecial(pos,kind){const g=new THREE.Group();const S=SPECIAL[kind];
  if(kind==='heart'){const r=cm(0xe8344e),h=cm(0xff8a9a,true);mk(g,0.22,0.22,0.16,r,-0.1,0.06,0);mk(g,0.22,0.22,0.16,r,0.1,0.06,0);const t=mk(g,0.26,0.26,0.16,r,0,-0.07,0);t.rotation.z=Math.PI/4;mk(g,0.06,0.06,0.18,h,-0.12,0.12,0);}
  else if(kind==='magnet'){const r=cm(0xd8343a),s=cm(0xd8dde4);mk(g,0.12,0.36,0.12,r,-0.16,0,0);mk(g,0.12,0.36,0.12,r,0.16,0,0);mk(g,0.44,0.12,0.12,r,0,0.18,0);mk(g,0.12,0.1,0.13,s,-0.16,-0.21,0);mk(g,0.12,0.1,0.13,s,0.16,-0.21,0);}
  else{const gd=cm(0xe0b84a),w=cm(0xfff8d8,true);mk(g,0.1,0.5,0.1,gd,0,0,0);mk(g,0.36,0.1,0.1,gd,0,0.1,0);const c=mk(g,0.16,0.16,0.16,w,0,0.1,0);c.rotation.set(0.6,0.6,0);}
  g.children.forEach(q=>q.castShadow=false);scene.add(g);fx.pillar(pos.clone().setY(pos.y-0.4),S.c,0.4,4,1);pickups.push({kind,mesh:g,pos:pos.clone(),vel:new V3(rand(-2,2),5,rand(-2,2)),t:0,ph:rand(0,6),special:true});}
function rareDrop(c,elite){const r=Math.random();if(elite){if(r<0.5)dropSpecial(c,'heart');else if(r<0.7)dropSpecial(c,'magnet');return;}
  if(r<0.004)dropSpecial(c,'magnet');else if(r<0.011)dropSpecial(c,'heart');else if(r<0.014)dropSpecial(c,'bomb');}
function urnLoot(c){const r=Math.random();if(r<0.3)dropSpecial(c,'heart');else if(r<0.45)dropSpecial(c,'magnet');else if(r<0.6)dropSpecial(c,'bomb');else if(r<0.75)dropFood(c,'meat');else dropGems(c,Math.round(5+minutes()*2.5));
  if(Math.random()<0.5)dropGems(c,Math.round(2+minutes()));}
function collectSpecial(kind){const S=SPECIAL[kind];toast(S.n,kind==='heart'?'Restored health':kind==='magnet'?'Every gem flies to you':'Holy light scours the horde',1.6);
  if(kind==='heart'){healPlayer(P.maxHp*0.35);sfx('heal');healFx(P.pos);}
  else if(kind==='magnet'){sfx('relic');for(const p of pickups)if(p.kind==='xp')p.vac=true;fx.ring(P.pos,0x7fd4ff,0.5,14,0.6);}
  else{sfx('bomb');G.flash=0.22;const c=P.pos.clone();ACTOR=P;fx.ring(c,0xfff0a0,0.5,30,0.9,0.12,0.6);fx.shock(c,0xfff0a0,1,30,0.8,3);flashLight(_t1.set(c.x,c.y+3,c.z),0xfff0a0,4,60,0.8);G.shake=1;let n=0;
    for(const e of ents){if(e.dead||e.passive||e.T.prop)continue;if(Math.hypot(e.pos.x-c.x,e.pos.z-c.z)<30){const ep=e.pos.clone();damageEnt(e,Math.round(120*(1+minutes()*0.12)),{kb:6,src:c,up:4,col:0xfff0a0,quiet:true});
      if(n++<14&&ep.distanceTo(c)>4)fx.pillar(ep,0xfff0a0,0.45,7,0.45);else fx.sparks(_t1.set(ep.x,ep.y+1,ep.z),0xfff0a0,4,6);}}}}
function updatePickups(dt){
  for(let i=pickups.length-1;i>=0;i--){const p=pickups[i];p.t+=dt;const dx=P.pos.x-p.pos.x,dy=P.pos.y+0.9-p.pos.y,dz=P.pos.z-p.pos.z;const d=Math.hypot(dx,dy,dz);
    const mag=(p.kind==='xp'?5:p.special?3:2.4)*P.st.magnet;let grounded=false;
    if(((p.vac&&p.t>0.2)||(p.t>0.45&&d<mag))&&!P.dead){const sp=p.vac?16+p.t*4:7+p.t*10;p.vel.set(dx/d*sp,dy/d*sp,dz/d*sp);p.pos.addScaledVector(p.vel,dt);}
    else{p.vel.y-=18*dt;p.vel.x*=1-dt*2;p.vel.z*=1-dt*2;p.pos.addScaledVector(p.vel,dt);if(world.solid(p.pos.x,p.pos.y-0.1,p.pos.z)){p.pos.y=Math.floor(p.pos.y-0.1)+1.1;p.vel.y=Math.abs(p.vel.y)*0.35;p.vel.x*=0.5;p.vel.z*=0.5;grounded=true;}}
    p.mesh.position.copy(p.pos);if(grounded||p.vel.lengthSq()<1)p.mesh.position.y+=0.18+Math.sin(G.runT*3+p.ph)*0.08;p.mesh.rotation.y+=dt*(p.kind==='xp'?2:3);
    if(p.kind==='xp'&&Math.random()<0.04)PA.emit(p.pos.x,p.pos.y+0.2,p.pos.z,0,0.5,0,0.4,0.28,0.02,col(p.mesh.userData.c),0,0,0.8);
    else if(p.special&&Math.random()<0.3)PA.emit(p.pos.x+rand(-0.3,0.3),p.pos.y+0.2,p.pos.z+rand(-0.3,0.3),0,0.9,0,0.6,0.3,0.03,col(SPECIAL[p.kind].c),0,0,0.9);
    if(d<1&&p.t>0.3&&!P.dead){scene.remove(p.mesh);pickups.splice(i,1);
      if(p.kind==='xp'){sfx('gem');gainXp(p.v);}
      else if(p.special)collectSpecial(p.kind);
      else{P.food[p.kind==='meat'?'meat':'berries']+=1;sfx('pick');popText(_t1.set(P.pos.x,P.pos.y+2,P.pos.z),p.kind==='meat'?'+ Meat':'+ Berries','heal');buildActionBar();}
      continue;}
    if(p.t>(p.kind==='xp'?150:90)){scene.remove(p.mesh);pickups.splice(i,1);}}
}
function animEnt(e,dt,moving){
  const m=e.m;e.ph+=dt*(moving?9:2);const s=moving?Math.sin(e.ph):0;
  if(m.legL&&m.armL){m.legL.rotation.x=s*0.7;m.legR.rotation.x=-s*0.7;m.torso.rotation.z=s*0.05;
    if(e.type==='zombie'){m.armL.rotation.x=-Math.PI/2+Math.sin(e.ph*0.5)*0.15;m.armR.rotation.x=-Math.PI/2-Math.sin(e.ph*0.5)*0.15;m.head.rotation.z=Math.sin(e.ph*0.3)*0.15;
      if(e.wind>0){m.armL.rotation.x=m.armR.rotation.x=-2.6;m.torso.rotation.x=-0.2;}else m.torso.rotation.x=0.15;}
    else if(e.type==='skeleton'){m.armL.rotation.x=e.aiming?-Math.PI/2:-s*0.5;m.armR.rotation.x=e.aiming?-Math.PI/2:s*0.5;m.armR.rotation.y=e.aiming?-0.5:0;}
    else if(e.type==='cultist'){if(e.wind>0){m.armL.rotation.x=m.armR.rotation.x=-2.7;m.armL.rotation.z=0.4;m.armR.rotation.z=-0.4;}else{m.armL.rotation.x=-s*0.4;m.armR.rotation.x=s*0.4;m.armL.rotation.z=m.armR.rotation.z=0;}}
    else if(e.type==='bloater'){m.armL.rotation.x=-s*0.3-0.3;m.armR.rotation.x=s*0.3-0.3;m.armL.rotation.z=0.5;m.armR.rotation.z=-0.5;const sw=1+e.swell*0.55+Math.sin(e.ph*0.7)*0.04;m.belly.scale.setScalar(sw);m.torso.rotation.x=-0.1;}
    else if(e.type==='brute'){m.armL.rotation.x=-s*0.5;m.armR.rotation.x=e.wind>0?-2.9+(1-e.wind/e.T.wind)*0.4:s*0.5-0.3;m.torso.rotation.x=e.wind>0?-0.3:0.15;m.body.position.y=Math.abs(s)*0.06;}
    else if(e.type==='golem'){m.armL.rotation.x=-s*0.4;m.armR.rotation.x=s*0.4;if(e.wind>0){m.armL.rotation.x=m.armR.rotation.x=-2.8;}m.body.position.y=Math.abs(s)*0.08;}}
  if(m.wingL){const f=Math.sin(e.ph*2.6);m.wingL.rotation.z=f*0.9;m.wingR.rotation.z=-f*0.9;m.body.position.y=0.3+f*0.06;m.body.rotation.x=0.2;}
  if(m.legs){if(e.type==='spider'){m.legs.forEach((l,i)=>{l.rotation.y=Math.sin(e.ph*1.5+i)*0.4*(moving?1:0.2);});}else m.legs.forEach((l,i)=>{l.rotation.x=Math.sin(e.ph+(i%2?Math.PI:0)+(i>1?Math.PI:0))*0.6*(moving?1:0);});
    if(m.head&&e.passive&&!moving)m.head.rotation.x=Math.max(0,Math.sin(e.ph*0.4))*0.5;}
  if(e.type==='slime'){const sq=e.onGround?1-Math.max(0,e.squash||0):1;m.body.scale.set(1+(1-sq)*0.5,sq*(e.onGround?1:1.15),1+(1-sq)*0.5);e.squash=Math.max(0,(e.squash||0)-dt*3);}
  if(e.type==='urn'){m.body.rotation.y+=dt*0.3;}
}
function targets(){const t=[];if(G.state==='playing'&&!P.dead&&P.model)t.push(P);for(const r of NET.peers.values())if(r.inGame&&!r.dead)t.push(r);for(const d of decoys)if(!d.dead)t.push(d);return t;}
function hurtTarget(t,amt,src,opts={}){if(t===P)hurtPlayer(amt,src,opts);else if(t.isDecoy){t.hp-=amt;fx.burst(_t1.set(t.pos.x,t.pos.y+1,t.pos.z),0x9be07a,6,3,0.3,0.2);}
  else if(NET.on)send({t:'hurt',to:t.id,a:Math.round(amt),s:src?[r2(src.x),r2(src.z)]:null,kb:opts.kb});}
function entVisual(e,dt,moving){const m=e.m;m.root.position.copy(e.pos);let dyaw=((e.yaw-m.root.rotation.y)%TAU+TAU*1.5)%TAU-Math.PI;m.root.rotation.y+=dyaw*Math.min(1,dt*10);animEnt(e,dt,moving);
  // rising out of the ground, and a squash-and-stretch kick whenever something lands a hit
  if(e.rise>0){const k=smooth(0,0.7,e.rise);m.root.position.y-=k*(e.h+0.1);if(Math.random()<0.5)PN.emit(e.pos.x+rand(-0.5,0.5),e.pos.y+0.1,e.pos.z+rand(-0.5,0.5),rand(-1,1),rand(1,2.5),rand(-1,1),0.5,0.25,0.4,col(0x7a5a3a),8,1,0.9);}
  const hp=e.hitP>0?e.hitP/0.14:0;m.root.scale.set(e.sc*(1+hp*0.18),e.sc*(1-hp*0.16),e.sc*(1+hp*0.18));
  const near=e.pos.distanceTo(camera.position)<55;
  if(near&&(e.burn>0||e.burnFlag)&&Math.random()<0.7)PA.emit(e.pos.x+rand(-e.w,e.w),e.pos.y+rand(0,e.h),e.pos.z+rand(-e.w,e.w),0,2,0,0.4,0.4,0.05,col(e.poison>0?pick([0x7aff5a,0x3ac83a]):pick([0xff6a1a,0xffb03a])),-1,0,1);
  if(near&&e.elite&&Math.random()<0.4)PA.emit(e.pos.x+rand(-1,1),e.pos.y+rand(0.5,e.h),e.pos.z+rand(-1,1),0,1,0,0.6,0.3,0.05,col(e.affix?e.affix.c:0x6ff3ff),0,0,0.8);
  if(near&&e.stun>0&&Math.random()<0.2)PA.emit(e.pos.x+Math.cos(e.ph*4)*0.4,e.pos.y+e.h+0.25,e.pos.z+Math.sin(e.ph*4)*0.4,0,0,0,0.3,0.25,0.05,col(0xffe48a),0,0,1);
  if(near&&e.type==='bloater'&&Math.random()<0.15)PN.emit(e.pos.x+rand(-0.3,0.3),e.pos.y+1.3,e.pos.z+rand(-0.3,0.3),0,0.6,0,1,0.3,0.6,col(0x8ab84a),-0.3,0.5,0.5);
  const frozen=e.frozen>0||e.frozenFlag;const em=e.flash>0?0xffffff:frozen?0x2a6a8a:e.swell>0.5&&Math.sin(G.runT*30)>0?0x405a10:(e.elite?(e.affix?e.affix.c&0x3f3f3f:0x0a1418):0);
  for(const mt of e.mats)if(mt.emissive)mt.emissive.setHex(em);}
/* relocate a straggler that fell far behind to the edge of the fight, so the horde keeps pressing in */
function recycleEnt(e){const tg=avatars().filter(a=>!a.dead);if(!tg.length)return false;const s=ringSpot(pick(tg),26,34);if(!s)return false;
  e.pos.copy(s);e.tpos.copy(s);e.vel.set(0,0,0);e.rise=e.fly?0:0.7;e.wind=0;e.leaping=0;return true;}
function updateEnts(dt){
  const tg=targets(),av=avatars();
  for(let i=ents.length-1;i>=0;i--){const e=ents[i];if(e.dead){ents.splice(i,1);continue;}
    e.flash-=dt;if(e.hitP>0)e.hitP=Math.max(0,e.hitP-dt);
    if(e.net){if(e.rise>0)e.rise-=dt;const px=e.pos.x,pz=e.pos.z;if(e.pos.distanceTo(e.tpos)>8)e.pos.copy(e.tpos);else e.pos.lerp(e.tpos,Math.min(1,dt*10));const mv=Math.hypot(e.pos.x-px,e.pos.z-pz)>dt*0.6;e.onGround=true;entVisual(e,dt,mv);continue;}
    let near=1e9;for(const a of av){const d=Math.hypot(a.pos.x-e.pos.x,a.pos.z-e.pos.z);if(d<near)near=d;}
    if(e.T.prop){if(near>90){removeEnt(e);ents.splice(i,1);continue;}moveEnt(e,dt);entVisual(e,dt,false);continue;}
    if(e.passive&&near>96){removeEnt(e);ents.splice(i,1);continue;}
    if(e.poi&&near>120){removeEnt(e);ents.splice(i,1);e.poi.guarded=false;continue;}
    if(!e.passive&&!e.poi&&near>(e.boss?95:60)&&!recycleEnt(e)){if(near>130){removeEnt(e);ents.splice(i,1);continue;}}
    if(e.rise>0){e.rise-=dt;e.vel.set(0,Math.min(e.vel.y,0),0);if(!e.fly)moveEnt(e,dt);entVisual(e,dt,false);continue;}
    let tgt=null,dist=1e9;for(const t of tg){const d=Math.hypot(t.pos.x-e.pos.x,t.pos.z-e.pos.z)*(t.isDecoy?0.5:1)*(t===e.aggroOn?0.7:1);if(d<dist){dist=d;tgt=t;}}
    const tp=tgt?tgt.pos:e.pos;const dx=tp.x-e.pos.x,dz=tp.z-e.pos.z;dist=Math.max(0.001,Math.hypot(dx,dz));const dy=tp.y-e.pos.y;
    e.slow-=dt;e.stun-=dt;e.fear=(e.fear||0)-dt;e.atkT-=dt;e.leapT-=dt;e.hopT-=dt;e.throwT-=dt;
    if(e.frozen>0)e.frozen-=dt;
    if(e.burn>0||e.poison>0){e.burn-=dt;e.poison-=dt;e.burnAcc=(e.burnAcc||0)+dt;if(e.burnAcc>0.5){e.burnAcc=0;const bd=Math.round((e.poison>0?3:4)*(1+minutes()*0.06));withActor(P,()=>applyDamage(e,bd,{}));if(Math.random()<0.5)popText(_t1.set(e.pos.x,e.pos.y+e.h,e.pos.z),String(bd),'');if(e.dead)continue;}}
    if(e.affix&&e.affix.k==='vampiric'&&e.hp<e.maxHp)e.hp=Math.min(e.maxHp,e.hp+dt*e.maxHp*0.01);
    let mvx=0,mvz=0,speed=e.T.speed*e.speedMul*(e.slow>0?0.55:1);const canMove=e.frozen<=0&&e.stun<=0&&tgt;
    // the horde always hunts; only camp and ruin guards hold their ground until you come close
    const aggro=!e.passive&&tgt&&(!e.poi||dist<26||e.alert);
    const hurt=(amt,o)=>{hurtTarget(tgt,amt,e.pos,o);if(e.affix&&e.affix.k==='vampiric'){e.hp=Math.min(e.maxHp,e.hp+amt*1.5);}};
    if(e.passive){e.wanderT-=dt;if(e.flee>0&&tgt){e.flee-=dt;mvx=-dx/dist;mvz=-dz/dist;speed*=2.1;}else{if(e.wanderT<=0){e.wanderT=rand(2,6);const an=rand(0,TAU);e.wdir.set(Math.random()<0.4?0:Math.cos(an),0,Math.sin(an));}mvx=e.wdir.x;mvz=e.wdir.z;}}
    else if(aggro&&canMove){
      const tdx=dx/dist,tdz=dz/dist;
      if(e.fear>0){mvx=-tdx;mvz=-tdz;}
      else if(e.T.ranged){const want=e.type==='skeleton'?10:9;if(dist>want+3){mvx=tdx;mvz=tdz;}else if(dist<want-4){mvx=-tdx;mvz=-tdz;}else{const sg=Math.sign(Math.sin(e.ph*0.2))||1;mvx=-tdz*0.6*sg;mvz=tdx*0.6*sg;}
        e.aiming=dist<e.T.range+4;
        if(e.wind>0){e.wind-=dt;mvx*=0.2;mvz*=0.2;if(e.type==='cultist'){const hp=_t1.set(e.pos.x,e.pos.y+2.2,e.pos.z);PA.emit(hp.x+rand(-0.4,0.4),hp.y+rand(-0.2,0.2),hp.z+rand(-0.4,0.4),0,1,0,0.4,0.35,0.05,col(0xd07aff),0,0,1);}
          if(e.wind<=0)enemyShoot(e,tgt);}
        else if(e.atkT<=0&&dist<e.T.range){e.wind=e.T.wind;e.atkT=e.T.cd*rand(0.8,1.2)/e.speedMul;}}
      else if(e.fly){mvx=tdx;mvz=tdz;const ty=tp.y+1.1+Math.sin(e.ph*0.9)*0.5;e.vel.y=lerp(e.vel.y,clamp((ty-e.pos.y)*3,-6,6),Math.min(1,dt*4));
        if(dist<0.9+e.w&&Math.abs(tp.y+1-e.pos.y)<1.5&&e.atkT<=0){e.atkT=e.T.cd;hurt(e.dmg);e.vel.x-=tdx*9;e.vel.z-=tdz*9;e.vel.y=4;}}
      else if(e.type==='slime'){mvx=tdx;mvz=tdz;if(dist<e.w+0.7&&Math.abs(dy)<1.5&&e.atkT<=0){e.atkT=0.9;hurt(e.dmg*e.size);}}
      else if(e.type==='bloater'){mvx=tdx;mvz=tdz;if(dist<9)speed*=1.7;
        if(e.wind>0){e.wind-=dt;mvx=mvz=0;e.swell=1-e.wind/e.T.wind;if(e.wind<=0){killEnt(e,{noXp:true,self:true});continue;}}
        else if(dist<2.3&&Math.abs(dy)<2){e.wind=e.T.wind;efx('tele',{p:pk(e.pos),c:0x9aff4a,r:3.8,l:e.T.wind});sfx('swell');}}
      else if(e.type==='brute'){mvx=tdx;mvz=tdz;
        if(e.wind>0){e.wind-=dt;mvx=mvz=0;if(e.wind<=0)bruteSmash(e);}
        else if(dist<e.T.reach+0.6&&e.atkT<=0&&Math.abs(dy)<2.5){e.wind=e.T.wind;e.atkT=e.T.cd/e.speedMul;e.smashAt=new V3(e.pos.x+tdx*1.8,e.pos.y,e.pos.z+tdz*1.8);efx('tele',{p:pk(e.smashAt),c:0xff7a3a,r:2.8,l:e.T.wind});}}
      else if(e.type==='golem'){mvx=tdx;mvz=tdz;
        if(e.wind>0){e.wind-=dt;mvx=mvz=0;if(e.wind<=0)golemSlam(e);}
        else if(dist<e.T.reach+1.5&&e.atkT<=0){e.wind=e.T.wind;e.atkT=e.T.cd;efx('tele',{p:pk(e.pos),c:0xff6a3a,r:5.2,l:e.T.wind});}
        else if(dist>10&&e.throwT<=0&&dist<34){e.throwT=rand(4,6);golemThrow(e,tgt);}}
      else{mvx=tdx;mvz=tdz;
        if(e.type==='spider'&&dist<7&&dist>2.2&&e.leapT<=0&&e.onGround){e.leapT=rand(2.5,4);e.vel.set(tdx*13,7.5,tdz*13);e.leaping=0.8;}
        if(e.leaping>0){e.leaping-=dt;if(dist<1.4&&Math.abs(dy)<1.6){e.leaping=0;hurt(e.dmg);}}
        if(e.wind>0){e.wind-=dt;mvx*=0.25;mvz*=0.25;if(e.wind<=0){if(dist<e.T.reach+0.7&&Math.abs(dy)<2)hurt(e.dmg);efx('arc',{p:[r2(e.pos.x),r2(e.pos.y+1),r2(e.pos.z)],y:r2(Math.atan2(dx,dz)),r:e.T.reach+0.5});}}
        else if(dist<e.T.reach+0.3&&e.atkT<=0&&Math.abs(dy)<2){e.wind=e.T.wind;e.atkT=e.T.cd/e.speedMul;}}
      e.yaw=Math.atan2(dx,dz);
    }else if(!e.passive&&e.poi){e.wanderT-=dt;if(e.wanderT<=0){e.wanderT=rand(2,5);const an=rand(0,TAU);e.wdir.set(Math.random()<0.5?0:Math.cos(an),0,Math.sin(an));}mvx=e.wdir.x*0.5;mvz=e.wdir.z*0.5;}
    if(e.frozen>0||e.stun>0){mvx=mvz=0;if(e.fly)e.vel.y=Math.max(e.vel.y-dt*12,-6);}
    if((mvx||mvz)&&(e.passive||!aggro||e.fear>0))e.yaw=Math.atan2(mvx,mvz);
    const moving=!!(mvx||mvz);
    if(e.type==='slime'){if(e.onGround){e.vel.x*=1-Math.min(1,dt*8);e.vel.z*=1-Math.min(1,dt*8);if(e.hopT<=0&&moving){e.hopT=rand(0.7,1.1);e.vel.set(mvx*speed*1.6,7.5,mvz*speed*1.6);e.squash=0.4;}}}
    else if(!(e.leaping>0)){const acc=e.onGround||e.fly?10:2;e.vel.x=lerp(e.vel.x,mvx*speed,Math.min(1,acc*dt));e.vel.z=lerp(e.vel.z,mvz*speed,Math.min(1,acc*dt));}
    if(e.hitWall&&moving){if(e.fly)e.vel.y=7;else if(e.onGround&&e.type!=='slime')e.vel.y=8.6;}
    if(e.inWater&&!e.onGround&&!e.fly)e.vel.y=Math.max(e.vel.y,2);
    moveEnt(e,dt);
    if(e.type==='slime'&&e.onGround&&e.wasAir){e.squash=0.45;}e.wasAir=!e.onGround;
    entVisual(e,dt,moving&&(e.onGround||e.fly));
  }
  // keep the horde from stacking into one blob
  if(ents.length<240)for(let i=0;i<ents.length;i++){const a=ents[i];if(a.net||a.T.prop)continue;for(let j=i+1;j<ents.length;j++){const b=ents[j];if(b.T.prop)continue;const dx=b.pos.x-a.pos.x,dz=b.pos.z-a.pos.z,r=a.w+b.w+0.1;if(dx>r||dx<-r||dz>r||dz<-r)continue;const d=Math.hypot(dx,dz);
    if(d<r&&d>0.001&&Math.abs(a.pos.y-b.pos.y)<1.5){const push=(r-d)*0.5,nx=dx/d*push,nz=dz/d*push;if(!collides(a.pos.x-nx,a.pos.y,a.pos.z-nz,a.w,a.h)){a.pos.x-=nx;a.pos.z-=nz;}if(!collides(b.pos.x+nx,b.pos.y,b.pos.z+nz,b.w,b.h)){b.pos.x+=nx;b.pos.z+=nz;}}}}
  for(let i=decoys.length-1;i>=0;i--){const d=decoys[i];d.t-=dt;if(d.t<=0||d.hp<=0){fx.smoke(d.pos,0xd8d0c0,8,1.5,0.8,0.9);scene.remove(d.model.root);decoys.splice(i,1);}}
}
function bruteSmash(e){const c=e.smashAt||e.pos.clone();c.y=groundAt(c.x,c.z,e.pos.y+1);efx('slam',{p:pk(c),r:2.8});for(const t of targets())if(Math.hypot(t.pos.x-c.x,t.pos.z-c.z)<2.9&&Math.abs(t.pos.y-c.y)<2.5)hurtTarget(t,e.dmg,c,{kb:10});}
function eprojDef(kind){
  if(kind==='arrow')return{mesh:arrowMesh(0xc8c2b0,0x5a5a5a),grav:9,r:0.3,trail:p=>{if(Math.random()<0.4)PA.emit(p.pos.x,p.pos.y,p.pos.z,0,0,0,0.25,0.2,0.03,col(0xff8a6a),0,0,0.6);},onWorld:stickArrow};
  if(kind==='dark')return{mesh:glowBall(0xb04aff,0.35),grav:0,r:0.45,homing:1.6,trail:p=>PA.emit(p.pos.x,p.pos.y,p.pos.z,rand(-0.5,0.5),rand(-0.5,0.5),rand(-0.5,0.5),0.5,0.5,0.05,col(pick([0xb04aff,0x5a1a8a])),0,1,1),onWorld:p=>fx.burst(p.pos,0xb04aff,14,3,0.35,0.3)};
  const rk=new THREE.Mesh(ROCK,cm(0x7d8088));rk.castShadow=true;
  return{mesh:rk,grav:14,r:0.8,trail:p=>{p.mesh.rotation.x+=0.2;if(Math.random()<0.5)PN.emit(p.pos.x,p.pos.y,p.pos.z,0,0,0,0.6,0.4,0.9,col(0x8a8478),0,0,0.5);},onWorld:p=>rockBreakFx(p.pos)};
}
function enemyShoot(e,tgt){
  const from=new V3(e.pos.x,e.pos.y+1.5,e.pos.z);const tp=new V3(tgt.pos.x,tgt.pos.y+1,tgt.pos.z);let v,kind;
  if(e.type==='skeleton'){sfx('bow');const sp=30;const t=from.distanceTo(tp)/sp;if(tgt.vel){tp.x+=tgt.vel.x*t*0.7;tp.z+=tgt.vel.z*t*0.7;}v=tp.clone().sub(from).normalize().multiplyScalar(sp);v.y+=0.5*9*t;kind='arrow';}
  else{sfx('arcane');v=tp.clone().sub(from).normalize().multiplyScalar(10);kind='dark';}
  fireEnemyProj(e,kind,from,v,tgt);}
function fireEnemyProj(e,kind,from,v,tgt){const d=eprojDef(kind);const ow=d.onWorld;
  spawnProj(Object.assign(d,{pos:from,vel:v,life:kind==='dark'?5:4,owner:'e',dmg:e.dmg*(kind==='rock'?0.8:1),kind,tgt,src:e,
    onHit:(t,p)=>{hurtTarget(t,p.dmg,p.pos,{kb:kind==='rock'?8:kind==='dark'?4:2});if(kind==='dark')fx.burst(p.pos,0xb04aff,20,4,0.4,0.35);if(kind==='rock')rockBreakFx(p.pos);},
    onWorld:p=>{ow(p);if(kind==='rock')for(const t of targets())if(t.pos.distanceTo(p.pos)<2.5)hurtTarget(t,10,p.pos,{kb:5});}}));
  if(NET.on&&NET.host)send({t:'eproj',k:kind,p:pk(from),v:pk(v),tg:tgt&&tgt.id&&!tgt.isDecoy?tgt.id:0});}
function visualEnemyProj(m){const d=eprojDef(m.k);const tgt=m.tg===P.id?P:NET.peers.get(m.tg);spawnProj(Object.assign(d,{pos:new V3().fromArray(m.p),vel:new V3().fromArray(m.v),life:5,owner:'v',kind:m.k,tgt}));}
function golemSlamFx(c,r=5.5){sfx('boom');G.shake=Math.min(1,G.shake+(r>4?0.6:0.3));fx.ring(c,0xffa070,0.5,r,0.5);fx.shock(c,0xffa070,0.5,r,0.4,1.6);fx.smoke(c,0x8a8478,r>4?14:7,3,1.2,1.4);
  const cols=[0x7d8088,0x5c5f66,0x8a5b34];for(let i=0;i<20;i++){const an=rand(0,TAU),s=rand(4,9);DB.emit(c.x+Math.cos(an)*1.5,c.y+0.3,c.z+Math.sin(an)*1.5,pick(cols),rand(0.25,0.5),Math.cos(an)*s,rand(5,9),Math.sin(an)*s,1.5);}}
function golemSlam(e){const c=e.pos.clone();efx('slam',{p:pk(c)});for(const t of targets())if(Math.hypot(t.pos.x-c.x,t.pos.z-c.z)<5.4&&t.pos.y-c.y<2.5)hurtTarget(t,e.dmg,c,{kb:12});}
function golemThrow(e,tgt){const from=new V3(e.pos.x,e.pos.y+4,e.pos.z);const tp=new V3(tgt.pos.x,tgt.pos.y+0.5,tgt.pos.z);const sp=22,t=from.distanceTo(tp)/sp;const v=tp.clone().sub(from).normalize().multiplyScalar(sp);v.y+=0.5*14*t;
  sfx('heavy');fireEnemyProj(e,'rock',from,v,tgt);}
function rockBreakFx(p){sfx('heavy');DB.burst(p,[0x7d8088,0x5c5f66],10,6,0.3,6);fx.smoke(p,0x8a8478,6,2,1,1);if(P.pos.distanceTo(p)<12)G.shake=Math.min(1,G.shake+0.2);}

function updateProjs(dt){
  for(let i=projs.length-1;i>=0;i--){const p=projs[i];if(!p)continue;ACTOR=p.a||P;p.age+=dt;let end=false;
    if(p.homing){let tgt=null;if(p.owner!=='p'){if(p.tgt)tgt=_t1.set(p.tgt.pos.x,p.tgt.pos.y+1,p.tgt.pos.z);}else{let bd=p.seek||12;for(const e of ents){if(e.dead||e.passive||e.T.prop)continue;const d=dist3(e,p.pos);if(d<bd){bd=d;tgt=_t1.set(e.pos.x,e.pos.y+e.h*0.5,e.pos.z);}}}
      if(tgt){const sp=p.vel.length();_t2.copy(tgt).sub(p.pos).normalize().multiplyScalar(sp);p.vel.lerp(_t2,Math.min(1,p.homing*dt)).setLength(sp);}}
    // boomerangs fly out, then turn and home back to their thrower, able to hit everything again
    if(p.boomer){const o=p.boomer;if(!p.back&&p.age>p.turn){p.back=true;p.hit.clear();}
      if(p.back){_t2.set(o.pos.x-p.pos.x,o.pos.y+1.2-p.pos.y,o.pos.z-p.pos.z);const d=_t2.length();if(d<1.3||o.dead)end=true;else p.vel.lerp(_t2.multiplyScalar(Math.max(16,p.speed||18)/d),Math.min(1,dt*7));}
      else p.vel.multiplyScalar(1-dt*1.2);}
    const sp=p.vel.length()*dt;const n=Math.max(1,Math.ceil(sp/0.3));
    for(let s=0;s<n&&!end;s++){p.vel.y-=p.grav*dt/n;p.pos.addScaledVector(p.vel,dt/n);
      if(!p.noWorld&&world.solid(p.pos.x,p.pos.y,p.pos.z)){p.pos.addScaledVector(p.vel,-dt/n*0.5);p.onWorld&&p.onWorld(p);end=true;break;}
      if(p.owner==='p'){for(const e of ents){if(e.dead||p.hit.has(e))continue;const hx=Math.hypot(e.pos.x-p.pos.x,e.pos.z-p.pos.z);
          if(hx<e.w+p.r+0.1&&p.pos.y>e.pos.y-p.r&&p.pos.y<e.pos.y+e.h+p.r){p.hit.add(e);p.onHit&&p.onHit(e,p);if(!p.pierce){if(p.bnc>0&&augBounce(p,e))p.bnc--;else{end=true;break;}}}}}
      else if(p.owner==='e'){for(const t of targets()){const hx=Math.hypot(t.pos.x-p.pos.x,t.pos.z-p.pos.z);if(hx<t.w+p.r&&p.pos.y>t.pos.y-p.r&&p.pos.y<t.pos.y+t.h+p.r){p.onHit&&p.onHit(t,p);end=true;break;}}}
      else if(p.owner==='v'){for(const t of avatars()){const hx=Math.hypot(t.pos.x-p.pos.x,t.pos.z-p.pos.z);if(hx<t.w+p.r&&p.pos.y>t.pos.y-p.r&&p.pos.y<t.pos.y+t.h+p.r){if(p.kind==='rock')rockBreakFx(p.pos);else fx.burst(p.pos,p.kind==='dark'?0xb04aff:0xff8a6a,10,3,0.3,0.25);end=true;break;}}}}
    if(!end&&p.age>p.life){p.onExpire&&p.onExpire(p);end=true;}
    if(p.mesh){p.mesh.position.copy(p.pos);if(p.kind==='arrow'||p.kind==='ice')p.mesh.lookAt(_t2.copy(p.pos).add(p.vel));}
    if(!end&&p.trail)p.trail(p);
    if(end){if(p.mesh&&!p.keepMesh)scene.remove(p.mesh);const k=projs.indexOf(p);if(k>=0)projs.splice(k,1);}}
  ACTOR=P;
  for(let i=stuck.length-1;i>=0;i--){stuck[i].t-=dt;if(stuck[i].t<=0){scene.remove(stuck[i].m);stuck.splice(i,1);}}
}
function updateZones(dt){
  for(let i=zones.length-1;i>=0;i--){const z=zones[i];if(!z)continue;ACTOR=z.a||P;z.t+=dt;if(z.follow)z.pos=z.follow.pos;if(z.onFrame)z.onFrame(z,dt);
    if(z.every){z.tick-=dt;if(z.tick<=0){z.tick+=z.every;z.onTick(z);}}
    if(z.t>=z.life){const k=zones.indexOf(z);if(k>=0)zones.splice(k,1);z.end&&z.end();}}
  for(let i=traps.length-1;i>=0;i--){const t=traps[i];if(!t)continue;ACTOR=t.a||P;t.life-=dt;t.armed-=dt;if(t.life<=0){scene.remove(t.mesh);traps.splice(i,1);continue;}
    if(t.armed>0)continue;if(Math.random()<0.05)PA.emit(t.pos.x,t.pos.y+0.2,t.pos.z,0,0.6,0,0.6,0.2,0.05,col(0xd9b46a),0,0,0.6);
    for(const e of ents){if(e.dead||e.passive||e.T.prop||e.fly)continue;if(Math.hypot(e.pos.x-t.pos.x,e.pos.z-t.pos.z)<(t.mini?0.9:1.3)+e.w&&Math.abs(e.pos.y-t.pos.y)<1.5){
      fx.flare(_t1.copy(t.pos).setY(t.pos.y+0.4),0xffe0a0,t.mini?2:3.5,0.25,true);fx.spikes(t.pos,0x9aa0a8,t.mini?5:8,t.mini?0.35:0.55,0.5,1.1,cm(0x9aa0a8));
      sfx('crit');damageEnt(e,Math.round(32*(1+0.6*t.spiked)*(t.mini?0.6:1)),{kb:0,col:0xd9b46a});status(e,{stun:(3+t.spiked)*(t.mini?0.5:1)});fx.burst(_t1.copy(t.pos).setY(t.pos.y+0.4),0xd9b46a,24,5,0.4,0.3);fx.sparks(_t1,0xffe0a0,8,8);
      if(t.boom){const c=t.pos.clone();explodeFire(c.setY(c.y+0.5),t.mini?2.5:4,t.mini?20:40,0);}
      if(t.field){const tp=t.pos.clone(),ta=t.a;later(0.05,()=>withActor(ta,()=>{for(let k=0;k<4;k++){const an=k/4*TAU+rand(-0.4,0.4),d=rand(2,3.5);const to=new V3(tp.x+Math.cos(an)*d,0,tp.z+Math.sin(an)*d);to.y=groundAt(to.x,to.z,tp.y+2);makeTrap(ta,to,true);}}));}
      DB.burst(t.pos,[0x9aa0a8,0x4a3a2a],8,4,0.14,5);scene.remove(t.mesh);traps.splice(i,1);break;}}}
  for(let i=timers.length-1;i>=0;i--){const tm=timers[i];if(!tm)continue;tm.t-=dt;if(tm.t<=0){timers.splice(timers.indexOf(tm),1);ACTOR=tm.a||P;try{tm.fn();}catch(err){console.error(err);}}}
  ACTOR=P;
}

/* ---------------- spawning & points of interest ---------------- */
function surfaceSpot(x,z){const fx_=Math.floor(x),fz=Math.floor(z);const t=world.topAt(fx_,fz);if(t<=SEA||t>=CH-3)return null;const b=world.get(fx_,t,fz);
  if(b===B.LEAVES||b===B.PINE||b===B.SPINE||b===B.AUTUMN||b===B.CACTUS)return null;if(world.get(fx_,t+1,fz)||world.get(fx_,t+2,fz))return null;return new V3(fx_+0.5,t+1,fz+0.5);}
function chestModel(){const g=new THREE.Group();const w=cm(0x7a4e2a),gd=cm(0xe0b84a),dk=cm(0x3a2618);
  mk(g,0.9,0.5,0.6,w,0,0.25,0);mk(g,0.94,0.06,0.64,gd,0,0.08,0);mk(g,0.94,0.06,0.64,gd,0,0.46,0);const lid=pivot(g,0,0.5,-0.3);mk(lid,0.92,0.22,0.62,w,0,0.11,0.3);mk(lid,0.14,0.18,0.06,gd,0,0.05,0.62);
  mk(g,0.06,0.5,0.64,dk,-0.3,0.25,0);mk(g,0.06,0.5,0.64,dk,0.3,0.25,0);g.userData.lid=lid;return g;}
function dropChestAt(pos,o={}){const m=chestModel();const y=groundAt(pos.x,pos.z,pos.y+1);m.position.set(pos.x,y,pos.z);scene.add(m);chests.push({mesh:m,pos:new V3(pos.x,y,pos.z),open:false,t:0,relic:!!o.relic});
  fx.pillar(m.position.clone(),0xffd35a,0.8,10,1.2);}
function updatePOIs(dt){
  const gx0=Math.floor(P.pos.x/64),gz0=Math.floor(P.pos.z/64);let nearFire=null,nd=1e9;
  for(let gz=gz0-1;gz<=gz0+1;gz++)for(let gx=gx0-1;gx<=gx0+1;gx++){const p=world.poi(gx,gz);if(!p)continue;const d=Math.hypot(P.pos.x-p.x,P.pos.z-p.z);
    if(p.type==='camp'&&d<nd){nd=d;nearFire=p;}
    if(p.type==='shrine'&&!p.used&&d<60&&Math.random()<0.5){const an=rand(0,TAU);PA.emit(p.x+0.5+Math.cos(an)*0.6,p.y+2.6,p.z+0.5+Math.sin(an)*0.6,0,rand(0.5,1.5),0,1.2,0.4,0.05,col(0xb98aff),0,0,1);}
    if(p.type==='altar'&&!p.used&&d<60&&Math.random()<0.6){const k=randi(0,3),cx=p.x+0.5+(k&1?4:-4),cz=p.z+0.5+(k&2?4:-4);PA.emit(cx+rand(-0.4,0.4),p.y+5,cz+rand(-0.4,0.4),0,rand(0.8,2),0,1.2,0.4,0.05,col(0xff3a4a),0,0,1);
      PA.emit(p.x+0.5+rand(-0.3,0.3),p.y+4,p.z+0.5+rand(-0.3,0.3),0,rand(0.5,1.5),0,1,0.35,0.05,col(0xff5a6a),0,0,1);}
    if(!p.chestOut&&d<52&&p.type!=='shrine'&&p.type!=='altar'&&!p.looted){p.chestOut=true;const cp=p.type==='ruin'?new V3(p.x+0.5,p.y+1,p.z+0.5):new V3(p.x+2.5,p.y+1,p.z+1.5);const m=chestModel();m.position.copy(cp);m.rotation.y=hash2(p.gx,p.gz,7)*TAU;scene.add(m);
      chests.push({mesh:m,pos:cp,open:false,t:0,poi:p});}
    if(p.chestOut&&d>110){for(let i=chests.length-1;i>=0;i--)if(chests[i].poi===p&&!chests[i].open){scene.remove(chests[i].mesh);chests.splice(i,1);}p.chestOut=false;}}
  if(AUTH()){for(const a of avatars()){const ax=Math.floor(a.pos.x/64),az=Math.floor(a.pos.z/64);for(let gz=az-1;gz<=az+1;gz++)for(let gx=ax-1;gx<=ax+1;gx++){const p=world.poi(gx,gz);
      if(!p||p.guarded||p.cleared||p.type==='shrine'||p.type==='altar')continue;if(Math.hypot(a.pos.x-p.x,a.pos.z-p.z)>52)continue;p.guarded=true;
      const types=p.type==='ruin'?['skeleton','skeleton','zombie']:['zombie','zombie','skeleton','zombie'];
      if(p.type==='ruin'&&G.day>=3&&hash2(p.gx,p.gz,world.seed+999)<0.4)types.push('golem');
      for(const t of types){const an=rand(0,TAU),r=t==='golem'?0:rand(2,5);const s=surfaceSpot(p.x+0.5+Math.cos(an)*r,p.z+0.5+Math.sin(an)*r)||new V3(p.x+0.5,p.y+1,p.z+0.5);
        const e=spawnEnt(t,s.x,s.y,s.z,{poi:p});if(e)e.wdir.set(0,0,0);}}}}
  if(nearFire&&nd<60){const fp=_t1.set(nearFire.x+0.5,nearFire.y+1.6,nearFire.z+0.5);fireLight.position.copy(fp);fireLight.intensity=(G.night?2.4:1)*(0.85+Math.sin(G.runT*13)*0.1+Math.random()*0.1);
    for(let i=0;i<2;i++)PA.emit(fp.x+rand(-0.3,0.3),fp.y-0.4,fp.z+rand(-0.3,0.3),rand(-0.3,0.3),rand(1.5,3),rand(-0.3,0.3),rand(0.4,0.8),0.6,0.1,col(pick([0xff6a1a,0xffb03a,0xff3a10])),-0.5,0.5,1);
    if(Math.random()<0.15)PN.emit(fp.x,fp.y+0.6,fp.z,rand(-0.3,0.3),1.5,rand(-0.3,0.3),2.2,0.4,1.6,col(0x4a4448),-0.3,0.3,0.35);}
  else fireLight.intensity=0;
  for(const c of chests){if(c.open){c.t+=dt;c.mesh.userData.lid.rotation.x=-Math.min(1.9,c.t*6);}else if(Math.random()<0.1)PA.emit(c.pos.x+rand(-0.4,0.4),c.pos.y+0.6,c.pos.z+rand(-0.4,0.4),0,0.8,0,0.8,0.25,0.05,col(0xffd35a),0,0,0.9);}
}
let curInteract=null;
function findInteract(){
  curInteract=null;if(P.dead)return;let best=3.2;
  for(const r of NET.peers.values()){if(!r.inGame||!r.dead)continue;const d=r.pos.distanceTo(P.pos);if(d<3){curInteract={label:`Hold to revive ${r.name}`,hold:true,peer:r};best=d;}}
  if(curInteract)return;
  for(const c of chests){if(c.open)continue;const d=c.pos.distanceTo(P.pos);if(d<best){best=d;curInteract={label:'Open chest',fn:()=>openChest(c)};}}
  const p=world.poi(Math.floor(P.pos.x/64),Math.floor(P.pos.z/64));
  if(p&&p.type==='shrine'&&!p.used){const d=Math.hypot(P.pos.x-p.x-0.5,P.pos.z-p.z-0.5);if(d<3.5&&d<best+1)curInteract={label:'Pray at the shrine',fn:()=>useShrine(p)};}
  if(p&&p.type==='altar'&&!p.used&&!G.trial){const d=Math.hypot(P.pos.x-p.x-0.5,P.pos.z-p.z-0.5);if(d<4.5)curInteract={label:'Awaken the altar: survive its trial for a relic',fn:()=>beginTrial(p)};}
  if(!curInteract&&aim.block&&aim.block.b===B.BERRY){const b=aim.block;const d=Math.hypot(b.x+0.5-P.pos.x,b.y+0.5-P.pos.y-1,b.z+0.5-P.pos.z);if(d<5)curInteract={label:'Pick berries',fn:()=>{setBlockNet(b.x,b.y,b.z,B.BUSH);
    const n=randi(2,3);P.food.berries+=n;sfx('pick');popText(_t1.set(b.x+0.5,b.y+1.2,b.z+0.5),'+'+n+' Berries','heal');fx.burst(_t1.set(b.x+0.5,b.y+0.6,b.z+0.5),0xd23a4e,14,3,0.4,0.25,{g:6});buildActionBar();}};}
}
function setBlockNet(x,y,z,b){world.set(x,y,z,b);if(NET.on)send({t:'block',x,y,z,b});}
function interact(){if(curInteract&&!P.dead&&curInteract.fn)curInteract.fn();}
function updateRevive(dt){const ci=curInteract;if(ci&&ci.hold&&keys.KeyE&&!G.ui){P.revive+=dt;if(Math.random()<0.5)PA.emit(ci.peer.pos.x+rand(-0.5,0.5),ci.peer.pos.y+rand(0,1),ci.peer.pos.z+rand(-0.5,0.5),0,2,0,0.6,0.35,0.05,col(0x8dff9e),0,0,1);
    if(P.revive>=2){P.revive=0;send({t:'revive',to:ci.peer.id});healFx(ci.peer.pos);sfx('heal');}}else P.revive=0;}
function openChest(c){c.open=true;c.t=0;sfx('chest');fx.burst(_t1.copy(c.pos).setY(c.pos.y+0.6),0xffd35a,50,6,0.8,0.35,{up:4,g:4});flashLight(_t1,0xffd35a,3,10,0.6);fx.pillar(c.pos.clone(),0xffd35a,0.7,8,0.8);
  if(c.poi){c.poi.looted=true;c.poi.cleared=true;}P.food.meat+=randi(0,2);P.food.berries+=randi(1,3);buildActionBar();gainXp(10+G.day*4);
  later(0.6,()=>c.relic?openPerks('Altar relic','The altar offers its power. Choose one.','relic'):openPerks('Treasure','The chest holds a relic. Choose one.'));}
function useShrine(p){p.used=true;sfx('holy');healPlayer(P.maxHp);P.hunger=Math.min(100,P.hunger+30);P.buffs.shrine=90;const c=new V3(p.x+0.5,p.y+1,p.z+0.5);
  fx.pillar(c,0xb98aff,2.5,40,2);fx.ring(c,0xb98aff,0.5,8,0.8);fx.burst(_t1.copy(c).setY(c.y+1.5),0xb98aff,60,6,1,0.4,{up:3});toast('Blessing of the old stones','Full heal and +25% damage for 90s',3);}
function eat(){if(P.dead)return;const k=P.food.meat>0?'meat':P.food.berries>0?'berries':null;if(!k){toast('No food','Hunt animals or pick berries from bushes',1.6);return;}
  if(P.hunger>=99&&P.hp>=P.maxHp){toast('Not hungry','',1);return;}
  P.food[k]--;const f=P.st.food;if(k==='meat'){P.hunger=Math.min(100,P.hunger+38*f);healPlayer(12*f);}else{P.hunger=Math.min(100,P.hunger+16*f);healPlayer(4*f);}
  sfx('eat');fx.burst(_t1.set(P.pos.x,P.pos.y+1.6,P.pos.z),k==='meat'?0xd06a6a:0xd23a4e,10,2,0.4,0.2,{g:8});buildActionBar();}

/* ---------------- perks & upgrades ---------------- */
const PERKS=[
  {id:'vit',name:'Vitality',d:'+25 max health and heal fully.',f:p=>{p.maxHp+=25;p.hp=p.maxHp;}},
  {id:'might',name:'Might',d:'+15% damage with everything.',f:p=>p.st.dmg+=0.15},
  {id:'haste',name:'Haste',d:'Abilities recharge 12% faster.',f:p=>p.st.cd*=0.88},
  {id:'swift',name:'Fleetfoot',d:'+10% movement speed.',f:p=>p.st.speed+=0.1},
  {id:'vamp',name:'Vampirism',d:'Heal for 4% of damage you deal.',f:p=>p.st.leech+=0.04},
  {id:'crit',name:'Keen Edge',d:'+8% critical chance. Crits deal double.',f:p=>p.st.crit+=0.08},
  {id:'skin',name:'Ironhide',d:'Take 10% less damage.',f:p=>p.st.armor=1-(1-p.st.armor)*0.9},
  {id:'regen',name:'Second Wind',d:'Regenerate 1 health per second.',f:p=>p.st.regen+=1},
  {id:'forage',name:'Forager',d:'Food restores 50% more. Animals drop extra meat.',f:p=>p.st.food+=0.5},
  {id:'hunter',name:'Giant Slayer',d:'+35% damage to elites and bosses.',f:p=>p.st.elite+=0.35},
  {id:'magnet',name:'Lodestone',d:'Collect experience and loot from twice as far.',f:p=>p.st.magnet*=1.8},
  {id:'aoe',name:'Amplify',d:'All area abilities are 20% larger.',f:p=>p.st.area+=0.2},
  {id:'stam',name:'Endurance',d:'+40 stamina, recovers 30% faster.',f:p=>{p.st.stamMax+=40;p.st.stamRegen+=0.3;}},
  {id:'fury',name:'Fury',d:'Weapon attacks are 15% faster.',f:p=>p.st.atkSpd+=0.15},
  {id:'m1',cls:'mage',name:'Pyromancer',d:'Fire burns 35% larger and longer.',f:p=>p.st.fireRad+=0.35},
  {id:'m2',cls:'mage',name:'Shatter',d:'Frozen enemies take 50% more damage.',f:p=>p.st.shatter+=0.5},
  {id:'w1',cls:'warrior',name:'Bloodlust',d:'Whirlwind heals 3 health per enemy hit.',f:p=>p.st.wwHeal+=3},
  {id:'w2',cls:'warrior',name:'Juggernaut',d:'+40 max health and take 8% less damage.',f:p=>{p.maxHp+=40;p.hp+=40;p.st.armor=1-(1-p.st.armor)*0.92;}},
  {id:'p1',cls:'priest',name:'Radiance',d:'Mend also blasts nearby enemies for 40.',f:p=>p.st.radiance=1,once:true},
  {id:'r2',cls:'ranger',name:'Quickdraw',d:'Draw your bow 35% faster.',f:p=>p.st.chargeMul+=0.35},
  {id:'wis',name:'Wisdom',d:'Gain 20% more experience.',f:p=>p.st.xpMul+=0.2},
  {id:'luck',name:'Fortune',d:'+2 rerolls on level-ups.',f:p=>p.rerolls+=2},
];
/* every level-up draws three picks from one weighted pool: ability and weapon upgrades, relics (new or ranked up) and general boons */
function upgradeOptions(){const C=CLASSES[P.cls];const out=[];
  for(const ab of C.abilities){let inv=0;for(const[k,,,,ms]of MODS[ab])if(!ms)inv+=M(P,ab+'.'+k);
    for(const[k,n,d,max,ms]of MODS[ab]){const key=ab+'.'+k,r=M(P,key);if(r>=max||(ms&&inv<2))continue;
      out.push({kind:'mod',key,name:n,d,tag:ms?`${ABIL[ab].name} mastery`:`${ABIL[ab].name} upgrade · rank ${r+1}/${max}`,icon:ABIL[ab].icon,color:ABIL[ab].color,w:ms?2.2:1,mastery:!!ms});}}
  for(const w of C.weapons)for(const[k,n,d,max]of WMODS[w]){const key='w.'+w+'.'+k,r=M(P,key);if(r<max)out.push({kind:'mod',key,name:n,d,tag:`${WEAPONS[w].name} upgrade · rank ${r+1}/${max}`,icon:WEAPONS[w].icon,color:C.color,w:0.8});}
  const owned=relicCount(P);
  for(const k in RELICS){const R=RELICS[k],r=RK(P,k);if(r>=RMAX||(!r&&owned>=RSLOTS))continue;
    out.push({kind:'mod',key:'r.'+k,name:R.name,d:R.ranks[r],tag:r?`Relic · rank ${r+1}/${RMAX}`:'New relic',icon:R.icon,color:R.color,w:r?1.7:(owned<2?2.4:1.3),relic:true});}
  for(const k in AUGS){const A=AUGS[k],r=AK(P,k);if(r>=A.ranks.length)continue;
    out.push({kind:'mod',key:'x.'+k,name:A.name,d:A.ranks[r],tag:r?`Chaos augment · rank ${r+1}/${A.ranks.length}`:'Chaos augment',icon:'chaos',color:AUG_COLOR,w:P.lvl<3?0.4:r?1.1:1.2,chaos:true});}
  const pool=PERKS.filter(p=>(!p.cls||p.cls===P.cls)&&!(p.once&&P.perks.includes(p.name)));
  for(const p of pool)out.push({kind:'perk',name:p.name,d:p.d,tag:p.cls?CLASSES[p.cls].name+' boon':'Boon',color:p.cls?CLASSES[p.cls].color:'var(--ember)',w:5/pool.length,f:p.f});
  return out;}
function rollChoices(only){let pool=upgradeOptions();if(only==='relic'){const r=pool.filter(c=>c.relic||c.mastery);if(r.length>=1)pool=r;}const out=[];
  while(out.length<3&&pool.length){let tot=0;for(const c of pool)tot+=c.w;let r=Math.random()*tot,k=0;for(;k<pool.length-1;k++){r-=pool[k].w;if(r<=0)break;}out.push(pool.splice(k,1)[0]);}
  return out;}
function openPerks(title,sub,only){
  if(P.dead||G.state!=='playing')return;if(G.ui){G.perkQueue=(G.perkQueue||0)+1;return;}
  G.ui='levelup';mouse.l=false;P.charging=false;if(document.pointerLockElement)document.exitPointerLock();
  $('#lv-title').textContent=title;$('#lv-sub').textContent=sub+(NET.on?' The world keeps moving while you choose.':'');G.perkOnly=only;renderChoices(rollChoices(only));
  $('#levelup').hidden=false;
}
function renderChoices(choice){const box=$('#perks');box.innerHTML='';
  choice.forEach((c,i)=>{const b=document.createElement('button');b.className='perk'+(c.mastery?' mastery':c.relic?' relic':c.chaos?' chaos':'');b.style.setProperty('--c',c.color);
    b.innerHTML=`<em>${c.tag} · ${i+1}</em><div class="ph">${c.icon?'<canvas width="64" height="64"></canvas>':''}<b>${c.name}</b></div><span>${c.d}</span>`;
    if(c.icon)drawIcon(b.querySelector('canvas'),c.icon,c.color);
    b.onclick=()=>pickChoice(c);box.appendChild(b);});
  const rr=$('#reroll-perks');rr.hidden=P.rerolls<=0;rr.textContent=`Reroll (${P.rerolls})`;
  setTimeout(()=>box.firstChild&&box.firstChild.focus(),50);}
function pickChoice(c){if(c.kind==='mod'){const was=M(P,c.key);P.mods[c.key]=was+1;if(c.key==='x.storm'&&!was)P.st.crit+=0.05;if(c.chaos)sfx('bomb');if(c.relic){sfx('relic');if(!was)toast(c.name,'A new relic joins your arsenal',2);}}else c.f(P);
  P.perks.push(c.name);$('#levelup').hidden=true;G.ui=null;buildActionBar();if(!G.noLock)lockPointer();
  if(G.perkQueue>0){G.perkQueue--;later(0.3,()=>openPerks('Level '+P.lvl,'Another upgrade awaits.'));}}
$('#reroll-perks').onclick=()=>{if(P.rerolls<=0||G.ui!=='levelup')return;P.rerolls--;sfx('pick');renderChoices(rollChoices(G.perkOnly));};
addEventListener('keydown',e=>{if(G.ui!=='levelup')return;if(/^Digit[123]$/.test(e.code)){const b=$('#perks').children[+e.code.slice(5)-1];b&&b.click();}if(e.code==='KeyR')$('#reroll-perks').click();});

/* ---------------- HUD ---------------- */
function toast(t,s,dur=2){const el=$('#toast');el.innerHTML=t+(s?`<small>${s}</small>`:'');el.style.opacity=1;clearTimeout(toast._t);toast._t=setTimeout(()=>el.style.opacity=0,dur*1000);}
function pips(n,max){if(!max)return'';let s='<span class="pips">';for(let i=0;i<max;i++)s+=`<i class="${i<n?'on':''}"></i>`;return s+'</span>';}
function modRank(prefix,list){let n=0,m=0;for(const[k,,,max]of list){n+=M(P,prefix+k);m+=max;}return[n,m];}
function buildActionBar(){
  const C=CLASSES[P.cls],bar=$('#actbar');bar.innerHTML='';
  const id=C.weapons[P.widx],w=WEAPONS[id],ow=WEAPONS[C.weapons[1-P.widx]];const[wn]=modRank('w.'+id+'.',WMODS[id]);
  const ws=document.createElement('div');ws.className='slot wep ready';ws.style.setProperty('--c',C.color);ws.innerHTML=`<span class="k">LMB</span><canvas width="96" height="96"></canvas>${wn?`<span class="rk">+${wn}</span>`:''}<span class="nm">${w.name}</span>`;drawIcon(ws.querySelector('canvas'),w.icon,C.color);bar.appendChild(ws);
  const os=document.createElement('div');os.className='slot';os.style.opacity=0.6;os.innerHTML=`<span class="k">Q</span><canvas width="80" height="80"></canvas><span class="nm">Swap</span>`;drawIcon(os.querySelector('canvas'),ow.icon,'#bfb29b');bar.appendChild(os);
  bar.appendChild(Object.assign(document.createElement('div'),{className:'sep'}));
  C.abilities.forEach((ab,i)=>{const a=ABIL[ab];const[n]=modRank(ab+'.',MODS[ab]);const s=document.createElement('div');s.className='slot';s.dataset.i=i;s.style.setProperty('--c',a.color);
    s.title=MODS[ab].filter(([k])=>M(P,ab+'.'+k)).map(([k,nm])=>nm+' '+M(P,ab+'.'+k)).join(', ');
    s.innerHTML=`<span class="k">${i+1}</span><canvas width="80" height="80"></canvas><div class="cdv"></div><div class="cdn"></div>${n?`<span class="rk">+${n}</span>`:''}<span class="nm">${a.name}</span>`;drawIcon(s.querySelector('canvas'),a.icon,a.color);bar.appendChild(s);});
  bar.appendChild(Object.assign(document.createElement('div'),{className:'sep'}));
  const f=document.createElement('div');f.className='slot food';f.style.setProperty('--c','#d58a3c');f.innerHTML=`<span class="k">F</span><canvas width="64" height="64"></canvas><span class="cnt">${P.food.meat+P.food.berries}</span><span class="nm">Eat</span>`;
  drawIcon(f.querySelector('canvas'),'food','#d58a3c');f.title=`${P.food.meat} meat, ${P.food.berries} berries`;bar.appendChild(f);
  slotEls=[...bar.querySelectorAll('.slot[data-i]')].map(s=>({s,cdv:s.querySelector('.cdv'),cdn:s.querySelector('.cdn')}));
  const rl=$('#relics');rl.innerHTML='';for(const k in RELICS){const r=RK(P,k);if(!r)continue;const R=RELICS[k];const d=document.createElement('div');d.className='rel';d.style.setProperty('--c',R.color);d.title=R.name+' · rank '+r;
    d.innerHTML=`<canvas width="48" height="48"></canvas><b>${r}</b>`;drawIcon(d.querySelector('canvas'),R.icon,R.color);rl.appendChild(d);}
  for(const k in AUGS){const r=AK(P,k);if(!r)continue;const d=document.createElement('div');d.className='rel';d.style.setProperty('--c',AUG_COLOR);d.title=AUGS[k].name+' · rank '+r;
    d.innerHTML=`<canvas width="48" height="48"></canvas><b>${r}</b>`;drawIcon(d.querySelector('canvas'),'chaos',AUG_COLOR);rl.appendChild(d);}
  rl.hidden=!rl.children.length;const sh=document.querySelector('.stat').offsetHeight+24;$('#buffs').style.top=sh+'px';$('#party').style.top=(sh+34)+'px';
}
let slotEls=[];
const H={timer:$('#h-timer'),kills:$('#h-kills'),xp2:$('#h-xp2'),lv2:$('#h-lv2'),threat:$('#h-threat'),cls:$('#h-cls'),lvl:$('#h-lvl'),hp:$('#h-hp'),hpg:$('#h-hpg'),hpt:$('#h-hpt'),st:$('#h-st'),fd:$('#h-fd'),xp:$('#h-xp'),day:$('#h-day'),time:$('#h-time'),dayi:$('#h-dayi'),vig:$('#vig'),prompt:$('#prompt'),charge:$('#charge'),chargeI:$('#charge-i'),buffs:$('#buffs'),boss:$('#boss'),bossHp:$('#boss-hp'),bossN:$('#boss-n'),party:$('#party')};
let hudT=0;
function updateHUD(dt){
  H.hp.style.transform=`scaleX(${Math.max(0,P.hp/P.maxHp)})`;H.hpg.style.transform=`scaleX(${Math.max(0,P.ghostHp/P.maxHp)})`;H.st.style.transform=`scaleX(${P.stam/P.st.stamMax})`;H.fd.style.transform=`scaleX(${P.hunger/100})`;
  H.xp.style.transform=H.xp2.style.transform=`scaleX(${Math.min(1,P.xp/xpNeed())})`;
  H.vig.style.opacity=P.dead?0.6:Math.max(P.hitT*2,P.hp/P.maxHp<0.3?0.35+Math.sin(G.runT*6)*0.2:0,P.hunger<=0?0.25:0);
  H.charge.hidden=!P.charging;if(P.charging)H.chargeI.style.transform=`scaleX(${P.charge})`;
  slotEls.forEach((o,i)=>{const id=CLASSES[P.cls].abilities[i];const cd=P.cds[i],max=cdOf(P,id);
    if(cd>0){o.cdv.style.background=`conic-gradient(rgba(5,4,8,.72) ${cd/max}turn, transparent 0)`;o.cdn.textContent=cd>=1?Math.ceil(cd):cd.toFixed(1);o.s.classList.remove('ready');}
    else{if(o.cdn.textContent){o.cdv.style.background='none';o.cdn.textContent='';}o.s.classList.add('ready');}});
  if(curInteract){H.prompt.hidden=false;H.prompt.innerHTML=`<kbd>E</kbd> ${curInteract.label}${curInteract.hold&&P.revive>0?` <i class="rv" style="--p:${P.revive/2}"></i>`:''}`;}else H.prompt.hidden=true;
  hudT-=dt;if(hudT>0)return;hudT=0.2;
  H.cls.textContent=CLASSES[P.cls].name;H.lvl.textContent='Level '+P.lvl;H.lv2.textContent='LV '+P.lvl;
  const rs=Math.floor(G.runT);H.timer.textContent=`${String(Math.floor(rs/60)).padStart(2,'0')}:${String(rs%60).padStart(2,'0')}`;H.kills.textContent=P.kills+' slain';
  const th=threatName();H.threat.textContent=th[0];H.threat.style.color=th[1];H.hpt.textContent=`${Math.max(0,Math.ceil(P.hp))} / ${P.maxHp}`;
  const t=(G.time%1);const hrs=(6+t*24)%24;H.day.textContent=(G.blood&&G.night?'Blood Moon · ':'')+'Day '+G.day;H.time.textContent=`${String(Math.floor(hrs)).padStart(2,'0')}:${String(Math.floor((hrs%1)*60)).padStart(2,'0')}`;
  H.dayi.style.marginLeft=`calc(${t*100}% - 1px)`;
  const bl=[];if(P.buffs.cry)bl.push(['Battle Cry',P.buffs.cry,'#ff5b4c']);if(P.buffs.shield)bl.push(['Divine Shield',P.buffs.shield,'#fff3b8']);if(P.buffs.hot)bl.push(['Mending',P.buffs.hot,'#8dff9e']);
  if(P.buffs.shrine)bl.push(['Blessed',P.buffs.shrine,'#b98aff']);if(G.trial)bl.push(['Altar trial',G.trial.trial?G.trial.trial.t:0,'#ff3a4a']);if(P.hunger<=0)bl.push(['Starving',0,'#e0473b']);else if(P.hunger>75)bl.push(['Well fed',0,'#d58a3c']);
  if(P.dead&&NET.on)bl.push(['Downed: wait for a revive or dawn',0,'#e0473b']);
  if(Q.lvl<2)bl.push([['Low','Medium'][Q.lvl]+' graphics',0,'#8c816f']);
  H.buffs.innerHTML=bl.map(b=>`<span style="--c:${b[2]}">${b[0]}${b[1]?' '+Math.ceil(b[1])+'s':''}</span>`).join('');
  const boss=ents.find(e=>e.elite&&!e.dead&&e.pos.distanceTo(P.pos)<45&&(e.boss||e.type==='golem'));H.boss.hidden=!boss;if(boss){H.bossN.textContent=boss.T.name;H.bossHp.style.transform=`scaleX(${Math.max(0,boss.hp/boss.maxHp)})`;}
  const fs=document.querySelector('.slot.food .cnt');if(fs)fs.textContent=P.food.meat+P.food.berries;
  renderParty();
}
function renderParty(){if(!H.party)return;const rs=[...NET.peers.values()].filter(r=>r.inGame);H.party.hidden=!NET.on||!rs.length;
  H.party.innerHTML=rs.map(r=>`<div style="--c:${CLASSES[r.cls].color}"><b>${r.name}</b><small>${CLASSES[r.cls].name} · Lv ${r.lvl}${r.dead?' · Downed':''}</small><i><s style="transform:scaleX(${clamp(r.hp/r.maxHp,0,1)})"></s></i></div>`).join('');}
function updateBars(){const W=innerWidth,H2=innerHeight;for(const e of ents){if(!e.bar)continue;const d=e.pos.distanceTo(P.pos);if((e.hp>=e.maxHp&&!e.affix)||d>34||e.dead){e.bar.style.display='none';continue;}
  _pv.set(e.pos.x,e.pos.y+e.h+0.45,e.pos.z).project(camera);if(_pv.z>1){e.bar.style.display='none';continue;}e.bar.style.display='block';
  e.bar.style.transform=`translate(${(_pv.x+1)/2*W}px,${(1-_pv.y)/2*H2}px) translate(-50%,-50%)`;e.barI.style.transform=`scaleX(${Math.max(0,e.hp/e.maxHp)})`;}}
const mm=$('#minimap'),mctx=mm.getContext('2d'),mimg=mctx.createImageData(128,128);let mmT=0;const mmBase=document.createElement('canvas');mmBase.width=mmBase.height=128;const mbctx=mmBase.getContext('2d');
function updateMinimap(dt){
  mmT-=dt;const px=Math.floor(P.pos.x),pz=Math.floor(P.pos.z);
  if(mmT<=0){mmT=0.4;const d=mimg.data;for(let y=0;y<128;y++)for(let x=0;x<128;x++){const wx=px-64+x,wz=pz-64+y;const cx=Math.floor(wx/CS),cz=Math.floor(wz/CS);const i=(y*128+x)*4;
      if(!world.hasChunk(cx,cz)){d[i]=d[i+1]=d[i+2]=18;d[i+3]=255;continue;}const ch=world.getChunk(cx,cz);const li=(wx-cx*CS)+(wz-cz*CS)*CS;const t=ch.top[li],b=ch.topB[li];
      let r,g,bb;if(t<=SEA){const k=0.55+(t/SEA)*0.35;r=40*k;g=110*k;bb=170*k;}else{const c=BL[b]?(BL[b].top||BL[b].c):BL[1].c;const k=0.65+(t-SEA)/40;r=c.r*255*k;g=c.g*255*k;bb=c.b*255*k;}
      d[i]=r;d[i+1]=g;d[i+2]=bb;d[i+3]=255;}mbctx.putImageData(mimg,0,0);}
  mctx.clearRect(0,0,128,128);mctx.drawImage(mmBase,0,0);
  const toM=(x,z)=>[x-px+64,z-pz+64];
  const gx=Math.floor(P.pos.x/64),gz=Math.floor(P.pos.z/64);for(let j=gz-1;j<=gz+1;j++)for(let i=gx-1;i<=gx+1;i++){const p=world.poi(i,j);if(!p)continue;const[x,y]=toM(p.x,p.z);if(x<0||y<0||x>128||y>128)continue;
    mctx.fillStyle=p.type==='shrine'?(p.used?'#6a5a8a':'#c9a2ff'):p.type==='altar'?(p.used?'#6a3a3a':'#ff3a4a'):p.type==='camp'?'#ff9a3d':'#f2cf63';mctx.strokeStyle='#000';mctx.lineWidth=1.5;mctx.beginPath();
    if(p.type==='shrine'||p.type==='altar'){mctx.moveTo(x,y-5);mctx.lineTo(x+4,y);mctx.lineTo(x,y+5);mctx.lineTo(x-4,y);}else mctx.rect(x-3.5,y-3.5,7,7);mctx.closePath();mctx.stroke();mctx.fill();}
  for(const e of ents){const[x,y]=toM(e.pos.x,e.pos.z);if(x<0||y<0||x>128||y>128)continue;mctx.fillStyle=e.T.prop?'#ffd36a':e.passive?'#f4ead8':e.elite?'#ff9a2e':'#ff4a3a';mctx.fillRect(x-1.5,y-1.5,e.elite?5:3,e.elite?5:3);}
  for(const r of NET.peers.values()){if(!r.inGame)continue;const[x,y]=toM(r.pos.x,r.pos.z);mctx.fillStyle=CLASSES[r.cls].color;mctx.strokeStyle='#000';mctx.beginPath();mctx.arc(clamp(x,4,124),clamp(y,4,124),3.5,0,TAU);mctx.fill();mctx.stroke();}
  mctx.save();mctx.translate(64,64);mctx.rotate(-P.yaw+Math.PI);mctx.fillStyle='#fff';mctx.strokeStyle='#000';mctx.lineWidth=1.5;mctx.beginPath();mctx.moveTo(0,-6);mctx.lineTo(4.5,5);mctx.lineTo(0,2.5);mctx.lineTo(-4.5,5);mctx.closePath();mctx.stroke();mctx.fill();mctx.restore();
}

/* ---------------- day / night ---------------- */
const SKY={dayTop:new THREE.Color(0x3f86d6),dayHor:new THREE.Color(0xb4dcf5),setTop:new THREE.Color(0x3a4a8a),setHor:new THREE.Color(0xf29a5a),nightTop:new THREE.Color(0x060a1e),nightHor:new THREE.Color(0x18224a),
  bloodTop:new THREE.Color(0x1a0408),bloodHor:new THREE.Color(0x5a1014)};
const _top=new THREE.Color(),_hor=new THREE.Color(),_sd=new V3();
function updateDay(dt,center){
  if(G.state==='playing')G.time+=dt/DAY_LEN;
  const t=G.time%1;const ang=t*TAU;_sd.set(Math.cos(ang)*0.85,Math.sin(ang),0.4).normalize();
  const day=smooth(-0.15,0.2,_sd.y),dusk=1-Math.abs(clamp(_sd.y*4,-1,1));
  G.day=Math.floor(G.time)+1;const night=day<0.35;G.blood=G.day%3===0&&night;
  if(G.state==='playing'&&night!==G.lastNight){
    if(night){toast(G.blood?'Blood Moon rises':'Night falls',G.blood?'The dead grow restless. Something ancient stirs.':'Monsters grow bolder in the dark',3.5);sfx('spawn');
      if(G.blood&&G.bossAnnounced!==G.day&&AUTH()){G.bossAnnounced=G.day;later(8,()=>{const tg=avatars().filter(a=>!a.dead);if(!tg.length)return;const c=pick(tg).pos;const an=rand(0,TAU);const s=surfaceSpot(c.x+Math.cos(an)*30,c.z+Math.sin(an)*30);
        if(s&&spawnEnt('golem',s.x,s.y,s.z,{boss:true})){toast('An Ancient Golem awakens','Defeat it for a relic',3);sfx('boom');G.shake=0.6;}});}}
    else{toast('Day '+G.day,'You survived the night',3);if(P.dead&&NET.on&&!G.over){G.lastNight=night;revivePlayer(0.5);}}
    G.lastNight=night;}
  G.night=night;
  const nt=G.blood?SKY.bloodTop:SKY.nightTop,nh=G.blood?SKY.bloodHor:SKY.nightHor;
  _top.copy(nt).lerp(SKY.dayTop,day);_hor.copy(nh).lerp(SKY.dayHor,day);_hor.lerp(SKY.setHor,dusk*0.55*smooth(-0.3,0.1,_sd.y));_top.lerp(SKY.setTop,dusk*0.3);
  skyMat.uniforms.top.value.copy(_top);skyMat.uniforms.hor.value.copy(_hor);skyMat.uniforms.sunDir.value.copy(_sd);skyMat.uniforms.sunCol.value.setRGB(1,0.85+day*0.1,0.6+day*0.3).multiplyScalar(day>0.01?1:0);
  skyMat.uniforms.night.value=1-day;skyMat.uniforms.moonCol.value.set(G.blood?0xff5a4a:0xdfe8ff);
  scene.fog.color.copy(_hor);stars.material.opacity=Math.max(0,1-day*1.6)*0.9;
  const up=_sd.y>0;const ldir=up?_sd:_t3.copy(_sd).negate();const ws=92/sun.shadow.mapSize.x;_t2.set(Math.round(center.x/ws)*ws,Math.round(center.y/ws)*ws,Math.round(center.z/ws)*ws);
  sun.position.copy(_t2).addScaledVector(ldir,120);sun.target.position.copy(_t2);
  sun.intensity=up?0.86*day:0.32*(1-day);sun.color.set(up?0xfff0d0:(G.blood?0xff7a6a:0x9ab0ff));if(up)sun.color.lerp(_c.set(0xffa060),dusk*0.6);
  hemi.intensity=0.24+0.38*day;hemi.color.copy(_top).lerp(_c.set(0xffffff),0.35);hemi.groundColor.set(G.blood?0x3a1010:0x4a3e30);amb.intensity=0.1+0.05*day;
  waterMat.uniforms.uDay.value=0.3+0.7*day;waterMat.uniforms.uSun.value.copy(_sd);waterMat.uniforms.uSky.value.copy(_hor).lerp(_top,0.3);
  clouds.material.color.copy(_hor).lerp(_c.set(0xffffff),0.5+0.4*day);if(dusk>0.3&&up)clouds.material.color.lerp(_c.set(0xffb08a),dusk*0.35);clouds.material.opacity=0.6+0.28*day;
}

/* the post pipeline needs the sun: shafts only by day, and a warmer, stronger bloom at night and in the blood moon */
const _sunCol=new THREE.Color();
function renderFrame(){const up=_sd.y>0.02;const day=smooth(-0.15,0.2,_sd.y);_sunCol.copy(sun.color).multiplyScalar(up?0.85:0);
  const U=POST.uniforms;U.bloom.value=0.6+(1-day)*0.3;U.exposure.value=1.0+(1-day)*0.08;U.sat.value=G.blood&&G.night?1.18:1.1;
  POST.render(up?_sd:null,_sunCol,up?0.9*day:0);}

/* ---------------- camera ---------------- */
const camPos=new V3(),_cd=new V3(),_tgt=new V3(),camS={d:6.6,sh:0.75};
function updateCamera(dt){
  const f=lookOf(P,_cd);const right=rightOf(P,_rt);
  _tgt.set(P.pos.x,P.visY+1.78,P.pos.z);
  let sh=0.75;for(let s=0.25;s<=0.75;s+=0.25)if(world.solid(_tgt.x+right.x*s,_tgt.y,_tgt.z+right.z*s)){sh=Math.max(0,s-0.3);break;}
  camS.sh=sh<camS.sh?lerp(camS.sh,sh,Math.min(1,dt*20)):lerp(camS.sh,sh,Math.min(1,dt*4));_tgt.addScaledVector(right,camS.sh);
  let dist=6.6;const back=_t1.copy(f).negate();
  // exact voxel raycast from the shoulder back to the camera, plus two offset rays so blocks just off the line don't fill the screen
  for(const oy of[0,0.35,-0.35]){_t3.set(_tgt.x,_tgt.y+oy,_tgt.z);const hit=voxelRay(_t3,back,dist);if(hit)dist=Math.max(0.5,hit.t-0.3);}
  // pull in quickly when something blocks the view, ease back out gently, so sweeping past trees doesn't make the camera pop
  const hard=dist;camS.d=dt<=0?dist:dist<camS.d?lerp(camS.d,dist,Math.min(1,dt*22)):lerp(camS.d,dist,Math.min(1,dt*3.5));camS.d=Math.min(camS.d,hard+0.6);
  camPos.copy(_tgt).addScaledVector(back,camS.d);
  if(G.shake>0){const s=G.shake*G.shake*0.35;camPos.x+=rand(-s,s);camPos.y+=rand(-s,s);camPos.z+=rand(-s,s);G.shake=Math.max(0,G.shake-dt*2.2);}
  camera.position.copy(camPos);camera.lookAt(_t2.copy(camPos).add(f));
  const tf=P.charging?64-P.charge*10:(P.dash?80:72);camera.fov=lerp(camera.fov,tf,Math.min(1,dt*8));camera.updateProjectionMatrix();
}
let menuA=0,menuCenter=new V3();
function menuCamera(dt){menuA+=dt*0.05;const c=menuCenter;const r=34;camera.position.set(c.x+Math.cos(menuA)*r,c.y+16,c.z+Math.sin(menuA)*r);camera.lookAt(c.x,c.y+2,c.z);if(camera.fov!==60){camera.fov=60;camera.updateProjectionMatrix();}}

/* ---------------- quality ---------------- */
function applyQuality(l){Q.lvl=l;POST.setLevel(l);const L=[{view:4,sh:0,pr:1,pq:0.35},{view:5,sh:1024,pr:1.25,pq:0.65},{view:6,sh:2048,pr:1.75,pq:1}][l];
  Q.view=L.view;scene.fog.far=L.view*CS-6;scene.fog.near=Math.min(34,scene.fog.far*0.45);
  sun.castShadow=L.sh>0;if(L.sh&&sun.shadow.mapSize.x!==L.sh){sun.shadow.mapSize.set(L.sh,L.sh);if(sun.shadow.map){sun.shadow.map.dispose();sun.shadow.map=null;}}
  renderer.setPixelRatio(Math.min(window.devicePixelRatio||1,L.pr));renderer.setSize(innerWidth,innerHeight);PA.q=PN.q=L.pq;PA.pts.material.uniforms.pr.value=PN.pts.material.uniforms.pr.value=renderer.getPixelRatio();
  const sel=$('#gfx');if(sel&&Q.mode!=='auto')sel.value=String(l);}
function autoQuality(dt){if(Q.mode!=='auto'||G.state!=='playing')return;Q.ema=lerp(Q.ema,dt*1000,0.05);
  if(Q.ema>30){Q.badT+=dt;Q.goodT=0;if(Q.badT>2.5&&Q.lvl>0){applyQuality(Q.lvl-1);Q.badT=0;Q.ema=18;toast('Graphics lowered','Keeping the frame rate smooth',1.5);}}
  else if(Q.ema<13){Q.goodT+=dt;Q.badT=0;if(Q.goodT>12&&Q.lvl<2){applyQuality(Q.lvl+1);Q.goodT=0;}}else Q.badT=Math.max(0,Q.badT-dt);}
$('#gfx').onchange=e=>{const v=e.target.value;if(v==='auto'){Q.mode='auto';}else{Q.mode='fixed';applyQuality(+v);}try{localStorage.setItem('emberwild-gfx',v);}catch(err){}};

/* ---------------- flow ---------------- */
function newWorld(seed){
  for(const ch of meshed.values())disposeChunk(ch);meshed.clear();if(world)for(const ch of world.chunks.values())disposeChunk(ch);
  for(const e of ents)removeEnt(e);ents=[];
  G.seed=seed;world=new World(seed);$('#seed').textContent=seed;menuCenter=world.spawnPoint();
  updateChunks(menuCenter.x,menuCenter.z,9999);
}
function buildMenu(){const box=$('#classes');box.innerHTML='';
  for(const id of Object.keys(CLASSES)){const C=CLASSES[id];const b=document.createElement('button');b.className='cls';b.style.setProperty('--c',C.color);b.id='cls-'+id;
    b.innerHTML=`<h2>${C.name}<small>${C.hp} HP</small></h2><p class="role">${C.role}</p><h3>Weapons</h3><ul>${C.weapons.map(w=>`<li><canvas width="44" height="44" data-i="${WEAPONS[w].icon}"></canvas><span><b>${WEAPONS[w].name}.</b> ${WEAPONS[w].desc}</span></li>`).join('')}</ul>
      <h3>Abilities</h3><ul>${C.abilities.map((a,i)=>`<li><canvas width="44" height="44" data-i="${ABIL[a].icon}" data-c="${ABIL[a].color}"></canvas><span><b>${i+1} · ${ABIL[a].name}.</b> ${ABIL[a].desc}</span></li>`).join('')}</ul>
      <p class="ups">${C.abilities.reduce((n,a)=>n+MODS[a].length,0)+C.weapons.reduce((n,w)=>n+WMODS[w].length,0)} upgrades to unlock, like ${MODS[C.abilities[0]][1][1]} and ${MODS[C.abilities[1]][0][1]}.</p><span class="go">Play as ${C.name} →</span>`;
    b.querySelectorAll('canvas').forEach(cv=>drawIcon(cv,cv.dataset.i,cv.dataset.c||C.color));b.onclick=()=>startRun(id);box.appendChild(b);}
  try{const best=JSON.parse(localStorage.getItem('emberwild-best')||'null');if(best)$('#best').textContent=`Best run: ${best.days} days as ${best.cls}, level ${best.lvl}, ${best.kills} kills`;}catch(e){}
  renderCoop();
}
function coopStatus(s){$('#coop-status').textContent=s;}
function renderCoop(){const on=NET.on;$('#host').hidden=on||!(window.native&&window.native.host);$('#join').hidden=on;$('#addr').disabled=on;$('#leave').hidden=!on;
  $('#host-note').hidden=!!(window.native&&window.native.host)||on;$('#reroll').disabled=on&&!NET.host;
  if(on&&!NET.host&&!NET.welcomed)coopStatus('Connected. Waiting for the host…');}
$('#pname').value=(()=>{try{return localStorage.getItem('emberwild-name')||'';}catch(e){return'';}})();
$('#pname').oninput=e=>{P.name=cleanName(e.target.value);try{localStorage.setItem('emberwild-name',e.target.value);}catch(err){}};
P.name=cleanName($('#pname').value||'Wanderer');
$('#host').onclick=async()=>{coopStatus('Starting a server…');try{const r=await window.native.host(7777);NET.port=r.port;NET.hostIps=r.ips;await netConnect('ws://127.0.0.1:'+r.port);}catch(e){coopStatus('Could not host: '+(e&&e.message||e)+'. Is port 7777 already in use?');}};
$('#join').onclick=async()=>{let a=$('#addr').value.trim();if(!a){coopStatus('Type the host address, like 192.168.1.20:7777');return;}if(!/^wss?:\/\//.test(a))a='ws://'+a;if(!/:\d+$/.test(a.replace(/^wss?:\/\//,'')))a+=':7777';
  coopStatus('Connecting to '+a+'…');try{await netConnect(a);try{localStorage.setItem('emberwild-addr',$('#addr').value.trim());}catch(e){}}catch(e){coopStatus('Could not connect to '+a+'. Check the address and that the host allowed it through their firewall.');}};
$('#leave').onclick=()=>{netLeave();coopStatus('Left the session.');};
try{$('#addr').value=localStorage.getItem('emberwild-addr')||'';}catch(e){}
function startRun(cls){AU.init();if(NET.on&&!NET.host&&!NET.welcomed){coopStatus('Still waiting for the host to answer…');return;}
  $('#menu').hidden=true;$('#hud').hidden=false;resetRun(cls);buildActionBar();G.state='playing';
  updateChunks(P.pos.x,P.pos.z,9999);toast('Day '+G.day,CLASSES[cls].name+' · Survive. Explore. Grow stronger.',3.5);
  later(4,()=>toast('Tip','Level up to unlock upgrades for each ability',3));later(8,()=>toast('Tip','Hunt animals and pick berries. Press F to eat.',3));
  if(!G.noLock)lockPointer();canvas.focus();}
function pauseGame(){if(G.state!=='playing'||G.ui)return;G.ui='paused';$('#pause').hidden=false;mouse.l=false;$('#pause-note').hidden=!NET.on;}
function resume(){$('#pause').hidden=true;G.ui=null;if(!G.noLock)lockPointer();}
$('#resume').onclick=resume;
if(window.native&&window.native.quit){for(const id of['#quit-app','#quit-app2','#fullscreen'])$(id).hidden=false;$('#quit-app').onclick=$('#quit-app2').onclick=()=>window.native.quit();$('#fullscreen').onclick=()=>window.native.fullscreen();}
$('#quit').onclick=()=>{$('#pause').hidden=true;G.ui=null;if(NET.on){netLeave();}G.over=true;P.hp=0;killPlayer();};
function showDeath(){G.state='dead';G.ui=null;$('#levelup').hidden=true;$('#hud').hidden=true;$('#death').hidden=false;
  for(const e of ents)if(e.bar)e.bar.style.display='none';for(const r of NET.peers.values())if(r.tag)r.tag.style.display='none';
  const days=G.day,secs=Math.floor(G.runT);const C=CLASSES[P.cls];
  $('#d-title').textContent=NET.on?(days>1?`Your party lasted ${days} days`:'Your party fell on the first day'):(days>1?`You lasted ${days} days`:'You fell on the first day');
  $('#d-sub').textContent=`${C.name} · seed ${G.seed}${P.perks.length?' · Upgrades: '+P.perks.join(', '):''}`;
  $('#d-stats').innerHTML=[[days,'Days'],[P.lvl,'Level'],[P.kills,'Kills'],[`${Math.floor(secs/60)}:${String(secs%60).padStart(2,'0')}`,'Time']].map(s=>`<div><b>${s[0]}</b><span>${s[1]}</span></div>`).join('');
  $('#again').textContent=NET.on&&!NET.host?'Back to class select':'New run';
  try{const best=JSON.parse(localStorage.getItem('emberwild-best')||'null');if(!best||days>best.days||(days===best.days&&P.kills>best.kills))localStorage.setItem('emberwild-best',JSON.stringify({days,cls:C.name,lvl:P.lvl,kills:P.kills}));}catch(e){}
  setTimeout(()=>$('#again').focus(),50);}
$('#again').onclick=()=>{$('#death').hidden=true;if(NET.on&&!NET.host){toMenu(false);coopStatus('Waiting for the host to start a new world.');return;}toMenu(true);if(NET.on&&NET.host)send({t:'newworld',seed:G.seed});};
$('#reroll').onclick=()=>{newWorld((Math.random()*1e9)>>>0);if(NET.on&&NET.host)send({t:'newworld',seed:G.seed});};
function toMenu(fresh){if(P.model){scene.remove(P.model.root);P.model=null;}for(const e of ents)removeEnt(e);ents=[];for(const p of pickups)scene.remove(p.mesh);pickups=[];for(const c of chests)scene.remove(c.mesh);chests=[];
  for(const t of traps)scene.remove(t.mesh);traps=[];for(const p of projs)p.mesh&&scene.remove(p.mesh);projs=[];for(const z of zones)z.cleanup&&z.cleanup();zones=[];timers=[];for(const d of decoys)scene.remove(d.model.root);decoys=[];
  clearRelics(P);clearCorpses();G.trial=null;
  G.state='menu';G.ui=null;G.over=false;$('#menu').hidden=false;$('#hud').hidden=true;if(fresh)newWorld((Math.random()*1e9)>>>0);buildMenu();}
function checkPartyWipe(dt){if(!NET.on||!NET.host||G.over||G.state!=='playing')return;checkPartyWipe.t=(checkPartyWipe.t||0)-dt;if(checkPartyWipe.t>0)return;checkPartyWipe.t=0.5;
  let alive=P.dead?0:1;for(const r of NET.peers.values())if(r.inGame&&!r.dead)alive++;if(alive===0){G.over=true;send({t:'over'});setTimeout(showDeath,1200);}}

/* ---------------- main loop ---------------- */
let last=performance.now(),camT=new V3();
function loop(now){requestAnimationFrame(loop);frame(now);}
function frame(now){
  let dt=Math.min(0.05,(now-last)/1000);if(dt<=0)return;last=now;ACTOR=P;
  let sdt=dt;if(G.hitstop>0&&!NET.on){G.hitstop-=dt;sdt=dt*0.08;}else G.hitstop=0;if(P.dead&&!NET.on)sdt*=0.5;
  timeU.value+=dt;waterMat.uniforms.uTime.value=timeU.value;
  const sim=G.state==='playing'&&(!G.ui||NET.on);
  if(sim){
    G.runT+=sdt;updatePlayer(sdt);updateAim();updateRemotes(sdt);updateEnts(sdt);updateProjs(sdt);updateZones(sdt);updatePickups(sdt);
    if(AUTH())updateDirector(sdt);updateTrial(sdt);updateRelics(sdt);augUpdate(sdt);updateCorpses(sdt);
    updatePOIs(sdt);findInteract();checkPartyWipe(sdt);
    if(NET.on){sendState(dt);if(NET.host)sendSnap(dt);}
    camT.copy(P.pos);ambientFx(sdt);autoQuality(dt);
  }else if(G.state==='menu'){menuCamera(dt);camT.copy(menuCenter);if(Math.random()<0.5)ambientFx(dt,menuCenter);if(NET.on)updateRemotes(dt);}
  if(G.state==='playing'){updateCamera(dt);updateHUD(dt);updateBars();updateMinimap(dt);}
  updateDay(sim?sdt:(G.state==='menu'&&!NET.on?dt*0.6:0),camT);
  if(sim||G.state!=='playing'){updateFx(sdt);PA.update(sdt);PN.update(sdt);DB.update(sdt);}
  updateText(dt);updateMusic(dt);
  if(G.flash>0){G.flash=Math.max(0,G.flash-dt);$('#flash').style.opacity=G.flash*1.6;}
  updateChunks(camT.x,camT.z,G.state==='playing'?2:3);
  water.position.x=Math.round(camT.x/8)*8;water.position.z=Math.round(camT.z/8)*8;
  sky.position.copy(camera.position);stars.position.copy(camera.position);stars.rotation.y=G.time*TAU*0.5;
  const cd=clouds.userData.d,co=clouds.userData.o;for(let i=0;i<cd.length;i++){const c=cd[i];c.x+=dt*1.6;const rx=((c.x-camT.x+300)%600+600)%600-300,rz=((c.z-camT.z+300)%600+600)%600-300;
    co.position.set(camT.x+rx+c.ox,c.y,camT.z+rz+c.oz);co.scale.set(c.sx,c.sy,c.sz);co.updateMatrix();clouds.setMatrixAt(i,co.matrix);}clouds.instanceMatrix.needsUpdate=true;
  if(!document.hidden&&!frame.noRender)renderFrame();
}
/* keep the host's world running while its window is in the background */
setInterval(()=>{if(document.hidden&&NET.on)frame(performance.now());},50);
function ambientFx(dt,center){const c=center||P.pos;
  if(G.night){for(let i=0;i<2;i++)if(Math.random()<0.5){const x=c.x+rand(-22,22),z=c.z+rand(-22,22);const t=world.topAt(Math.floor(x),Math.floor(z));if(t>SEA){PA.emit(x,t+rand(1,3),z,rand(-0.4,0.4),rand(-0.2,0.2),rand(-0.4,0.4),rand(2,4),0.22,0.22,col(G.blood?0xff5a3a:0xd8ff6a),0,0.2,1);}}}
  else if(Math.random()<0.5){const x=c.x+rand(-14,14),z=c.z+rand(-14,14);const t=world.topAt(Math.floor(x),Math.floor(z));if(t>SEA)PA.emit(x,t+rand(1,3.5),z,rand(-0.3,0.3),rand(-0.1,0.2),rand(-0.3,0.3),rand(3,5),0.12,0.12,col(0xfff2b0),0,0.1,0.6);}
  if(!G.night&&Math.random()<0.25){const x=c.x+rand(-20,20),z=c.z+rand(-20,20);const t=world.topAt(Math.floor(x),Math.floor(z));const b=world.get(Math.floor(x),t,Math.floor(z));
    if(b===B.LEAVES||b===B.AUTUMN)PN.emit(x,t,z,rand(0.2,0.8),-0.6,rand(-0.3,0.3),4,0.3,0.25,col(b===B.AUTUMN?0xd4772a:0x4f9a35),0.3,0.3,0.9);}}
addEventListener('resize',()=>{renderer.setSize(innerWidth,innerHeight);camera.aspect=innerWidth/innerHeight;camera.updateProjectionMatrix();PA.pts.material.uniforms.pr.value=PN.pts.material.uniforms.pr.value=renderer.getPixelRatio();});

/* ---------------- boot ---------------- */
function boot(){
  try{const g=localStorage.getItem('emberwild-gfx');if(g){$('#gfx').value=g;if(g!=='auto'){Q.mode='fixed';applyQuality(+g);}}}catch(e){}
  newWorld((Math.random()*1e9)>>>0);buildMenu();$('#loading').hidden=true;requestAnimationFrame(t=>{last=t;loop(t);});
}
window.__game={G,P,NET,Q,get world(){return world;},get ents(){return ents;},get projs(){return projs;},get pickups(){return pickups;},startRun,useAbility,spawnEnt,swapWeapon,newWorld,gainXp,openPerks,mouse,keys,applyQuality,
  netConnect,send,render:()=>{if(G.state==='playing')updateCamera(0);renderFrame();},POST,tick:(n,dt=1/60)=>{frame.noRender=true;try{for(let i=0;i<n;i++)frame(last+dt*1000);}finally{frame.noRender=false;}}};
setTimeout(boot,30);
