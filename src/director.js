'use strict';
/* Emberwild director: the horde. Instead of monsters living in the world, the director keeps a crowd of them
   hunting you: it spawns packs in a ring just around the players, raises the headcount and unlocks tougher
   kinds as the minutes pass, throws horde events and bosses at you, scatters loot urns, runs altar trials,
   and scores the music to how hot the fight is. Only the host (or a solo player) runs the spawning. */

const D={pack:1.5,event:70,boss:300,urn:2,animal:3,evI:0};
function resetDirector(){Object.assign(D,{pack:1.5,event:70,boss:300,urn:2,animal:3,evI:0});}
function minutes(){return G.runT/60;}
function hostileCap(){return[70,100,CAP.hostile][Q.lvl];}
const THREAT=[[0,'Calm','#9fd8ff'],[3,'Restless','#c8e070'],[6,'Dangerous','#f0c64a'],[10,'Deadly','#ff8a3d'],[15,'Nightmare','#ff4a3a'],[21,'Apocalypse','#d04aff']];
function threatName(){const m=minutes();let t=THREAT[0];for(const q of THREAT)if(m>=q[0])t=q;return[t[1],t[2]];}
/* how many hostiles should be hunting right now */
function hordeTarget(){const m=minutes();let n=6+m*6.5+(G.day-1)*3;if(G.night)n*=1.3;if(G.blood)n*=1.4;n*=1+0.45*(playerCount()-1);if(G.trial)n+=10;return Math.min(n,hostileCap());}
/* a spawn point on the ring around a player; spots in front of them are pushed further out so the horde rises at the edge of sight */
function ringSpot(a,r0,r1){const fx_=Math.sin(a.yaw||0),fz=Math.cos(a.yaw||0);
  for(let k=0;k<10;k++){const an=rand(0,TAU);let d=rand(r0,r1);if(Math.cos(an)*fx_+Math.sin(an)*fz>0.6)d+=8;
    const s=surfaceSpot(a.pos.x+Math.cos(an)*d,a.pos.z+Math.sin(an)*d);if(s&&Math.abs(s.y-a.pos.y)<14&&!world.poiNear(s.x,s.z,2))return s;}return null;}
/* [type, minute it starts appearing, weight]: new kinds of monster join the horde as the run goes on */
const ROSTER=[['zombie',0,6],['slime',0,4],['bat',1.5,3.5],['skeleton',2.5,2.4],['spider',3.5,3],['bloater',4.5,2.2],['cultist',6,2],['brute',8,1.4]];
const PACK={zombie:[2,5],slime:[2,4],bat:[4,8],skeleton:[1,3],spider:[2,3],bloater:[1,2],cultist:[1,2],brute:[1,1]};
function pickType(b){const m=minutes();const opts=[];let tot=0;
  for(const[t,at,w]of ROSTER){if(m<at)continue;let k=w*Math.min(1,0.35+(m-at)*0.4);
    if(t==='skeleton'&&b===BIO.DESERT)k*=2;if(t==='spider'&&b===BIO.FOREST)k*=2;if(t==='cultist'&&(b===BIO.SNOW||b===BIO.PEAK))k*=1.8;if(t==='slime'&&(b===BIO.PLAINS||b===BIO.FOREST))k*=1.4;
    if(G.night&&(t==='zombie'||t==='skeleton'||t==='bat'))k*=1.5;opts.push([t,k]);tot+=k;}
  let r=Math.random()*tot;for(const o of opts){r-=o[1];if(r<=0)return o[0];}return'zombie';}
function slimeOpts(b){return{size:Math.random()<0.3?1.6:1,color:b===BIO.DESERT?0xe0b85a:b===BIO.SNOW?0x8fd8ff:pick([0x6fd46a,0x6fd46a,0xff7a9a])};}
function spawnPack(a,type,n,o={}){const s=o.at||ringSpot(a,o.r0||26,o.r1||36);if(!s)return 0;const b=world.column(Math.floor(s.x),Math.floor(s.z)).b;let k=0;
  for(let i=0;i<n;i++){const sp=surfaceSpot(s.x+rand(-2.5,2.5),s.z+rand(-2.5,2.5))||s;const eo={force:o.force};
    if(type==='slime')Object.assign(eo,slimeOpts(b));if(i===0&&o.elite)eo.affix=pick(AFFIX);
    if(spawnEnt(type,sp.x,type==='bat'?sp.y+rand(2,4):sp.y,sp.z,eo))k++;}
  if(k&&Math.random()<0.25)sfx('rise');return k;}
