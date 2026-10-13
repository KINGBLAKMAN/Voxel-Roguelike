'use strict';
/* Emberwild VFX kit: procedurally drawn textures (magic circles, cracks, scorch, soft glow) and the effect
   primitives the abilities are built from: magic circles, sweeping weapon trails, ground decals, light flares,
   erupting ice and earth spikes, energy shells and light streaks. Everything is additive and HDR-bright so the
   bloom pass makes it glow. */
const TEX=(()=>{
  const mkc=(S,fn)=>{const c=document.createElement('canvas');c.width=c.height=S;const x=c.getContext('2d');fn(x,S);const t=new THREE.CanvasTexture(c);t.anisotropy=4;return t;};
  const rng=mulberry32(91);
  // magic circles: rings, rune ticks, glyphs and a polygram whose point count sets the school of magic
  const circle=(pts,runes)=>mkc(512,(x,S)=>{const c=S/2;x.translate(c,c);x.strokeStyle='#fff';x.fillStyle='#fff';x.shadowColor='#fff';x.shadowBlur=10;
    const ring=(r,w)=>{x.lineWidth=w;x.beginPath();x.arc(0,0,r,0,TAU);x.stroke();};
    ring(240,7);ring(222,3);ring(150,4);ring(136,2);
    for(let i=0;i<72;i++){const a=i/72*TAU,l=i%6===0?16:7;x.lineWidth=i%6===0?4:2;x.beginPath();x.moveTo(Math.cos(a)*222,Math.sin(a)*222);x.lineTo(Math.cos(a)*(222-l),Math.sin(a)*(222-l));x.stroke();}
    x.lineWidth=3;for(let i=0;i<runes;i++){const a=i/runes*TAU;x.save();x.rotate(a);x.translate(0,-186);
      for(let k=0;k<3;k++){x.beginPath();const p=[[rng()*20-10,rng()*24-12],[rng()*20-10,rng()*24-12],[rng()*20-10,rng()*24-12]];x.moveTo(p[0][0],p[0][1]);x.lineTo(p[1][0],p[1][1]);x.lineTo(p[2][0],p[2][1]);x.stroke();}
      x.beginPath();x.arc(0,0,3,0,TAU);x.fill();x.restore();}
    x.lineWidth=5;const step=pts===6?2:pts===5?2:1;x.beginPath();for(let i=0;i<=pts;i++){const a=(i*step)/pts*TAU-Math.PI/2;const px=Math.cos(a)*150,py=Math.sin(a)*150;i?x.lineTo(px,py):x.moveTo(px,py);}x.stroke();
    if(pts===6){x.beginPath();for(let i=0;i<=3;i++){const a=(i*2+1)/6*TAU-Math.PI/2;const px=Math.cos(a)*150,py=Math.sin(a)*150;i?x.lineTo(px,py):x.moveTo(px,py);}x.stroke();}
    ring(54,4);ring(20,6);for(let i=0;i<pts;i++){const a=i/pts*TAU-Math.PI/2;x.beginPath();x.arc(Math.cos(a)*150,Math.sin(a)*150,9,0,TAU);x.fill();}});
  const glow=mkc(128,(x,S)=>{const g=x.createRadialGradient(64,64,0,64,64,64);g.addColorStop(0,'rgba(255,255,255,1)');g.addColorStop(0.25,'rgba(255,255,255,0.55)');g.addColorStop(1,'rgba(255,255,255,0)');x.fillStyle=g;x.fillRect(0,0,S,S);});
  const star=mkc(128,(x,S)=>{x.translate(64,64);for(const[w,l,a]of[[4,62,0],[4,62,Math.PI/2],[2,40,Math.PI/4],[2,40,-Math.PI/4]]){x.save();x.rotate(a);const g=x.createLinearGradient(-l,0,l,0);g.addColorStop(0,'rgba(255,255,255,0)');g.addColorStop(0.5,'rgba(255,255,255,1)');g.addColorStop(1,'rgba(255,255,255,0)');x.fillStyle=g;x.fillRect(-l,-w/2,l*2,w);x.restore();}
    const g=x.createRadialGradient(0,0,0,0,0,26);g.addColorStop(0,'rgba(255,255,255,1)');g.addColorStop(1,'rgba(255,255,255,0)');x.fillStyle=g;x.fillRect(-26,-26,52,52);});
  const crack=mkc(256,(x,S)=>{x.translate(128,128);x.strokeStyle='#fff';x.lineCap='round';
    for(let i=0;i<11;i++){let a=i/11*TAU+rng()*0.4,r=10,px=Math.cos(a)*r,py=Math.sin(a)*r;x.lineWidth=7;x.beginPath();x.moveTo(px,py);
      while(r<120){r+=10+rng()*14;a+=(rng()-0.5)*0.5;const nx=Math.cos(a)*r,ny=Math.sin(a)*r;x.lineTo(nx,ny);x.lineWidth=Math.max(1.5,7-r/20);x.stroke();x.beginPath();x.moveTo(nx,ny);
        if(rng()<0.25){x.save();x.lineWidth=2;const b=a+(rng()-0.5)*1.4;x.lineTo(nx+Math.cos(b)*22,ny+Math.sin(b)*22);x.stroke();x.restore();x.beginPath();x.moveTo(nx,ny);}}}
    const g=x.createRadialGradient(0,0,0,0,0,40);g.addColorStop(0,'rgba(255,255,255,0.9)');g.addColorStop(1,'rgba(255,255,255,0)');x.fillStyle=g;x.fillRect(-40,-40,80,80);});
  const scorch=mkc(256,(x,S)=>{for(let i=0;i<26;i++){const a=rng()*TAU,r=rng()*60,rr=30+rng()*50;const g=x.createRadialGradient(128+Math.cos(a)*r,128+Math.sin(a)*r,0,128+Math.cos(a)*r,128+Math.sin(a)*r,rr);
    g.addColorStop(0,'rgba(255,255,255,0.35)');g.addColorStop(1,'rgba(255,255,255,0)');x.fillStyle=g;x.fillRect(0,0,S,S);}});
  return{arcane:circle(6,12),holy:circle(8,16),nature:circle(5,10),frost:circle(6,18),fire:circle(3,9),war:circle(4,8),glow,star,crack,scorch};
})();
const DECAL=new THREE.PlaneGeometry(2,2);DECAL.rotateX(-Math.PI/2);
const texMat=(map,c,op,blend=THREE.AdditiveBlending)=>new THREE.MeshBasicMaterial({map,color:c,transparent:true,opacity:op,depthWrite:false,blending:blend,side:THREE.DoubleSide,polygonOffset:true,polygonOffsetFactor:-4,polygonOffsetUnits:-4});
/* a sweeping arc whose bright leading edge races round and leaves a fading tail: every melee swing */
const swooshGeos={};
function swooshGeo(r,len){const k=r.toFixed(1)+'_'+len.toFixed(2);if(swooshGeos[k])return swooshGeos[k];const N=28,p=[],uv=[],ix=[];
  for(let i=0;i<=N;i++){const t=i/N,a=-Math.PI/2-len/2+t*len;for(const[j,rr]of[[0,r*0.25],[1,r]]){p.push(Math.cos(a)*rr,0,-Math.sin(a)*rr);uv.push(t,j);}}
  for(let i=0;i<N;i++){const a=i*2;ix.push(a,a+1,a+2,a+1,a+3,a+2);}
  const g=new THREE.BufferGeometry();g.setAttribute('position',new THREE.Float32BufferAttribute(p,3));g.setAttribute('uv',new THREE.Float32BufferAttribute(uv,2));g.setIndex(ix);g.computeBoundingSphere();return swooshGeos[k]=g;}
