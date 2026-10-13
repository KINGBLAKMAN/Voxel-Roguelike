'use strict';
/* Relics: auto-firing weapons any class can pick up on level-up (up to 4 at once, 5 ranks each), so a build keeps
   fighting on its own while you aim your abilities. They run per actor like abilities do: teammates see each
   other's relics, and only the local player's relics deal real damage. Ranks live in a.mods as 'r.<id>' so they
   sync to teammates with the other upgrades. */
const RELICS={
  blades:{name:'Whirling Blades',icon:'blades',color:'#cfe6ff',ranks:['Two blades orbit you and cut whatever they touch.','+1 blade.','+35% blade damage.','+1 blade and a wider orbit.','+1 blade, and they spin 40% faster.']},
  thunder:{name:'Thunder Call',icon:'bolt',color:'#9fd8ff',ranks:['Lightning strikes a nearby enemy every 2.5s.','+1 strike per volley.','Strikes come 30% more often.','+1 strike and +30% damage.','Every strike chains to two more enemies.']},
  aura:{name:'Ember Aura',icon:'aura',color:'#ff9a4a',ranks:['Embers sear every enemy within 3m of you.','+20% radius.','+40% damage.','The aura also slows what it burns.','+25% radius and +40% damage.']},
  wisp:{name:'Spirit Wisp',icon:'wisp',color:'#b8f0ff',ranks:['A wisp follows you and fires at the nearest enemy.','Fires 35% faster.','Fires two bolts at a time.','+40% damage.','A second wisp joins, and bolts pierce.']},
  axe:{name:'Ricochet Axe',icon:'axe',color:'#e8c890',ranks:['Hurl a spinning axe that flies out and comes back.','+1 axe.','+35% damage and bigger axes.','+1 axe.','Throw 35% more often.']},
  flask:{name:'Firebomb Flask',icon:'flask',color:'#ff6a3a',ranks:['Lob a flask at the horde that leaves a burning pool.','+1 flask.','Pools are 30% larger and last longer.','+1 flask and +30% damage.','Pools burn hotter and slow.']},
  frost:{name:'Frost Halo',icon:'halo',color:'#9fe6ff',ranks:['A ring of ice shards bursts from you every 4s.','+4 shards.','Shards chill: three chills freeze.','+40% damage.','Bursts come 40% faster and shards pierce.']},
};
const RMAX=5,RSLOTS=4;
const RK=(a,k)=>M(a,'r.'+k);
function relicCount(a){let n=0;for(const k in RELICS)if(RK(a,k))n++;return n;}
function relicState(a){return a.rel||(a.rel={t:{},objs:{},hitT:new Map(),ang:0});}
function clearRelics(a){const s=a&&a.rel;if(!s)return;for(const k in s.objs)for(const o of s.objs[k])scene.remove(o);a.rel=null;}
function relicObjs(s,k,n,make){const L=s.objs[k]||(s.objs[k]=[]);while(L.length<n){const o=make();scene.add(o);L.push(o);}while(L.length>n)scene.remove(L.pop());return L;}
function bladeMesh(){const g=new THREE.Group();const st=cm(0xd8e2ec),gl=cmAdd(0xcfe6ff,0.75),hd=cm(0x4a3a2a);
  mk(g,0.14,0.05,0.9,st,0,0,0.25);mk(g,0.22,0.06,0.18,gl,0,0,0.72);mk(g,0.32,0.07,0.07,cm(0xb08a3a),0,0,-0.22);mk(g,0.06,0.06,0.22,hd,0,0,-0.36);g.children.forEach(q=>q.castShadow=false);return g;}
function axeMesh(big){const g=new THREE.Group();const wd=cm(0x6a4528),st=cm(0xc9d0d8),ed=cmAdd(0xe8f4ff,0.7);
  mk(g,0.07,0.07,0.8,wd,0,0,0);mk(g,0.06,0.42,0.3,st,0,0.12,0.32);mk(g,0.07,0.46,0.06,ed,0,0.12,0.48);g.children.forEach(q=>q.castShadow=false);if(big)g.scale.setScalar(1.35);return g;}