function announce(a,b){efx('toast',{a,b});}
function updateDirector(dt){
  const tg=avatars().filter(a=>!a.dead);if(!tg.length)return;
  let hostile=0,animals=0,urns=0,ranged=0;for(const e of ents){if(e.T.prop)urns++;else if(e.passive)animals++;else if(!e.poi){hostile++;if(e.T.ranged)ranged++;}}
  const m=minutes();
  D.pack-=dt;if(D.pack<=0){D.pack=Math.max(0.3,1.0-m*0.05);const want=hordeTarget();
    if(hostile<want){const a=pick(tg);const b=world.column(Math.floor(a.pos.x),Math.floor(a.pos.z)).b;let type=pickType(b);if(ETYPES[type].ranged&&ranged>want*0.22)type=pick(['zombie','spider','slime']);const[lo,hi]=PACK[type];
      const n=Math.min(Math.ceil(want-hostile),randi(lo,hi)+Math.floor(m/4));spawnPack(a,type,n,{elite:Math.random()<clamp(0.02+m*0.012+(G.blood?0.08:0),0,0.3)});}}
  D.event-=dt;if(D.event<=0){D.event=rand(75,100)*Math.max(0.6,1-m*0.02);hordeEvent(D.evI++,pick(tg));}
  D.boss-=dt;if(D.boss<=0){D.boss=300;spawnBoss(pick(tg),'An Ancient Golem awakens','Bring it down for a relic');}
  D.urn-=dt;if(D.urn<=0){D.urn=6;if(urns<4){const s=ringSpot(pick(tg),14,34);if(s)spawnEnt('urn',s.x,s.y,s.z);}}
  D.animal-=dt;if(D.animal<=0){D.animal=4;if(animals<8){const s=ringSpot(pick(tg),22,40);
    if(s){const tb=world.get(Math.floor(s.x),Math.floor(s.y)-1,Math.floor(s.z));if(tb===B.GRASS||tb===B.FGRASS||tb===B.SNOW){const type=tb===B.SNOW?'sheep':pick(['pig','pig','sheep','chicken','chicken']);
      for(let k=0;k<randi(2,4);k++){const sp=surfaceSpot(s.x+rand(-3,3),s.z+rand(-3,3));if(sp)spawnEnt(type,sp.x,sp.y,sp.z);}}}}}
}
/* set pieces that break up the steady stream */
function hordeEvent(i,a){const m=minutes();sfx('horn');
  if(i%3===0){// encircled: a full ring of the dead rising all around you at once
    const n=Math.min(44,14+Math.floor(m*3)),r=rand(19,23);announce('The horde closes in!','They rise all around you');
    for(let k=0;k<n;k++){const an=k/n*TAU;const s=surfaceSpot(a.pos.x+Math.cos(an)*r,a.pos.z+Math.sin(an)*r);if(!s)continue;const type=m>5&&k%4===0?'skeleton':m>8&&k%7===0?'bloater':'zombie';spawnEnt(type,s.x,s.y,s.z,{force:true});}}
  else if(i%3===1){// a shrieking swarm from one direction
    const n=Math.min(40,10+Math.floor(m*2.5));const an=rand(0,TAU);announce('A swarm descends!','Bats pour in from the '+['east','south','west','north'][Math.round(an/(TAU/4))%4]);
    for(let k=0;k<n;k++){const d=rand(26,34),sp=an+rand(-0.35,0.35);const s=surfaceSpot(a.pos.x+Math.cos(sp)*d,a.pos.z+Math.sin(sp)*d);if(s)spawnEnt('bat',s.x,s.y+rand(3,6),s.z,{force:true});}}
  else{// an elite war band
    announce('Elites approach','A war band hunts you');const lead=m>8?'brute':m>4?pick(['zombie','spider']):'zombie';
    spawnPack(a,lead,1+Math.floor(m/6),{elite:true,force:true,r0:24,r1:30});spawnPack(a,lead==='brute'?'zombie':lead,4+Math.floor(m/2),{elite:true,force:true,r0:24,r1:30});}}
function spawnBoss(a,t,sub){const s=ringSpot(a,26,32);if(!s)return;if(spawnEnt('golem',s.x,s.y,s.z,{boss:true,force:true})){announce(t,sub);sfx('boom');G.shake=0.6;}}

/* ---------------- altar trials ---------------- */
function beginTrial(p,by){if(p.trial)return;p.used=true;p.trial={t:35,spawn:1,mine:!by};G.trial=p;
  toast('The altar awakens',by?by+' began a trial. Survive 35s near the altar.':'Survive 35 seconds near the altar',3);sfx('horn');
  const c=new V3(p.x+0.5,p.y+1,p.z+0.5);fx.pillar(c,0xff3a4a,2,30,1.5);fx.ring(c,0xff3a4a,0.5,12,0.8);fx.shock(c,0xff3a4a,0.5,12,0.6,3);G.shake=Math.min(1,G.shake+0.5);
  if(NET.on&&!NET.host&&!by)send({t:'trial',to:NET.hostId,gx:p.gx,gz:p.gz});}