const swooshMat=c=>new THREE.ShaderMaterial({transparent:true,depthWrite:false,blending:THREE.AdditiveBlending,side:THREE.DoubleSide,uniforms:{c:{value:new THREE.Color(c)},rev:{value:0},o:{value:1}},
  vertexShader:`varying vec2 vU;void main(){vU=uv;gl_Position=projectionMatrix*modelViewMatrix*vec4(position,1.0);}`,
  fragmentShader:`uniform vec3 c;uniform float rev,o;varying vec2 vU;void main(){if(vU.x>rev)discard;float tail=smoothstep(rev-0.75,rev,vU.x);float edge=smoothstep(0.0,0.6,vU.y)*smoothstep(1.0,0.9,vU.y);
    float core=exp(-pow((vU.y-0.86)*10.0,2.0))*smoothstep(rev-0.3,rev,vU.x);float a=tail*tail*edge*o;gl_FragColor=vec4((c*a+vec3(core*o))*1.7,1.0);}`});
Object.assign(fx,{
  arc(pos,yaw,r,len,c,life=0.2,tilt=0){const m=new THREE.Mesh(swooshGeo(r,len),swooshMat(c));m.position.copy(pos);m.rotation.order='YXZ';m.rotation.set(tilt,yaw,0);m.renderOrder=7;
    return addFx(m,life*1.5,(f,t)=>{const u=m.material.uniforms;u.rev.value=Math.min(1.35,t*2.4);u.o.value=t<0.5?1:(1-t)*2;m.scale.setScalar(0.92+t*0.12);});},
  /* a rotating magic circle, flat on the ground or standing up facing dir */
  circle(pos,c,r,life,o={}){o.op=(o.op??0.95)*0.62;const m=new THREE.Mesh(DECAL,texMat(TEX[o.style||'arcane'],c,o.op));m.position.set(pos.x,pos.y+(o.y??0.06),pos.z);m.renderOrder=6;
    if(o.dir){m.lookAt(_c3.copy(m.position).add(o.dir));m.rotateX(Math.PI/2);}const sp=o.spin??0.8,r0=o.grow===false?r:r*0.2;
    return addFx(m,life,(f,t)=>{const k=t<0.15?t/0.15:1;m.scale.setScalar(lerp(r0,r,1-Math.pow(1-k,3)));m.rotateY(sp*0.016);m.material.opacity=(o.op??0.95)*(t<0.8?1:(1-t)/0.2)*(0.85+0.15*Math.sin(f.t*14));if(o.follow){m.position.set(o.follow.pos.x,o.follow.pos.y+(o.y??0.06),o.follow.pos.z);}});},
  /* ground decals: scorch and cracks darken the ground and linger, frost and glow brighten it */
  decal(pos,kind,r,life,c=0x000000){const dark=kind==='scorch'||kind==='crack';const m=new THREE.Mesh(DECAL,texMat(TEX[kind==='frost'?'scorch':kind],dark?0x000000:c,dark?0.75:0.6,dark?THREE.NormalBlending:THREE.AdditiveBlending));
    m.position.set(pos.x,pos.y+0.04,pos.z);m.rotation.y=rand(0,TAU);m.scale.setScalar(r);m.renderOrder=3;
    const glow=kind==='crack'&&c?new THREE.Mesh(DECAL,texMat(TEX.crack,c,1)):null;if(glow){glow.position.y=0.01;m.add(glow);}
    return addFx(m,life,(f,t)=>{m.material.opacity=(dark?0.75:0.6)*(t<0.75?1:(1-t)/0.25);if(glow)glow.material.opacity=Math.max(0,1-t*3);});},
  /* a camera-facing burst of light, the star-shaped kind for crits and impacts */
  flare(pos,c,size,life=0.3,star=false){const s=new THREE.Sprite(new THREE.SpriteMaterial({map:star?TEX.star:TEX.glow,color:c,blending:THREE.AdditiveBlending,depthWrite:false,transparent:true}));s.position.copy(pos);s.renderOrder=9;
    s.material.rotation=rand(0,TAU);const cap=Math.max(0.6,s.position.distanceTo(camera.position)*0.28);
    return addFx(s,life,(f,t)=>{const k=t<0.2?t/0.2:1;s.scale.setScalar(Math.min(size,cap)*(0.4+0.8*k));s.material.opacity=(1-t)*(size>cap?0.7:1);});},
  /* crystals or rock shards punching up out of the ground in a ring, then sinking back */
  spikes(pos,c,n,r,life=0.9,h=1.6,mat){const g=new THREE.Group();g.position.copy(pos);const m=mat||new THREE.MeshLambertMaterial({color:c,emissive:new THREE.Color(c).multiplyScalar(0.35)});const sp=[];
    for(let i=0;i<n;i++){const a=i/n*TAU+rand(-0.15,0.15),rr=r*rand(0.75,1.05);const s=new THREE.Mesh(OCT,m);s.position.set(Math.cos(a)*rr,0,Math.sin(a)*rr);s.rotation.set(rand(-0.35,0.35),rand(0,TAU),rand(-0.35,0.35));
      s.userData.h=h*rand(0.6,1.2);s.userData.w=rand(0.9,1.4);s.castShadow=true;g.add(s);sp.push(s);}
    return addFx(g,life,(f,t)=>{const k=t<0.12?t/0.12:t>0.7?1-(t-0.7)/0.3:1;const e=1-Math.pow(1-Math.min(1,k),3);for(const s of sp){s.scale.set(s.userData.w*e*1.4,s.userData.h*e/0.14*0.5,s.userData.w*e*1.4);s.position.y=s.userData.h*0.5*e-0.2;}});},
  /* an expanding glowing shell: the shove of a big blast */
  sphere(pos,c,r0,r1,life=0.4){const m=new THREE.Mesh(SPH,bubbleMat(c));m.position.copy(pos);m.renderOrder=8;r1=Math.min(r1,Math.max(r0+0.5,pos.distanceTo(camera.position)*0.7));
    return addFx(m,life,(f,t)=>{const e=1-Math.pow(1-t,3);m.scale.setScalar(lerp(r0,r1,e));m.material.uniforms.o.value=(1-t)*(1-t)*0.8;});},
  /* a bright streak of light between two points */
  streak(a,b,c,life=0.25,w=0.18){const g=new THREE.Group();const len=a.distanceTo(b);for(const[cc,ww,op]of[[c,w,0.8],[0xffffff,w*0.35,1]]){const s=new THREE.Mesh(BOXG,basicAdd(cc,op));s.scale.set(ww,ww,len);g.add(s);}
    g.position.copy(a).lerp(b,0.5);g.lookAt(b);g.renderOrder=7;return addFx(g,life,(f,t)=>{g.children.forEach((s,i)=>{s.material.opacity=(i?1:0.8)*(1-t);s.scale.x=s.scale.y=(i?w*0.35:w)*(1-t*0.7);});});},
});
const _c3=new V3();