function flaskMesh(){const g=new THREE.Group();mk(g,0.22,0.22,0.22,cm(0xff6a3a,true),0,0,0);mk(g,0.1,0.14,0.1,cm(0xd8e2ec),0,0.17,0);mk(g,0.06,0.06,0.06,cm(0x6a4528),0,0.27,0);g.children.forEach(q=>q.castShadow=false);return g;}
function auraMesh(){const g=new THREE.Group();const r=new THREE.Mesh(RING,basicAdd(0xff8a3a,0.55));const d=new THREE.Mesh(DISK,basicAdd(0xff5a1a,0.09));g.add(r,d);g.renderOrder=6;return g;}
function updateRelics(dt){for(const a of avatars()){const s=a.rel;if(a.dead){if(s)for(const k in s.objs)for(const o of s.objs[k])o.visible=false;continue;}
  if(!s&&!relicCount(a))continue;ACTOR=a;try{relicsFor(a,dt);}finally{ACTOR=P;}}}
function relicsFor(a,dt){const s=relicState(a),cd=a.st.cd,area=a.st.area,t=s.t;for(const k in s.objs)for(const o of s.objs[k])o.visible=true;
  if(s.hitT.size>400)s.hitT.clear();
  // Whirling Blades: orbiting steel that cuts on contact
  let r=RK(a,'blades');const bl=relicObjs(s,'blades',r?[0,2,3,3,4,5][r]:0,bladeMesh);
  if(bl.length){s.ang+=dt*3.1*(r>=5?1.4:1);const R=2.6*(r>=4?1.25:1)*area,dmg=12*(r>=3?1.35:1),n=bl.length;
    for(let i=0;i<n;i++){const an=s.ang+i/n*TAU,x=a.pos.x+Math.cos(an)*R,z=a.pos.z+Math.sin(an)*R,y=a.pos.y+1.05+Math.sin(s.ang*2+i)*0.15;const m=bl[i];m.position.set(x,y,z);m.rotation.set(0,-an,Math.sin(s.ang*3+i)*0.4);
      if(Math.random()<0.5)PA.emit(x,y,z,0,0,0,0.22,0.28,0.04,col(0xcfe6ff),0,0,0.7);
      if(a.local)for(const e of ents){if(e.dead||e.passive)continue;if(Math.hypot(e.pos.x-x,e.pos.z-z)<0.75+e.w&&y>e.pos.y-0.4&&y<e.pos.y+e.h+0.4){const k=s.hitT.get(e)||0;if(G.runT<k)continue;s.hitT.set(e,G.runT+0.42);
        damageEnt(e,dmg,{kb:4,src:a.pos,col:0xcfe6ff});fx.sparks(_t3.set(x,y,z),0xe8f4ff,5,7);}}}}
  // Thunder Call: lightning on random nearby foes
  r=RK(a,'thunder');if(r){t.thunder=(t.thunder??1)-dt;if(t.thunder<=0){t.thunder=2.5*(r>=3?0.7:1)*cd;const fs=foesNear(a.pos,20).slice(0,10);const n=Math.min(fs.length,1+(r>=2)+(r>=4));const dmg=34*(r>=4?1.3:1);
    for(let i=0;i<n;i++){const e=fs.splice(randi(0,fs.length-1),1)[0];const g=lightningStrike(a,e.pos.clone(),dmg,1.9);
      if(r>=5){let from=g.clone().setY(g.y+1);const hit=new Set([e]);for(let j=0;j<2;j++){const nx=foesNear(from,8).find(q=>!hit.has(q));if(!nx)break;hit.add(nx);const to=_t1.set(nx.pos.x,nx.pos.y+nx.h*0.5,nx.pos.z).clone();
        fx.lightning(from,to,0xa8dcff,0.22,0.2);damageEnt(nx,Math.round(dmg*0.6),{kb:2,src:from,col:0xa8dcff});from=to;}}}}}
  // Ember Aura: a burning ring that follows you
  r=RK(a,'aura');const au=relicObjs(s,'aura',r?1:0,auraMesh);
  if(au.length){const R=3*(r>=2?1.2:1)*(r>=5?1.25:1)*area,dmg=5*(r>=3?1.4:1)*(r>=5?1.4:1);const m=au[0];m.position.set(a.pos.x,a.pos.y+0.12,a.pos.z);m.scale.setScalar(R);m.rotation.y+=dt*0.8;
    m.children[0].material.opacity=0.4+Math.sin(G.runT*5)*0.15;
    for(let i=0;i<2;i++){const an=rand(0,TAU),rr=R*rand(0.7,1);PA.emit(a.pos.x+Math.cos(an)*rr,a.pos.y+0.2,a.pos.z+Math.sin(an)*rr,-Math.sin(an)*1.5,rand(1,2.5),Math.cos(an)*1.5,0.6,0.35,0.05,col(pick([0xff6a1a,0xffb03a,0xff3a10])),-1,0.5,1);}
    t.aura=(t.aura??0)-dt;if(t.aura<=0){t.aura=0.5;aoeEnemies(a.pos,R,e=>{if(e.passive)return;damageEnt(e,dmg,{kb:1.2,src:a.pos,col:0xff9a4a,quiet:true});status(e,r>=4?{burn:1.5,slow:0.8}:{burn:1});});}}
  // Spirit Wisp: a companion light that shoots the nearest foe
  r=RK(a,'wisp');const ws=relicObjs(s,'wisp',r?(r>=5?2:1):0,()=>glowBall(0xb8f0ff,0.24));
  if(ws.length){for(let i=0;i<ws.length;i++){const an=G.runT*1.3+i*Math.PI,m=ws[i];const tx=a.pos.x+Math.cos(an)*1.3-Math.sin(a.yaw)*0.6,tz=a.pos.z+Math.sin(an)*1.3-Math.cos(a.yaw)*0.6,ty=a.pos.y+2.3+Math.sin(G.runT*3+i)*0.2;
      m.position.x=lerp(m.position.x||tx,tx,Math.min(1,dt*6));m.position.y=lerp(m.position.y||ty,ty,Math.min(1,dt*6));m.position.z=lerp(m.position.z||tz,tz,Math.min(1,dt*6));
      if(Math.random()<0.6)PA.emit(m.position.x,m.position.y,m.position.z,rand(-0.3,0.3),rand(-0.2,0.4),rand(-0.3,0.3),0.5,0.3,0.03,col(0xb8f0ff),0,1,0.9);}
    t.wisp=(t.wisp??0.5)-dt;if(t.wisp<=0){t.wisp=1.15*(r>=2?0.65:1)*cd;const f=foesNear(a.pos,22);if(f.length){sfx('arcane');
      for(const m of ws)for(let k=0;k<(r>=3?2:1);k++){const e=f[k%f.length];const from=m.position.clone();seekBolt(a,from,_t2.set(e.pos.x-from.x,e.pos.y+e.h*0.5-from.y,e.pos.z-from.z),Math.round(12*(r>=4?1.4:1)),0xb8f0ff,{pierce:r>=5,size:0.16,speed:30});}}}}
  // Ricochet Axe: boomerangs that fly out through the horde and come back
  r=RK(a,'axe');if(r){t.axe=(t.axe??1.2)-dt;if(t.axe<=0){t.axe=2.6*(r>=5?0.65:1)*cd;const n=1+(r>=2)+(r>=4),big=r>=3;const f=foesNear(a.pos,18);sfx('whoosh');
    for(let i=0;i<n;i++){const e=f[i%Math.max(1,f.length)];const d=e?_t2.set(e.pos.x-a.pos.x,0,e.pos.z-a.pos.z).normalize().clone():fwdOf(a,new V3()).applyAxisAngle(UP,i*TAU/n);
      if(e&&i>=f.length)d.applyAxisAngle(UP,(i%2?1:-1)*0.5);const m=axeMesh(big);
      spawnProj({a,pos:new V3(a.pos.x,a.pos.y+1.2,a.pos.z),vel:d.multiplyScalar(20),speed:20,grav:0,r:big?0.9:0.7,life:4,turn:0.55,boomer:a,mesh:m,owner:'p',pierce:true,kind:'axe',noWorld:true,
        trail:p=>{m.rotation.x+=0.45;m.rotation.y=Math.atan2(p.vel.x,p.vel.z);if(Math.random()<0.5)PA.emit(p.pos.x,p.pos.y,p.pos.z,0,0,0,0.2,0.3,0.04,col(0xe8c890),0,0,0.7);},
        onHit:(e,p)=>{damageEnt(e,Math.round(22*(big?1.35:1)),{kb:5,src:p.pos,col:0xe8c890});fx.sparks(p.pos,0xffe0b0,5,7);}});}}}
  // Firebomb Flask: lobbed at the thick of the horde, leaves a burning pool
  r=RK(a,'flask');if(r){t.flask=(t.flask??2)-dt;if(t.flask<=0){t.flask=3.4*cd;const n=1+(r>=2)+(r>=4);const f=foesNear(a.pos,16);
    for(let i=0;i<n;i++){const e=f.length?pick(f.slice(0,8)):null;const tp=e?e.pos.clone():new V3(a.pos.x+rand(-8,8),a.pos.y,a.pos.z+rand(-8,8));const from=new V3(a.pos.x,a.pos.y+1.6,a.pos.z);
      const T=0.75,g=20;const v=new V3((tp.x-from.x)/T,(tp.y-from.y+0.5*g*T*T)/T,(tp.z-from.z)/T);const m=flaskMesh();
      const land=p=>{const c=p.pos.clone();c.y=groundAt(c.x,c.z,c.y+1);sfx('glass');fx.burst(_t1.set(c.x,c.y+0.4,c.z),0xff8a3a,26,6,0.5,0.45,{up:2});DB.burst(_t1,[0xd8e2ec,0xff6a3a],6,4,0.12,5);
        firePool(c,2.2*(r>=3?1.3:1)*area,3+(r>=3?1.5:0),Math.round(7*(r>=4?1.3:1)*(r>=5?1.3:1)),r>=5);};
      spawnProj({a,pos:from,vel:v,grav:g,r:0.3,life:3,mesh:m,owner:'p',kind:'flask',trail:p=>{m.rotation.x+=0.3;m.rotation.z+=0.2;if(Math.random()<0.5)PA.emit(p.pos.x,p.pos.y+0.25,p.pos.z,0,0.5,0,0.3,0.25,0.04,col(0xffb03a),0,0,1);},
        onHit:land,onWorld:land,onExpire:land});}}}
  // Frost Halo: a periodic ring of ice shards
  r=RK(a,'frost');if(r){t.frost=(t.frost??1.5)-dt;if(t.frost<=0){t.frost=4*(r>=5?0.6:1)*cd;const n=10+(r>=2?4:0),dmg=Math.round(11*(r>=4?1.4:1));const c=new V3(a.pos.x,a.pos.y+1,a.pos.z);sfx('ice');
    fx.ring(a.pos,0x9fe6ff,0.5,4,0.35);fx.burst(c,0xbff4ff,20,5,0.4,0.35);
    for(let i=0;i<n;i++){const an=i/n*TAU+s.ang*0.3;const sh=new THREE.Mesh(OCT,cm(0xbff4ff,true));sh.scale.set(0.8,0.8,2.4);
      spawnProj({a,pos:c.clone(),vel:new V3(Math.cos(an),0,Math.sin(an)).multiplyScalar(24),grav:0,r:0.3,life:0.9,mesh:sh,owner:'p',kind:'ice',pierce:r>=5,
        trail:p=>PA.emit(p.pos.x,p.pos.y,p.pos.z,0,0,0,0.25,0.3,0.04,col(0x9fe6ff),0,0,0.8),onHit:(e,p)=>{damageEnt(e,dmg,{kb:2,src:p.pos,col:0x9fe6ff});status(e,r>=3?{slow:2,chill:1,thr:3}:{slow:2});}});}}}
}