function updateTrial(dt){const p=G.trial;if(!p)return;const T=p.trial;if(!T){G.trial=null;return;}T.t-=dt;const c=new V3(p.x+0.5,p.y+1,p.z+0.5);
  if(Math.random()<0.7){const an=rand(0,TAU),r=rand(0,12);PA.emit(c.x+Math.cos(an)*r,c.y,c.z+Math.sin(an)*r,0,rand(2,4),0,0.8,0.4,0.05,col(0xff3a4a),0,0,1);}
  if(AUTH()){T.spawn-=dt;if(T.spawn<=0){T.spawn=2.2;spawnPack({pos:c,yaw:0},pick(['zombie','skeleton','spider','zombie',minutes()>4?'bloater':'slime']),3+Math.floor(minutes()/3),{r0:12,r1:16,force:true,elite:Math.random()<0.3});}}
  if(T.mine&&!P.dead&&Math.hypot(P.pos.x-c.x,P.pos.z-c.z)>40){toast('The trial is broken','You strayed too far from the altar',2.5);G.trial=null;p.trial=null;return;}
  if(T.t<=0){G.trial=null;p.trial=null;if(Math.hypot(P.pos.x-c.x,P.pos.z-c.z)>45)return;toast('Trial complete','The altar yields a relic',3);sfx('chest');
    const cp=new V3(p.x+0.5,p.y+1,p.z+3.5);dropChestAt(cp,{relic:true});fx.pillar(c,0xffd35a,2,30,1.5);dropGems(cp,Math.round(20+minutes()*6));}}

/* ---------------- music ----------------
   A small generative score: a slow minor pad, a pentatonic pluck line, and drums that join in as the horde closes in. */
const MUS={t:0,step:0,bar:0,out:null,heat:0};
const SCALE=[0,3,5,7,10,12,15,17,19];
const mf=s=>110*Math.pow(2,s/12);
function mnote(f,t,dur,type,vol,att=0.01){const c=AU.ctx;const o=c.createOscillator(),g=c.createGain();o.type=type;o.frequency.value=f;g.gain.setValueAtTime(0.0001,t);g.gain.linearRampToValueAtTime(vol,t+att);
  g.gain.exponentialRampToValueAtTime(0.0001,t+dur);o.connect(g);g.connect(MUS.out);o.start(t);o.stop(t+dur+0.05);}
function mkick(t,v){const c=AU.ctx;const o=c.createOscillator(),g=c.createGain();o.frequency.setValueAtTime(130,t);o.frequency.exponentialRampToValueAtTime(38,t+0.18);g.gain.setValueAtTime(v,t);g.gain.exponentialRampToValueAtTime(0.0001,t+0.25);o.connect(g);g.connect(MUS.out);o.start(t);o.stop(t+0.3);}
function mhat(t,v,len=0.05){const c=AU.ctx;const s=c.createBufferSource();s.buffer=AU.noise;const f=c.createBiquadFilter();f.type='highpass';f.frequency.value=7000;const g=c.createGain();g.gain.setValueAtTime(v,t);g.gain.exponentialRampToValueAtTime(0.0001,t+len);
  s.connect(f);f.connect(g);g.connect(MUS.out);s.start(t,Math.random()*0.5);s.stop(t+len+0.02);}
function updateMusic(dt){const c=AU.ctx;if(!c)return;
  if(!MUS.out){MUS.out=c.createGain();MUS.out.gain.value=0.5;const lp=c.createBiquadFilter();lp.type='lowpass';lp.frequency.value=2600;MUS.out.connect(lp);lp.connect(AU.master);}
  if(AU.muted||G.state!=='playing'||document.hidden){MUS.t=0;return;}
  let near=0;for(const e of ents)if(!e.passive&&!e.T.prop&&!e.dead&&Math.hypot(e.pos.x-P.pos.x,e.pos.z-P.pos.z)<22)near++;
  MUS.heat=lerp(MUS.heat,clamp(near/22+(ents.some(e=>e.boss&&!e.dead)?0.5:0),0,1),Math.min(1,dt*0.6));const h=MUS.heat;
  const beat=0.17;if(MUS.t<c.currentTime)MUS.t=c.currentTime+0.05;
  while(MUS.t<c.currentTime+0.3){const s=MUS.step%16,t=MUS.t;const root=[0,-4,-2,-5][MUS.bar%4]+(G.night?-2:0);
    if(s===0){for(const iv of[0,7,10,15])mnote(mf(root+iv-12),t,beat*17,'triangle',0.035,0.9);mnote(mf(root-24),t,beat*16,'sine',0.06,0.4);}
    if(s%2===0&&Math.random()<0.35+h*0.35)mnote(mf(root+pick(SCALE)+(Math.random()<0.3?12:0)),t,0.4,'sine',0.04);
    if(h>0.2&&s%4===0)mkick(t,0.18+h*0.12);
    if(h>0.45&&s%4===2)mhat(t,0.05+h*0.04);
    if(h>0.7&&s%2===1&&Math.random()<0.6)mhat(t,0.03,0.03);
    if(h>0.55&&s%8===4)mnote(mf(root-12),t,0.25,'sawtooth',0.03);
    MUS.step++;if(MUS.step%16===0)MUS.bar++;MUS.t+=beat;}
}
