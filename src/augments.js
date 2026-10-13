'use strict';
/* Chaos augments: level-up picks that don't make one ability stronger but bend the rules of the whole run:
   kills that chain-explode, abilities that cast themselves twice, ricocheting projectiles, black holes,
   lightning on every crit. They stack with each other and snowball, which is the point.
   Ranks live in a.mods as 'x.<id>' and sync to teammates like every other upgrade. */
const AUGS={
  chain:{name:'Chain Reaction',ranks:['Slain enemies have a 15% chance to explode, and explosions can kill more.','+10% chance and bigger blasts.','+10% chance, and blasts set the horde ablaze.']},
  echo:{name:'Echo',ranks:['Abilities have a 20% chance to cast themselves again for free.','+15% chance to echo.']},
  surge:{name:'Arcane Surge',ranks:['Every ability you cast also fires 4 seeking bolts.','+2 bolts.','+2 bolts and +50% bolt damage.']},
  storm:{name:'Stormcaller',ranks:['+5% critical chance, and critical hits call down lightning.','Lightning strikes twice as often and hits 50% harder.']},
  frenzy:{name:'Frenzy',ranks:['Every kill takes 0.15s off all your ability cooldowns.','0.3s per kill.']},
  well:{name:'Gravity Well',ranks:['Every 9s a black hole tears open in the thickest crowd, drags enemies in and detonates.','Every 6s, wider and twice as deadly.']},
  overload:{name:'Overload',ranks:['Every 5th ability you cast unleashes a massive elemental nova.']},
  bounce:{name:'Ricochet',ranks:['Your projectiles bounce to another enemy after a hit.','They bounce twice.']},
  phoenix:{name:'Phoenix Heart',ranks:['The first time you would die, you explode in flame and rise with half your health.']},
  berserk:{name:'Berserker',ranks:['Below half health, deal 40% more damage and attack 25% faster.','80% more damage and 50% faster attacks.']},
};
const AUG_COLOR='#ff4ad8';
const AK=(a,k)=>M(a,'x.'+k);
const CLS_NOVA={ranger:0x9be07a,mage:0xb46aff,warrior:0xff6a4a,priest:0xffd35a};
function augKill(e,c){if(P.dead||G.state!=='playing'||e.passive||e.T.prop)return;
  const ch=AK(P,'chain');if(ch&&Math.random()<0.05+0.1*ch){const p=c.clone(),r=2.4+ch*0.6;
    later(0.1,()=>withActor(P,()=>{sfx('boom');fx.flare(p,0xff4ad8,4+ch,0.3,true);fx.sphere(p,0xff4ad8,0.3,r,0.3);fx.burst(p,0xff7ae8,24,r*2.4,0.45,0.45);fx.sparks(p,0xffffff,8,10);
      aoeEnemies(p,r,q=>{damageEnt(q,Math.round(24*(1+minutes()*0.1)*(1+0.3*(ch-1))),{kb:7,src:p,up:4,col:0xff4ad8,quiet:true});if(ch>=3)status(q,{burn:2});});}));}
  const fr=AK(P,'frenzy');if(fr)for(let i=0;i<4;i++)P.cds[i]=Math.max(0,P.cds[i]-0.15*fr);}
function augCast(id){const a=P;
  const su=AK(a,'surge');if(su){const n=2+2*su,dmg=Math.round(10*(su>=3?1.5:1)*(1+minutes()*0.05));const from=handOf(a);sfx('arcane');
    for(let i=0;i<n;i++){const an=i/n*TAU;seekBolt(a,from.clone(),new V3(Math.cos(an),0.5,Math.sin(an)),dmg,0xff7ae8,{homing:6,size:0.15,speed:22});}}
  const ov=AK(a,'overload');if(ov){a.castN=(a.castN||0)+1;if(a.castN%5===0){const c=a.pos.clone(),col=CLS_NOVA[a.cls]||0xff4ad8,r=9*a.st.area;sfx('bomb');G.shake=1;
    fx.sphere(_t1.set(c.x,c.y+1,c.z).clone(),col,0.5,r,0.5);fx.circle(c,col,r,1,{style:'arcane',spin:5});fx.shock(c,col,0.5,r,0.5,3);fx.flare(_t1.set(c.x,c.y+1.5,c.z),col,12,0.5,true);flashLight(_t1,col,6,30,0.5);
    popText(_t1.set(c.x,c.y+2.6,c.z),'OVERLOAD','crit');aoeEnemies(c,r,e=>{damageEnt(e,Math.round(80*(1+minutes()*0.1)),{kb:12,src:c,up:7,col});status(e,{stun:0.8});});}}
  const ec=AK(a,'echo');if(ec&&!a.echoing&&Math.random()<0.05+0.15*ec)later(0.35,()=>{if(a.dead||G.state!=='playing')return;a.echoing=true;try{a.hand=handOf(a);a.aimPt.copy(aim.point);ACTOR=a;
    if(ABILITY_FN[id](a)!==false){popText(_t1.set(a.pos.x,a.pos.y+2.4,a.pos.z),'ECHO','info');afterimage(a,0xff7ae8,0.5);if(NET.on)send(castMsg('cast',{ab:id}));}}finally{a.echoing=false;}});}
function augCrit(e){const s=AK(P,'storm');if(!s||G.runT<(P.stormT||0))return;P.stormT=G.runT+(s>=2?0.18:0.36);
  lightningStrike(P,e.pos.clone(),Math.round(20*(s>=2?1.5:1)*(1+minutes()*0.06)),1.7,0xd8a8ff);}
/* projectiles that hit something look for the next victim instead of stopping */
function augBounce(p,e){let best=null,bd=12;for(const q of ents){if(q.dead||q.passive||q.T.prop||q===e||p.hit.has(q))continue;const d=Math.hypot(q.pos.x-p.pos.x,q.pos.z-p.pos.z);if(d<bd){bd=d;best=q;}}
  if(!best)return false;const sp=Math.max(18,p.vel.length());p.vel.set(best.pos.x-p.pos.x,best.pos.y+best.h*0.5-p.pos.y,best.pos.z-p.pos.z).setLength(sp);p.age=Math.min(p.age,p.life*0.5);
  fx.sparks(p.pos,0xff7ae8,5,6);return true;}
function augPhoenix(){if(!AK(P,'phoenix')||P.phoenixUsed)return false;P.phoenixUsed=true;P.hp=Math.round(P.maxHp*0.5);P.invuln=2.5;const c=P.pos.clone();ACTOR=P;
  sfx('bomb');G.shake=1;G.flash=0.25;toast('Phoenix Heart','You rise from the ashes',2.5);fx.pillar(c.clone(),0xff7a2a,2.5,26,1.4);fx.sphere(_t1.set(c.x,c.y+1,c.z).clone(),0xff8a3a,0.5,10,0.6);fx.circle(c,0xff6a1a,8,1.6,{style:'fire',spin:3});
  fx.decal(c,'scorch',8,14);fx.burst(_t1.set(c.x,c.y+1,c.z),0xffb03a,90,12,1,0.7,{up:4});aoeEnemies(c,9,e=>{damageEnt(e,Math.round(120*(1+minutes()*0.1)),{kb:14,src:c,up:8,col:0xff8a3a});status(e,{burn:4});});return true;}
function augDmg(a){const b=AK(a,'berserk');return b&&a.local&&a.hp<a.maxHp*0.5?1+0.4*b:1;}
function augAtk(a){const b=AK(a,'berserk');return b&&a.hp<a.maxHp*0.5?1+0.25*b:1;}
/* Gravity Well: a black hole that drags the horde together and then pops */
function augUpdate(dt){const w=AK(P,'well');if(!w||P.dead)return;P.wellT=(P.wellT??3)-dt;if(P.wellT>0)return;P.wellT=w>=2?6:9;
  const fs=foesNear(P.pos,20);if(!fs.length)return;let best=fs[0],bn=-1;for(const e of fs.slice(0,12)){let n=0;for(const q of fs)if(Math.hypot(q.pos.x-e.pos.x,q.pos.z-e.pos.z)<4)n++;if(n>bn){bn=n;best=e;}}
  const c=best.pos.clone();c.y=groundAt(c.x,c.z,c.y+1);const R=(w>=2?8.5:7)*P.st.area;const core=new THREE.Mesh(SPH,new THREE.MeshBasicMaterial({color:0x05020a}));core.position.set(c.x,c.y+1.4,c.z);
  fx.circle(c,0xb04aff,R,1.7,{style:'arcane',spin:-6});sfx('arcane');ACTOR=P;let pull=0;
  addFx(core,1.7,(f,t)=>{core.scale.setScalar(t<0.85?0.3+t*1.1:Math.max(0.01,(1-t)*8));});
  zone({pos:c,r:R,life:1.6,onFrame:(z,dt)=>{for(let i=0;i<4;i++){const an=rand(0,TAU),rr=R*rand(0.5,1);PA.emit(c.x+Math.cos(an)*rr,c.y+rand(0.3,2.4),c.z+Math.sin(an)*rr,-Math.cos(an)*rr*1.8+Math.sin(an)*5,0,-Math.sin(an)*rr*1.8-Math.cos(an)*5,0.5,0.35,0.05,col(pick([0xb04aff,0xff4ad8,0xffffff])),0,0,1);}
      if((pull-=dt)<=0){pull=0.2;let n=0;for(const e of ents){if(e.dead||e.passive||e.T.prop||n>=14)continue;const dx=c.x-e.pos.x,dz=c.z-e.pos.z,d=Math.hypot(dx,dz);if(d<R&&d>0.6){n++;status(e,{push:[dx/d*9,2,dz/d*9],slow:0.5});}}}},
    end:()=>withActor(P,()=>{sfx('bomb');G.shake=Math.min(1,G.shake+0.6);const p=_t1.set(c.x,c.y+1.2,c.z).clone();fx.flare(p,0xd08aff,10,0.45,true);fx.sphere(p,0xb04aff,0.5,R*0.8,0.45);fx.shock(c,0xff4ad8,0.5,R*0.8,0.4,2.5);
      fx.burst(p,0xff7ae8,60,R*2.5,0.6,0.5);aoeEnemies(c,R*0.75,e=>damageEnt(e,Math.round(55*(w>=2?2:1)*(1+minutes()*0.1)),{kb:10,src:c,up:6,col:0xd08aff}));})});}
