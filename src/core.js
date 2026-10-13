'use strict';
/* Emberwild engine: world generation, meshing, audio, effects, models, class data. */
const V3=THREE.Vector3, TAU=Math.PI*2;
const $=s=>document.querySelector(s);
const clamp=(v,a,b)=>v<a?a:v>b?b:v, lerp=(a,b,t)=>a+(b-a)*t;
const rand=(a,b)=>a+Math.random()*(b-a), randi=(a,b)=>Math.floor(a+Math.random()*(b-a+1));
const pick=a=>a[Math.floor(Math.random()*a.length)];
const smooth=(a,b,x)=>{const t=clamp((x-a)/(b-a),0,1);return t*t*(3-2*t);};
function mulberry32(a){return function(){a|=0;a=a+0x6D2B79F5|0;let t=Math.imul(a^a>>>15,1|a);t=t+Math.imul(t^t>>>7,61|t)^t;return((t^t>>>14)>>>0)/4294967296;};}
function hash2(x,z,s){let h=(Math.imul(x|0,374761393)+Math.imul(z|0,668265263)+Math.imul(s|0,1013904223))|0;h=Math.imul(h^(h>>>13),1274126177);h^=h>>>16;h=Math.imul(h,0x45d9f3b);h^=h>>>15;return(h>>>0)/4294967296;}
function hash3(x,y,z,s){return hash2(x,(z|0)+Math.imul(y|0,198491317),s);}
function hashInt(x,z,s){return(hash2(x,z,s)*4294967296)>>>0;}
function makeNoise(seed){
  const rng=mulberry32(seed),p=new Uint8Array(256);for(let i=0;i<256;i++)p[i]=i;
  for(let i=255;i>0;i--){const j=Math.floor(rng()*(i+1));const t=p[i];p[i]=p[j];p[j]=t;}
  const perm=new Uint8Array(512);for(let i=0;i<512;i++)perm[i]=p[i&255];
  const gx=[1,-1,1,-1,1,-1,0,0],gy=[1,1,-1,-1,0,0,1,-1];
  const F2=0.5*(Math.sqrt(3)-1),G2=(3-Math.sqrt(3))/6;
  return function(xin,yin){
    const s=(xin+yin)*F2,i=Math.floor(xin+s),j=Math.floor(yin+s),t=(i+j)*G2;
    const x0=xin-(i-t),y0=yin-(j-t);const i1=x0>y0?1:0,j1=x0>y0?0:1;
    const x1=x0-i1+G2,y1=y0-j1+G2,x2=x0-1+2*G2,y2=y0-1+2*G2;const ii=i&255,jj=j&255;
    let n=0,tt,g;
    tt=0.5-x0*x0-y0*y0;if(tt>0){g=perm[ii+perm[jj]]&7;tt*=tt;n+=tt*tt*(gx[g]*x0+gy[g]*y0);}
    tt=0.5-x1*x1-y1*y1;if(tt>0){g=perm[ii+i1+perm[jj+j1]]&7;tt*=tt;n+=tt*tt*(gx[g]*x1+gy[g]*y1);}
    tt=0.5-x2*x2-y2*y2;if(tt>0){g=perm[ii+1+perm[jj+1]]&7;tt*=tt;n+=tt*tt*(gx[g]*x2+gy[g]*y2);}
    return 70*n;
  };
}
function fbm(n,x,z,o){let a=1,f=1,s=0,t=0;for(let i=0;i<o;i++){s+=n(x*f,z*f)*a;t+=a;a*=0.5;f*=2.03;}return s/t;}

/* ---------------- constants ---------------- */
const CS=16,CH=64,CS2=CS*CS,SEA=15,RD=6,DAY_LEN=360;
/* hard caps keep big fights from lagging out */
const CAP={ents:160,hostile:130,projs:280,fx:150,pickups:240,stuck:40,zones:44,traps:12,decoys:3};
const Q={mode:'auto',lvl:2,ema:16,badT:0,goodT:0,view:RD};
const BIO={BEACH:0,PLAINS:1,FOREST:2,DESERT:3,SNOW:4,PEAK:5};
const B={AIR:0,GRASS:1,DIRT:2,STONE:3,SAND:4,SNOW:5,WOOD:6,LEAVES:7,PINE:8,CACTUS:9,BRICK:10,MOSS:11,GRAVEL:12,FGRASS:13,BIRCH:14,AUTUMN:15,CRYSTAL:16,BUSH:17,BERRY:18,SPINE:19,PLANK:20,CLOTH:21,GOLD:22,EMBER:23,DSTONE:24,RUNE:25};
const BL=[];
function defB(id,c,o={}){BL[id]={c:new THREE.Color(c),top:o.top!=null?new THREE.Color(o.top):null,side:o.side!=null?new THREE.Color(o.side):null,j:o.j??0.07,tint:!!o.tint};}
defB(B.GRASS,0x5a9638,{top:0x6cb243,side:0x8a5b34,tint:true,j:0.05});defB(B.DIRT,0x8a5b34,{j:0.08});defB(B.STONE,0x8c9097,{j:0.09});
defB(B.SAND,0xe0cd8c,{j:0.05});defB(B.SNOW,0xe4ebf2,{top:0xf5f8fb,side:0x8a6a4a,j:0.03});defB(B.WOOD,0x6a4528,{j:0.08});
defB(B.LEAVES,0x3f8a2f,{tint:true,j:0.13});defB(B.PINE,0x2b5d36,{j:0.1});defB(B.CACTUS,0x4c8b37,{j:0.08});
defB(B.BRICK,0x9f988b,{j:0.12});defB(B.MOSS,0x66804e,{j:0.12});defB(B.GRAVEL,0x837b70,{j:0.15});
defB(B.FGRASS,0x46802e,{top:0x569634,side:0x6e4a2c,tint:true,j:0.05});defB(B.BIRCH,0xe6e2d6,{j:0.1});defB(B.AUTUMN,0xd4772a,{j:0.15});
defB(B.CRYSTAL,0xb98aff,{j:0.15});defB(B.BUSH,0x3a7a2c,{j:0.12});defB(B.BERRY,0x3a7a2c,{j:0.1});
defB(B.SPINE,0x2b5d36,{top:0xeef3f6,j:0.08});defB(B.PLANK,0xa77845,{j:0.08});defB(B.CLOTH,0xb04a3a,{j:0.05});
defB(B.GOLD,0xf0c04a,{j:0.08});defB(B.EMBER,0xff8a2a,{j:0.25});defB(B.DSTONE,0x62666e,{j:0.1});defB(B.RUNE,0xe8344e,{j:0.18});
const BERRYC=new THREE.Color(0xd23a4e);
const BIOME_NAMES=['Shore','Meadow','Forest','Desert','Tundra','Highlands'];

/* ---------------- renderer & scene ---------------- */
const canvas=$('#c');
const renderer=new THREE.WebGLRenderer({canvas,antialias:true,powerPreference:'high-performance'});
renderer.setPixelRatio(Math.min(window.devicePixelRatio||1,1.75));
renderer.setSize(innerWidth,innerHeight);
renderer.shadowMap.enabled=true;renderer.shadowMap.type=THREE.PCFSoftShadowMap;
const scene=new THREE.Scene();
scene.fog=new THREE.Fog(0xa8d4f2,34,RD*CS-6);
const camera=new THREE.PerspectiveCamera(70,innerWidth/innerHeight,0.1,700);
const hemi=new THREE.HemisphereLight(0xcfe7ff,0x4a4a32,0.6);scene.add(hemi);
const sun=new THREE.DirectionalLight(0xfff0d0,0.9);sun.castShadow=true;sun.shadow.mapSize.set(2048,2048);
{const c=sun.shadow.camera;c.left=-46;c.right=46;c.top=46;c.bottom=-46;c.near=1;c.far=280;}
sun.shadow.bias=-0.0006;sun.shadow.normalBias=0.04;scene.add(sun);scene.add(sun.target);
const amb=new THREE.AmbientLight(0xffffff,0.12);scene.add(amb);
const fireLight=new THREE.PointLight(0xff8a3a,0,16,2);scene.add(fireLight);

const skyMat=new THREE.ShaderMaterial({side:THREE.BackSide,depthWrite:false,depthTest:false,fog:false,
  uniforms:{top:{value:new THREE.Color()},hor:{value:new THREE.Color()},sunDir:{value:new V3(0,1,0)},sunCol:{value:new THREE.Color()},night:{value:0},moonCol:{value:new THREE.Color(0xdfe8ff)}},
  vertexShader:`varying vec3 vD;void main(){vD=position;vec4 p=projectionMatrix*modelViewMatrix*vec4(position,1.0);gl_Position=p.xyww;}`,
  fragmentShader:`uniform vec3 top,hor,sunCol,sunDir,moonCol;uniform float night;varying vec3 vD;
  void main(){vec3 d=normalize(vD);float h=clamp(d.y,0.0,1.0);vec3 c=mix(hor,top,pow(h,0.45));
  c=mix(c,hor*1.12+vec3(0.03),exp(-h*14.0)*0.55);
  if(d.y<0.0)c=mix(hor,hor*0.62,clamp(-d.y*4.0,0.0,1.0));
  float s=max(dot(d,sunDir),0.0);c+=sunCol*(smoothstep(0.9985,0.9992,s)*2.5+pow(s,18.0)*0.35+pow(s,4.0)*0.16*exp(-h*3.0));
  float m=max(dot(d,-sunDir),0.0);c+=moonCol*(smoothstep(0.9990,0.9994,m)*1.3+pow(m,40.0)*0.18+pow(m,6.0)*0.05)*night;
  gl_FragColor=vec4(c,1.0);}`});
const sky=new THREE.Mesh(new THREE.SphereGeometry(500,32,16),skyMat);sky.frustumCulled=false;sky.renderOrder=-10;scene.add(sky);
const stars=(()=>{const n=1400,p=new Float32Array(n*3);for(let i=0;i<n;i++){const u=Math.random()*TAU,v=Math.random()*0.95+0.05,r=470;const cv=Math.sqrt(1-v*v);p[i*3]=Math.cos(u)*cv*r;p[i*3+1]=v*r;p[i*3+2]=Math.sin(u)*cv*r;}
  const g=new THREE.BufferGeometry();g.setAttribute('position',new THREE.BufferAttribute(p,3));
  const m=new THREE.PointsMaterial({color:0xffffff,size:1.7,sizeAttenuation:false,transparent:true,opacity:0,fog:false,depthWrite:false});
  const s=new THREE.Points(g,m);s.frustumCulled=false;scene.add(s);return s;})();
/* puffy clouds: clusters of shaded boxes (lit tops, grey bellies) that drift with the wind */
const clouds=(()=>{const g=new THREE.BoxGeometry(1,1,1),nm=g.attributes.normal,cl=[];for(let i=0;i<nm.count;i++){const y=nm.getY(i);const k=y>0.5?1:y<-0.5?0.72:0.88;cl.push(k,k,k*1.02);}
  g.setAttribute('color',new THREE.Float32BufferAttribute(cl,3));
  const d=[];for(let i=0;i<48;i++){const cx=rand(-300,300),cz=rand(-300,300),cy=rand(80,98),w=rand(14,30);const parts=randi(2,4);
    for(let k=0;k<parts;k++)d.push({x:cx,z:cz,ox:k?rand(-w*0.45,w*0.45):0,oz:k?rand(-w*0.3,w*0.3):0,y:cy+(k?rand(0.5,2.5):0),sx:k?w*rand(0.35,0.6):w,sy:k?rand(2,3.5):rand(2.4,3.4),sz:k?w*rand(0.3,0.5):w*rand(0.5,0.8)});}
  const m=new THREE.InstancedMesh(g,new THREE.MeshLambertMaterial({color:0xffffff,vertexColors:true,transparent:true,opacity:0.88,fog:false,depthWrite:false}),d.length);
  m.userData.d=d;m.userData.o=new THREE.Object3D();m.frustumCulled=false;m.renderOrder=-5;scene.add(m);return m;})();

const timeU={value:0};
const waterMat=new THREE.ShaderMaterial({transparent:true,fog:true,depthWrite:false,
  uniforms:THREE.UniformsUtils.merge([THREE.UniformsLib.fog,{uTime:{value:0},uDay:{value:1},uDeep:{value:new THREE.Color(0x17557f)},uShal:{value:new THREE.Color(0x36b0cf)},uSun:{value:new V3(0,1,0)},uSky:{value:new THREE.Color(0xb4dcf5)}}]),
  vertexShader:`uniform float uTime;varying vec3 vW;
  #include <fog_pars_vertex>
  void main(){vec3 p=position;vec4 w=modelMatrix*vec4(p,1.0);w.y+=sin(w.x*0.35+uTime*1.3)*0.06+cos(w.z*0.4+uTime*1.1)*0.06;vW=w.xyz;vec4 mvPosition=viewMatrix*w;gl_Position=projectionMatrix*mvPosition;
  #include <fog_vertex>
  }`,
  fragmentShader:`uniform float uTime,uDay;uniform vec3 uDeep,uShal,uSun,uSky;varying vec3 vW;
  #include <fog_pars_fragment>
  void main(){float r=sin(vW.x*0.9+uTime*1.7)*sin(vW.z*0.8-uTime*1.3)+sin((vW.x+vW.z)*0.45+uTime);
  float r2=sin(vW.x*1.7-uTime*2.1+sin(vW.z*0.6))*0.5+sin(vW.z*1.9+uTime*1.6+sin(vW.x*0.5))*0.5;
  vec3 N=normalize(vec3(cos(vW.x*0.9+uTime*1.7)*0.1+r2*0.07,1.0,cos(vW.z*0.8-uTime*1.3)*0.1-r2*0.06));
  vec3 V=normalize(cameraPosition-vW);float fres=pow(1.0-clamp(dot(N,V),0.0,1.0),3.0);
  vec3 c=mix(uDeep,uShal,0.45+0.16*r+0.06*r2);c=mix(c,uSky,fres*0.6);
  vec3 H=normalize(V+uSun);float sp=pow(max(dot(N,H),0.0),160.0)*step(0.0,uSun.y);
  c+=vec3(1.0,0.95,0.82)*(sp*1.8+smoothstep(1.45,1.95,r)*0.16)*uDay;
  c*=0.35+0.65*uDay;gl_FragColor=vec4(c,mix(0.74,0.94,fres));
  #include <fog_fragment>
  }`});
const water=new THREE.Mesh(new THREE.PlaneGeometry(RD*CS*2+64,RD*CS*2+64,1,1),waterMat);
water.rotation.x=-Math.PI/2;water.position.y=SEA+0.88;water.renderOrder=2;scene.add(water);

/* ---------------- world ---------------- */
class World{
  constructor(seed){
    this.seed=seed>>>0;const s=this.seed;
    this.nC=makeNoise(s);this.nH=makeNoise(s+1);this.nR=makeNoise(s+2);this.nT=makeNoise(s+3);this.nM=makeNoise(s+4);this.nV=makeNoise(s+5);this.nL=makeNoise(s+6);
    this.chunks=new Map();this.poiCache=new Map();this.lk=NaN;this.lc=null;
  }
  /* broad, gently rolling land so hordes can reach you across open ground; lakes in the hollows and real mountains only in rare ranges */
  column(x,z){
    const t=fbm(this.nT,x*0.0021,z*0.0021,2),m=fbm(this.nM,x*0.0024+31.7,z*0.0024-12.3,2);
    const c=fbm(this.nC,x*0.0026,z*0.0026,3),hl=fbm(this.nH,x*0.011,z*0.011,2);
    const rr=1-Math.abs(this.nR(x*0.0052,z*0.0052));const mm=smooth(0.32,0.75,this.nC(x*0.0011+77,z*0.0011-41));
    const lk=smooth(0.3,0.58,fbm(this.nL,x*0.005,z*0.005,2));
    let h=22.5+c*5+hl*1.3+mm*(4+rr*rr*21)-lk*9.5*(1-mm);
    const des=clamp((t-0.24)*5,0,1)*clamp((0.1-m)*5,0,1);h=lerp(h,20.5+c*2.5+hl*0.9,des*0.8);
    h=Math.floor(clamp(h,3,CH-12));
    let b;if(h<=SEA+1)b=BIO.BEACH;else if(h>=39)b=BIO.PEAK;else if(t<-0.24)b=BIO.SNOW;else if(des>0.5)b=BIO.DESERT;else if(m>0.14)b=BIO.FOREST;else b=BIO.PLAINS;
    return{h,b,t,m};
  }
  key(cx,cz){return(cx+32768)*65536+(cz+32768);}
  getChunk(cx,cz){const k=this.key(cx,cz);if(k===this.lk)return this.lc;let c=this.chunks.get(k);if(!c)c=this.gen(cx,cz);this.lk=k;this.lc=c;return c;}
  hasChunk(cx,cz){return this.chunks.has(this.key(cx,cz));}
  get(x,y,z){if(y<0)return B.STONE;if(y>=CH)return 0;const cx=Math.floor(x/CS),cz=Math.floor(z/CS);const c=this.getChunk(cx,cz);return c.d[(x-cx*CS)+(z-cz*CS)*CS+y*CS2];}
  solid(x,y,z){return this.get(Math.floor(x),Math.floor(y),Math.floor(z))!==0;}
  set(x,y,z,b){if(y<1||y>=CH)return;const cx=Math.floor(x/CS),cz=Math.floor(z/CS);const c=this.getChunk(cx,cz);const lx=x-cx*CS,lz=z-cz*CS;c.d[lx+lz*CS+y*CS2]=b;c.dirty=true;
    if(lx===0)this.markDirty(cx-1,cz);if(lx===CS-1)this.markDirty(cx+1,cz);if(lz===0)this.markDirty(cx,cz-1);if(lz===CS-1)this.markDirty(cx,cz+1);this.recalcTop(c,lx,lz);}
  markDirty(cx,cz){const c=this.chunks.get(this.key(cx,cz));if(c)c.dirty=true;}
  recalcTop(c,lx,lz){const i=lx+lz*CS;for(let y=CH-1;y>=0;y--){const b=c.d[i+y*CS2];if(b){c.top[i]=y;c.topB[i]=b;return;}}c.top[i]=0;c.topB[i]=0;}
  topAt(x,z){const cx=Math.floor(x/CS),cz=Math.floor(z/CS);const c=this.getChunk(cx,cz);const i=(x-cx*CS)+(z-cz*CS)*CS;return c.top[i];}
  poi(gx,gz){
    const k=gx*100003+gz;if(this.poiCache.has(k))return this.poiCache.get(k);
    const s=this.seed;let p=null;
    if(hash2(gx,gz,s+101)<0.62&&!(gx===0&&gz===0)){
      const x=gx*64+16+Math.floor(hash2(gx,gz,s+102)*32),z=gz*64+16+Math.floor(hash2(gx,gz,s+103)*32);
      const col=this.column(x,z);
      if(col.h>SEA+2&&col.h<38){const tr=hash2(gx,gz,s+104);const type=tr<0.34?'ruin':tr<0.6?'camp':tr<0.8?'shrine':'altar';
        p={type,x,z,y:col.h,r:type==='shrine'?6:7,gx,gz,active:false,used:false,b:col.b};}
    }
    this.poiCache.set(k,p);return p;
  }
  poiNear(x,z,m){const p=this.poi(Math.floor(x/64),Math.floor(z/64));if(!p)return null;const dx=x-p.x,dz=z-p.z;return dx*dx+dz*dz<(p.r+m)*(p.r+m)?p:null;}
  fillCol(d,lx,lz,h,b,wx,wz){
    const s=this.seed;let top,sub;
    switch(b){case BIO.BEACH:case BIO.DESERT:top=B.SAND;sub=B.SAND;break;case BIO.SNOW:top=B.SNOW;sub=B.DIRT;break;
      case BIO.FOREST:top=B.FGRASS;sub=B.DIRT;break;case BIO.PEAK:top=h>50?B.SNOW:B.STONE;sub=B.STONE;break;default:top=B.GRASS;sub=B.DIRT;}
    if(h<SEA){top=hash2(wx,wz,s+7)<0.28?B.GRAVEL:B.SAND;sub=B.SAND;}
    for(let y=0;y<CH;y++){const i=lx+lz*CS+y*CS2;
      if(y>h){d[i]=0;continue;}
      if(y===h)d[i]=top;else if(y>h-4)d[i]=sub;else d[i]=((y+Math.floor(hash2(wx>>2,wz>>2,s+8)*4))%7===0)?B.DSTONE:B.STONE;}
  }
  gen(cx,cz){
    const d=new Uint8Array(CS2*CH),top=new Uint8Array(CS2),topB=new Uint8Array(CS2),bio=new Uint8Array(CS2),hs=new Int16Array(CS2);
    const ch={cx,cz,d,top,topB,bio,hs,mesh:null,fol:null,dirty:true};
    const x0=cx*CS,z0=cz*CS;
    for(let lz=0;lz<CS;lz++)for(let lx=0;lx<CS;lx++){const wx=x0+lx,wz=z0+lz;const col=this.column(wx,wz);const i=lx+lz*CS;bio[i]=col.b;hs[i]=col.h;this.fillCol(d,lx,lz,col.h,col.b,wx,wz);}
    this.chunks.set(this.key(cx,cz),ch);
    const set=(x,y,z,b,soft)=>{const lx=x-x0,lz=z-z0;if(lx<0||lz<0||lx>=CS||lz>=CS||y<1||y>=CH)return;const i=lx+lz*CS+y*CS2;if(soft&&d[i])return;d[i]=b;};
    // points of interest: flatten then build
    const done=new Set();
    for(let gz=Math.floor((z0-14)/64);gz<=Math.floor((z0+CS+14)/64);gz++)for(let gx=Math.floor((x0-14)/64);gx<=Math.floor((x0+CS+14)/64);gx++){
      const p=this.poi(gx,gz);if(!p||done.has(p))continue;done.add(p);
      const R=p.r,RR=R+4;
      for(let lz=0;lz<CS;lz++)for(let lx=0;lx<CS;lx++){const wx=x0+lx,wz=z0+lz;const dd=Math.hypot(wx-p.x,wz-p.z);if(dd>RR)continue;
        const i=lx+lz*CS;const target=dd<=R?p.y:Math.round(lerp(p.y,hs[i],(dd-R)/4));hs[i]=target;this.fillCol(d,lx,lz,target,bio[i]===BIO.BEACH?BIO.PLAINS:bio[i],wx,wz);}
      this.buildPOI(p,set);
    }
    this.features(ch,set);
    for(let lz=0;lz<CS;lz++)for(let lx=0;lx<CS;lx++)this.recalcTop(ch,lx,lz);
    return ch;
  }
  buildPOI(p,set){
    const s=this.seed,cx=p.x,cz=p.z,h0=p.y;
    if(p.type==='ruin'){
      for(let dz=-6;dz<=6;dz++)for(let dx=-6;dx<=6;dx++){const x=cx+dx,z=cz+dz,r=hash2(x,z,s+201);
        if(Math.abs(dx)<=5&&Math.abs(dz)<=5){if(r>0.12)set(x,h0,z,r<0.42?B.MOSS:B.BRICK);
          const edge=Math.abs(dx)===5||Math.abs(dz)===5;
          if(edge&&!(dx===0||dz===0)){const hh=1+Math.floor(hash2(x,z,s+202)*4.2);if(hash2(x,z,s+203)>0.16)for(let y=1;y<=hh;y++)set(x,h0+y,z,hash3(x,y,z,s+204)<0.35?B.MOSS:B.BRICK);}}
        else if(r<0.08)set(x,h0+1,z,r<0.03?B.MOSS:B.BRICK);}
      for(const[px,pz]of[[-3,-3],[3,-3],[-3,3],[3,3]]){const x=cx+px,z=cz+pz;const hh=2+Math.floor(hash2(x,z,s+205)*5);for(let y=1;y<=hh;y++)set(x,h0+y,z,y===hh&&hh>4?B.DSTONE:B.BRICK);}
      set(cx,h0,cz,B.GOLD);
    }else if(p.type==='camp'){
      for(let dz=-4;dz<=4;dz++)for(let dx=-4;dx<=4;dx++){if(dx*dx+dz*dz<=13&&hash2(cx+dx,cz+dz,s+211)<0.7)set(cx+dx,h0,cz+dz,B.GRAVEL);}
      set(cx,h0+1,cz,B.EMBER);for(const[dx,dz]of[[1,0],[-1,0],[0,1],[0,-1]])set(cx+dx,h0+1,cz+dz,B.WOOD);
      const flip=hash2(p.gx,p.gz,s+212)<0.5;
      for(const side of[-1,1]){for(let l=-2;l<=1;l++)for(let yy=0;yy<3;yy++){for(const sg of yy===2?[0]:[-1,1]){const a=side*5+sg*(2-yy),b=l+(side>0?2:-2);
        const x=flip?cx+b:cx+a,z=flip?cz+a:cz+b;set(x,h0+1+yy,z,B.CLOTH);}}}
      set(cx+3,h0+1,cz-3,B.PLANK);set(cx+3,h0+2,cz-3,B.PLANK);set(cx+4,h0+1,cz-3,B.PLANK);set(cx-3,h0+1,cz+3,B.PLANK);
    }else if(p.type==='altar'){
      // cursed altar: a dark stepped dais, four rune-capped pillars and a glowing heart stone
      for(let dz=-5;dz<=5;dz++)for(let dx=-5;dx<=5;dx++){const d=Math.max(Math.abs(dx),Math.abs(dz));if(d<=4)set(cx+dx,h0,cz+dz,(dx+dz)%2?B.DSTONE:B.BRICK);if(d<=2)set(cx+dx,h0+1,cz+dz,B.DSTONE);}
      for(const[dx,dz]of[[-4,-4],[4,-4],[-4,4],[4,4]]){const hh=3+Math.floor(hash2(cx+dx,cz+dz,s+231)*2);for(let y=1;y<=hh;y++)set(cx+dx,h0+y,cz+dz,y===hh?B.RUNE:B.DSTONE);}
      set(cx,h0+2,cz,B.DSTONE);set(cx,h0+3,cz,B.RUNE);for(const[dx,dz]of[[1,0],[-1,0],[0,1],[0,-1]])set(cx+dx,h0+2,cz+dz,B.BRICK);
    }else{
      for(let dz=-1;dz<=1;dz++)for(let dx=-1;dx<=1;dx++)set(cx+dx,h0,cz+dz,B.BRICK);
      set(cx,h0+1,cz,B.GOLD);
      for(let i=0;i<7;i++){const a=i/7*TAU+hash2(p.gx,p.gz,s+221);const x=cx+Math.round(Math.cos(a)*5),z=cz+Math.round(Math.sin(a)*5);
        const hh=3+Math.floor(hash2(x,z,s+222)*2);for(let y=1;y<=hh;y++)set(x,h0+y,z,hash3(x,y,z,s+223)<0.3?B.MOSS:B.STONE);if(i%2===0)set(x,h0+hh+1,z,B.CRYSTAL);}
    }
  }
  features(ch,set){
    const s=this.seed,x0=ch.cx*CS,z0=ch.cz*CS;
    const grid=(G,M,salt,fn)=>{for(let gz=Math.floor((z0-M)/G);gz<=Math.floor((z0+CS+M)/G);gz++)for(let gx=Math.floor((x0-M)/G);gx<=Math.floor((x0+CS+M)/G);gx++){
      const tx=gx*G+Math.floor(hash2(gx,gz,s+salt+1)*G),tz=gz*G+Math.floor(hash2(gx,gz,s+salt+2)*G);
      if(this.poiNear(tx,tz,3))continue;const col=this.column(tx,tz);if(col.h<=SEA)continue;
      fn(tx,tz,col,hash2(gx,gz,s+salt),mulberry32(hashInt(gx,gz,s+salt+3)));}};
    // boulders
    grid(13,4,300,(x,z,col,r,R)=>{const dens=col.b===BIO.PEAK?0.5:col.b===BIO.DESERT?0.2:0.14;if(r>dens)return;
      const rad=1.1+R()*1.5,mossy=col.b===BIO.FOREST||col.b===BIO.PLAINS;const y0=col.h;
      for(let dy=-2;dy<=3;dy++)for(let dz=-3;dz<=3;dz++)for(let dx=-3;dx<=3;dx++){const dd=dx*dx+dy*dy*1.4+dz*dz;if(dd<=rad*rad+hash3(x+dx,y0+dy,z+dz,s+301)*1.2)
        set(x+dx,y0+dy,z+dz,dy>0&&mossy&&hash3(x+dx,y0+dy,z+dz,s+302)<0.45?B.MOSS:(hash3(x+dx,y0+dy,z+dz,s+303)<0.25?B.DSTONE:B.STONE));}});
    // trees
    // trees: forests open into glades so there is always room to fight
    grid(5,7,400,(x,z,col,r,R)=>{let dens=[0.03,0.05,0.46,0.07,0.24,0.05][col.b];if(col.b===BIO.FOREST||col.b===BIO.PLAINS)dens*=smooth(-0.35,0.25,this.nV(x*0.021+11,z*0.021-7));if(r>dens)return;const y=col.h+1;
      if(col.b===BIO.DESERT){this.cactus(set,x,y,z,R);return;}
      if(col.b===BIO.SNOW||col.b===BIO.PEAK){this.pine(set,x,y,z,R,col.b===BIO.SNOW||col.h>38);return;}
      if(col.b===BIO.BEACH)return;
      const autumn=this.nV(x*0.012,z*0.012)>0.3;const q=R();
      if(col.b===BIO.FOREST&&q<0.1)this.bigTree(set,x,y,z,R,autumn?B.AUTUMN:B.LEAVES);
      else if(q<0.38)this.birch(set,x,y,z,R);else this.oak(set,x,y,z,R,autumn&&R()<0.7?B.AUTUMN:B.LEAVES);});
    // bushes
    grid(4,2,500,(x,z,col,r,R)=>{const dens=[0,0.2,0.28,0.04,0.05,0][col.b];if(r>dens)return;const y=col.h+1;
      const bb=col.b===BIO.DESERT?B.WOOD:(R()<0.5?B.BERRY:B.BUSH);set(x,y,z,bb,true);if(R()<0.5)set(x+(R()<0.5?1:-1),y,z,bb===B.WOOD?B.WOOD:B.BUSH,true);});
    // crystals
    grid(14,3,600,(x,z,col,r,R)=>{if(!(col.b===BIO.PEAK||col.b===BIO.SNOW)||r>0.28)return;const y=col.h+1;
      for(let i=0;i<4;i++){const dx=Math.floor(R()*3)-1,dz=Math.floor(R()*3)-1,hh=1+Math.floor(R()*4);for(let k=0;k<hh;k++)set(x+dx,y+k,z+dz,B.CRYSTAL,true);}});
  }
  leafBlob(set,x,y,z,rx,ry,leaf,salt){const s=this.seed;const ix=Math.ceil(rx),iy=Math.ceil(ry);
    for(let dy=-iy;dy<=iy;dy++)for(let dz=-ix;dz<=ix;dz++)for(let dx=-ix;dx<=ix;dx++){const dd=(dx*dx+dz*dz)/(rx*rx)+(dy*dy)/(ry*ry);if(dd>1.05)continue;
      if(dd>0.6&&hash3(x+dx,y+dy,z+dz,s+salt)<0.22)continue;set(x+dx,y+dy,z+dz,leaf,true);}}
  oak(set,x,y,z,R,leaf){const h=4+Math.floor(R()*3);this.leafBlob(set,x,y+h-1,z,2.6,2.1,leaf,401);for(let i=0;i<h;i++)set(x,y+i,z,B.WOOD);}
  birch(set,x,y,z,R){const h=6+Math.floor(R()*3);this.leafBlob(set,x,y+h-1,z,1.9,2.6,B.LEAVES,402);for(let i=0;i<h;i++)set(x,y+i,z,B.BIRCH);}
  bigTree(set,x,y,z,R,leaf){const h=8+Math.floor(R()*3);this.leafBlob(set,x,y+h,z,4.2,3,leaf,403);this.leafBlob(set,x+2,y+h-3,z-1,2.2,1.6,leaf,404);
    for(let i=0;i<h+1;i++){set(x,y+i,z,B.WOOD);set(x+1,y+i,z,B.WOOD);set(x,y+i,z+1,B.WOOD);set(x+1,y+i,z+1,B.WOOD);}
    set(x-1,y,z,B.WOOD);set(x+2,y,z+1,B.WOOD);set(x,y,z+2,B.WOOD);}
  pine(set,x,y,z,R,snowy){const h=6+Math.floor(R()*4);const L=snowy?B.SPINE:B.PINE;
    for(let k=0;k<h-1;k++){const yy=y+h-k;const r=Math.min(3,(k%3===0?0.6:1)+k*0.42);if(k<1){set(x,yy,z,L,true);continue;}
      for(let dz=-3;dz<=3;dz++)for(let dx=-3;dx<=3;dx++){if(dx*dx+dz*dz<=r*r)set(x+dx,yy,z+dz,L,true);}}
    set(x,y+h+1,z,L,true);for(let i=0;i<h;i++)set(x,y+i,z,B.WOOD);}
  cactus(set,x,y,z,R){const h=2+Math.floor(R()*3);for(let i=0;i<h;i++)set(x,y+i,z,B.CACTUS);
    if(h>2&&R()<0.6){const sx=R()<0.5?1:-1;set(x+sx,y+1,z,B.CACTUS);set(x+sx,y+2,z,B.CACTUS);}}
  spawnPoint(){
    for(let r=0;r<400;r+=3)for(let a=0;a<TAU;a+=0.5){const x=Math.round(Math.cos(a)*r),z=Math.round(Math.sin(a)*r);const c=this.column(x,z);
      if((c.b===BIO.PLAINS||c.b===BIO.FOREST)&&c.h>SEA+2&&!this.poiNear(x,z,4)){const t=this.topAt(x,z);if(t>SEA+1&&this.get(x,t+1,z)===0&&this.get(x,t+2,z)===0)return new V3(x+0.5,t+1,z+0.5);}}
    return new V3(0.5,40,0.5);
  }
}

/* ---------------- meshing ---------------- */
const chunkMat=new THREE.MeshLambertMaterial({vertexColors:true});
const folMat=new THREE.MeshLambertMaterial({vertexColors:true});
folMat.onBeforeCompile=sh=>{sh.uniforms.uTime=timeU;sh.vertexShader='attribute float sway;\nuniform float uTime;\n'+sh.vertexShader.replace('#include <begin_vertex>',
  '#include <begin_vertex>\ntransformed.x+=sin(uTime*1.8+position.x*0.7+position.z*0.4)*0.12*sway;\ntransformed.z+=cos(uTime*1.4+position.x*0.3+position.z*0.6)*0.08*sway;');};
const FACES=[
  {d:[1,0,0],c:[[1,0,0],[1,1,0],[1,1,1],[1,0,1]],s:0.8},
  {d:[-1,0,0],c:[[0,0,1],[0,1,1],[0,1,0],[0,0,0]],s:0.8},
  {d:[0,1,0],c:[[0,1,1],[1,1,1],[1,1,0],[0,1,0]],s:1.0},
  {d:[0,-1,0],c:[[0,0,0],[1,0,0],[1,0,1],[0,0,1]],s:0.5},
  {d:[0,0,1],c:[[1,0,1],[1,1,1],[0,1,1],[0,0,1]],s:0.68},
  {d:[0,0,-1],c:[[0,0,0],[0,1,0],[1,1,0],[1,0,0]],s:0.68}];
for(const F of FACES){const a=F.d[0]?0:F.d[1]?1:2,u=(a+1)%3,v=(a+2)%3;
  F.ao=F.c.map(c=>{const su=c[u]?1:-1,sv=c[v]?1:-1;const o1=F.d.slice(),o2=F.d.slice(),o3=F.d.slice();o1[u]+=su;o2[v]+=sv;o3[u]+=su;o3[v]+=sv;return[o1,o2,o3];});}
const AOF=[0.48,0.64,0.81,1];
function meshChunk(ch){
  const x0=ch.cx*CS,z0=ch.cz*CS,d=ch.d;let maxY=0;for(let i=0;i<CS2;i++)if(ch.top[i]>maxY)maxY=ch.top[i];
  const P_=[],N_=[],C_=[],I_=[];let vc=0;const ao=[0,0,0,0];
  // neighbours for the 1-voxel border
  const nb={};for(const[dx,dz]of[[-1,0],[1,0],[0,-1],[0,1],[-1,-1],[1,-1],[-1,1],[1,1]])nb[dx+','+dz]=world.getChunk(ch.cx+dx,ch.cz+dz).d;
  const gl=(x,y,z)=>{if(y<0)return 1;if(y>=CH)return 0;let ox=0,oz=0;if(x<0){ox=-1;x+=CS;}else if(x>=CS){ox=1;x-=CS;}if(z<0){oz=-1;z+=CS;}else if(z>=CS){oz=1;z-=CS;}
    const dd=(ox|oz)?nb[ox+','+oz]:d;return dd[x+z*CS+y*CS2];};
  for(let y=0;y<=maxY;y++)for(let z=0;z<CS;z++)for(let x=0;x<CS;x++){
    const b=d[x+z*CS+y*CS2];if(!b)continue;const def=BL[b];const wx=x0+x,wz=z0+z;
    const jit=1+(hash3(wx,y,wz,5)-0.5)*def.j*2;let tint=1;if(def.tint)tint=1+world.nV(wx*0.05,wz*0.05)*0.14;
    const glow=b===B.EMBER||b===B.CRYSTAL||b===B.GOLD||b===B.RUNE;
    // grass and snow wrap over the top edge of a block's sides and fade into dirt below
    const capped=def.side&&!gl(x,y+1,z);
    for(let f=0;f<6;f++){const F=FACES[f];if(gl(x+F.d[0],y+F.d[1],z+F.d[2]))continue;
      let col=(f===2&&def.top)?def.top:def.c;if(b===B.BERRY&&hash3(wx*3+f,y,wz,9)<0.5)col=BERRYC;
      const wrap=capped&&f!==2&&f!==3;
      for(let k=0;k<4;k++){const o=F.ao[k];const s1=gl(x+o[0][0],y+o[0][1],z+o[0][2])?1:0,s2=gl(x+o[1][0],y+o[1][1],z+o[1][2])?1:0,s3=gl(x+o[2][0],y+o[2][1],z+o[2][2])?1:0;ao[k]=(s1&&s2)?0:3-(s1+s2+s3);}
      const sh=glow?(b===B.EMBER||b===B.RUNE?1.6:Math.max(F.s,0.92)*1.15):F.s;
      for(let k=0;k<4;k++){const c=F.c[k];P_.push(wx+c[0],y+c[1],wz+c[2]);N_.push(F.d[0],F.d[1],F.d[2]);const m=sh*(glow?1:AOF[ao[k]])*jit;
        const vc=wrap?(c[1]?def.top||col:def.side):(def.side&&f!==2&&f!==3?def.side:col);
        C_.push(vc.r*m*(1-(tint-1)*0.8),vc.g*m*tint,vc.b*m);}
      if(ao[0]+ao[2]<ao[1]+ao[3])I_.push(vc+1,vc+2,vc+3,vc+1,vc+3,vc);else I_.push(vc,vc+1,vc+2,vc,vc+2,vc+3);vc+=4;}
  }
  const g=new THREE.BufferGeometry();g.setAttribute('position',new THREE.Float32BufferAttribute(P_,3));g.setAttribute('normal',new THREE.Float32BufferAttribute(N_,3));
  g.setAttribute('color',new THREE.Float32BufferAttribute(C_,3));g.setIndex(vc>65535?new THREE.Uint32BufferAttribute(I_,1):new THREE.Uint16BufferAttribute(I_,1));g.computeBoundingSphere();
  const mesh=new THREE.Mesh(g,chunkMat);mesh.castShadow=true;mesh.receiveShadow=true;mesh.matrixAutoUpdate=false;
  // foliage: grass tufts and flowers
  const fp=[],fc=[],fs=[],fn=[];const s=world.seed;
  const vtx=(x,y,z,c,sw)=>{fp.push(x,y,z);fc.push(c.r,c.g,c.b);fs.push(sw);fn.push(0,1,0);};
  const quad=(x,y,z,ax,az,h,c0,c1)=>{const a=[x-ax,y,z-az,c0,0],b=[x+ax,y,z+az,c0,0],c=[x+ax,y+h,z+az,c1,1],d=[x-ax,y+h,z-az,c1,1];
    for(const v of[a,b,c,a,c,d,a,c,b,a,d,c])vtx(v[0],v[1],v[2],v[3],v[4]);};
  const flat=(x,y,z,r,c)=>{const a=[x-r,y,z-r],b=[x+r,y,z-r],cc=[x+r,y,z+r],d=[x-r,y,z+r];for(const v of[a,cc,b,a,d,cc])vtx(v[0],v[1],v[2],c,0);};
  const tc0=new THREE.Color(),tc1=new THREE.Color();
  for(let lz=0;lz<CS;lz++)for(let lx=0;lx<CS;lx++){const i=lx+lz*CS,tb=ch.topB[i],ty=ch.top[i];
    const wx=x0+lx,wz=z0+lz,r=hash2(wx,wz,s+700);
    if(ty<SEA&&ty>=SEA-2&&r<0.09&&world.nV(wx*0.04,wz*0.04)>-0.1){ // lily pads on shallow water, some in bloom
      const x=wx+0.5,z=wz+0.5,y=SEA+0.9;tc0.setHex(0x3f8a34);flat(x,y,z,0.34,tc0);if(hash2(wx,wz,s+708)<0.3){tc0.setHex(0xf6c6dc);flat(x+0.05,y+0.06,z,0.13,tc0);}continue;}
    if(ty<=SEA||ty>=CH-2)continue;
    const isG=tb===B.GRASS||tb===B.FGRASS;const shore=ty<=SEA+1&&(tb===B.SAND||isG);
    const meadow=tb===B.GRASS&&world.nV(wx*0.03+5,wz*0.03-9)>0.22;
    const dens=shore?0.4:tb===B.GRASS?(meadow?0.5:0.3):tb===B.FGRASS?0.3:tb===B.SAND&&ch.bio[i]===BIO.DESERT?0.03:tb===B.SNOW?0.04:0;
    if(r>dens)continue;const jx=hash2(wx,wz,s+701)*0.5+0.25,jz=hash2(wx,wz,s+702)*0.5+0.25;const x=wx+jx,z=wz+jz,y=ty+1;
    const q=hash2(wx,wz,s+703);
    if(shore){const hh=0.9+hash2(wx,wz,s+709)*0.7;tc0.setHex(0x5a6a30);tc1.setHex(0x9ab05a);quad(x,y,z,0.1,0.22,hh,tc0,tc1);quad(x,y,z,0.22,-0.1,hh,tc0,tc1);
      if(q<0.4){tc0.setHex(0x6a4428);tc1.setHex(0x5a3820);quad(x+0.05,y+hh-0.25,z,0.04,0.04,0.28,tc0,tc1);}continue;}
    if(isG&&q<(meadow?0.34:0.12)){const pc=[0xe64545,0xf2d14a,0xf4f1ea,0x6a8cff,0xc070f0,0xff9a3d][Math.floor(hash2(wx,wz,s+704)*6)];
      tc0.setHex(0x3c7a2a);tc1.setHex(0x4f9a35);quad(x,y,z,0.18,0.18,0.42,tc0,tc1);quad(x,y,z,0.18,-0.18,0.42,tc0,tc1);
      tc0.setHex(pc);tc1.setHex(pc).multiplyScalar(1.15);quad(x,y+0.36,z,0.16,0.16,0.2,tc0,tc1);quad(x,y+0.36,z,0.16,-0.16,0.2,tc0,tc1);}
    else if(tb===B.FGRASS&&q<0.2){ // forest floor: mushrooms and ferns
      if(q<0.06){tc0.setHex(0xe8e0d0);tc1.setHex(0xf4ece0);quad(x,y,z,0.05,0.05,0.22,tc0,tc1);quad(x,y,z,0.05,-0.05,0.22,tc0,tc1);
        const cap=hash2(wx,wz,s+710)<0.6?0xd8343a:0xc89a5a;tc0.setHex(cap).multiplyScalar(0.8);tc1.setHex(cap);quad(x,y+0.18,z,0.17,0.17,0.12,tc0,tc1);quad(x,y+0.18,z,0.17,-0.17,0.12,tc0,tc1);}
      else{tc0.setHex(0x2a5a22);tc1.setHex(0x5a9a3a);for(let k=0;k<3;k++){const an=k*1.05+q*9;quad(x,y,z,Math.cos(an)*0.5,Math.sin(an)*0.5,0.55,tc0,tc1);}}}
    else{const hh=(0.28+hash2(wx,wz,s+705)*0.4)*(meadow&&q>0.85?1.6:1);
      if(isG){tc0.setHex(tb===B.GRASS?0x3f7a28:0x346622);tc1.setHex(tb===B.GRASS?0x7cbc4a:0x66a63c);}else if(tb===B.SAND){tc0.setHex(0x7a5a33);tc1.setHex(0xa98a55);}else{tc0.setHex(0x8fa1a8);tc1.setHex(0xd8e2e6);}
      quad(x,y,z,0.28,0.28,hh,tc0,tc1);quad(x,y,z,0.28,-0.28,hh,tc0,tc1);}}
  let fol=null;
  if(fp.length){const fg=new THREE.BufferGeometry();fg.setAttribute('position',new THREE.Float32BufferAttribute(fp,3));fg.setAttribute('color',new THREE.Float32BufferAttribute(fc,3));
    fg.setAttribute('sway',new THREE.Float32BufferAttribute(fs,1));fg.setAttribute('normal',new THREE.Float32BufferAttribute(fn,3));fg.computeBoundingSphere();
    fol=new THREE.Mesh(fg,folMat);fol.receiveShadow=true;fol.matrixAutoUpdate=false;}
  return{mesh,fol};
}
const meshed=new Map();
function disposeChunk(ch){if(ch.mesh){scene.remove(ch.mesh);ch.mesh.geometry.dispose();ch.mesh=null;}if(ch.fol){scene.remove(ch.fol);ch.fol.geometry.dispose();ch.fol=null;}}
function updateChunks(px,pz,budget){
  const pcx=Math.floor(px/CS),pcz=Math.floor(pz/CS);const need=[];
  const V=Q.view;for(let dz=-V;dz<=V;dz++)for(let dx=-V;dx<=V;dx++){const dd=dx*dx+dz*dz;if(dd>V*V+1)continue;const k=world.key(pcx+dx,pcz+dz);const ch=meshed.get(k);
    if(!ch||ch.dirty)need.push([dd,pcx+dx,pcz+dz]);}
  need.sort((a,b)=>a[0]-b[0]);
  for(let i=0;i<need.length&&i<budget;i++){const[,cx,cz]=need[i];const ch=world.getChunk(cx,cz);disposeChunk(ch);const r=meshChunk(ch);ch.mesh=r.mesh;ch.fol=r.fol;ch.dirty=false;
    scene.add(ch.mesh);if(ch.fol)scene.add(ch.fol);meshed.set(world.key(cx,cz),ch);}
  for(const[k,ch]of meshed){const dx=ch.cx-pcx,dz=ch.cz-pcz;if(dx*dx+dz*dz>(V+2)*(V+2)){disposeChunk(ch);meshed.delete(k);}}
  return need.length;
}
function voxelRay(o,dir,maxD){
  let x=Math.floor(o.x),y=Math.floor(o.y),z=Math.floor(o.z);const sx=Math.sign(dir.x),sy=Math.sign(dir.y),sz=Math.sign(dir.z);
  const tdx=sx?Math.abs(1/dir.x):Infinity,tdy=sy?Math.abs(1/dir.y):Infinity,tdz=sz?Math.abs(1/dir.z):Infinity;
  let tx=sx>0?(x+1-o.x)*tdx:sx<0?(o.x-x)*tdx:Infinity,ty=sy>0?(y+1-o.y)*tdy:sy<0?(o.y-y)*tdy:Infinity,tz=sz>0?(z+1-o.z)*tdz:sz<0?(o.z-z)*tdz:Infinity;
  let t=0;for(let i=0;i<400;i++){if(y>=0&&y<CH){const b=world.get(x,y,z);if(b)return{x,y,z,t,b};}
    if(tx<ty&&tx<tz){x+=sx;t=tx;tx+=tdx;}else if(ty<tz){y+=sy;t=ty;ty+=tdy;}else{z+=sz;t=tz;tz+=tdz;}if(t>maxD)break;}
  return null;
}

/* ---------------- audio ---------------- */
const AU={ctx:null,muted:false,master:null,
  init(){if(this.ctx)return;try{this.ctx=new(window.AudioContext||window.webkitAudioContext)();this.master=this.ctx.createGain();this.master.gain.value=0.32;this.master.connect(this.ctx.destination);
    const n=this.ctx.sampleRate;this.noise=this.ctx.createBuffer(1,n,n);const d=this.noise.getChannelData(0);for(let i=0;i<n;i++)d[i]=Math.random()*2-1;}catch(e){this.ctx=null;}},
  tone(f,dur,type='sine',vol=0.3,f2=null,delay=0){const c=this.ctx;if(!c||this.muted)return;const t=c.currentTime+delay;const o=c.createOscillator(),g=c.createGain();o.type=type;o.frequency.setValueAtTime(f,t);
    if(f2)o.frequency.exponentialRampToValueAtTime(f2,t+dur);g.gain.setValueAtTime(0.0001,t);g.gain.exponentialRampToValueAtTime(vol,t+0.01);g.gain.exponentialRampToValueAtTime(0.0001,t+dur);o.connect(g);g.connect(this.master);o.start(t);o.stop(t+dur+0.02);},
  hiss(dur,freq,vol=0.3,q=1,type='bandpass',f2=null){const c=this.ctx;if(!c||this.muted)return;const t=c.currentTime;const s=c.createBufferSource();s.buffer=this.noise;const fl=c.createBiquadFilter();fl.type=type;fl.frequency.setValueAtTime(freq,t);
    if(f2)fl.frequency.exponentialRampToValueAtTime(f2,t+dur);fl.Q.value=q;const g=c.createGain();g.gain.setValueAtTime(vol,t);g.gain.exponentialRampToValueAtTime(0.0001,t+dur);s.connect(fl);fl.connect(g);g.connect(this.master);s.start(t,Math.random()*0.5);s.stop(t+dur+0.02);}};
let sfxGate={};
function sfx(n){if(!AU.ctx||AU.muted)return;const now=performance.now();if(sfxGate[n]&&now-sfxGate[n]<45)return;sfxGate[n]=now;
  switch(n){
    case'swing':AU.hiss(0.18,1800,0.25,1.2,'bandpass',600);break;
    case'heavy':AU.hiss(0.32,900,0.35,1,'bandpass',250);break;
    case'hit':AU.tone(180,0.12,'square',0.12,70);AU.hiss(0.08,2500,0.2,0.8);break;
    case'crit':AU.tone(520,0.12,'square',0.1,900);AU.hiss(0.1,3000,0.25,0.8);break;
    case'bow':AU.tone(330,0.15,'triangle',0.2,140);AU.hiss(0.12,3500,0.12,2);break;
    case'fire':AU.hiss(0.35,700,0.3,0.7,'lowpass',2400);break;
    case'boom':AU.hiss(0.9,1200,0.6,0.6,'lowpass',80);AU.tone(90,0.6,'sine',0.4,35);break;
    case'ice':AU.tone(1400,0.25,'sine',0.12,2400);AU.tone(2100,0.2,'triangle',0.06,1300,0.03);break;
    case'arcane':AU.tone(440,0.4,'sawtooth',0.06,880);AU.tone(660,0.4,'sine',0.1,330);break;
    case'holy':AU.tone(784,0.6,'sine',0.12);AU.tone(988,0.6,'sine',0.1,null,0.06);AU.tone(1175,0.7,'sine',0.08,null,0.12);break;
    case'heal':[523,659,784,1046].forEach((f,i)=>AU.tone(f,0.35,'sine',0.1,null,i*0.06));break;
    case'level':[392,523,659,784,1046].forEach((f,i)=>AU.tone(f,0.45,'triangle',0.13,null,i*0.08));break;
    case'hurt':AU.tone(140,0.22,'square',0.16,60);break;
    case'pick':AU.tone(880,0.08,'sine',0.08,1320);break;
    case'dash':AU.hiss(0.25,1200,0.3,0.8,'bandpass',3000);break;
    case'die':AU.tone(220,0.5,'sawtooth',0.12,55);break;
    case'chest':[330,440,554,660,880].forEach((f,i)=>AU.tone(f,0.3,'triangle',0.1,null,i*0.05));break;
    case'shout':AU.tone(110,0.6,'sawtooth',0.18,90);AU.tone(165,0.6,'sawtooth',0.1,140);break;
    case'eat':AU.hiss(0.1,900,0.2,2);AU.hiss(0.1,700,0.2,2);break;
    case'spawn':AU.tone(70,0.6,'sine',0.15,40);break;
    case'zap':AU.hiss(0.3,5200,0.32,0.7,'highpass',1800);AU.tone(1800,0.14,'square',0.05,180);AU.tone(70,0.35,'sine',0.25,40,0.02);break;
    case'gem':AU.tone(1100+Math.random()*500,0.07,'sine',0.045,1900);break;
    case'horn':AU.tone(98,1.4,'sawtooth',0.12,94);AU.tone(147,1.3,'sawtooth',0.07,141,0.12);AU.hiss(1.2,300,0.08,1,'lowpass');break;
    case'glass':AU.tone(2400,0.09,'triangle',0.07,1300);AU.hiss(0.18,6000,0.18,1.5,'highpass');break;
    case'swell':AU.tone(110,0.7,'sawtooth',0.07,330);break;
    case'squelch':AU.hiss(0.4,500,0.4,0.8,'lowpass',90);AU.tone(160,0.3,'sine',0.25,50);break;
    case'whoosh':AU.hiss(0.3,600,0.22,0.9,'bandpass',2400);break;
    case'rise':AU.hiss(0.35,260,0.16,1,'lowpass',120);break;
    case'relic':[523,784,1046,1568].forEach((f,i)=>AU.tone(f,0.5,'triangle',0.1,null,i*0.07));AU.tone(262,0.8,'sine',0.12);break;
    case'bomb':AU.hiss(1.2,2400,0.6,0.5,'lowpass',60);AU.tone(60,0.9,'sine',0.5,28);AU.tone(880,0.5,'triangle',0.1,220);break;
  }}

/* ---------------- particles & fx ---------------- */
class Particles{
  constructor(cap,additive){
    this.cap=cap;this.i=0;this.q=1;const g=this.geo=new THREE.BufferGeometry();
    this.p=new Float32Array(cap*3);this.c=new Float32Array(cap*3);this.s=new Float32Array(cap);this.a=new Float32Array(cap);
    this.v=new Float32Array(cap*3);this.life=new Float32Array(cap);this.max=new Float32Array(cap);this.s0=new Float32Array(cap);this.s1=new Float32Array(cap);
    this.g=new Float32Array(cap);this.dr=new Float32Array(cap);this.a0=new Float32Array(cap);
    const at=(n,arr,k)=>{const b=new THREE.BufferAttribute(arr,k);b.setUsage(THREE.DynamicDrawUsage);g.setAttribute(n,b);return b;};
    this.ap=at('position',this.p,3);this.ac=at('pcol',this.c,3);this.as=at('psize',this.s,1);this.aa=at('palpha',this.a,1);
    const m=new THREE.ShaderMaterial({transparent:true,depthWrite:false,blending:additive?THREE.AdditiveBlending:THREE.NormalBlending,uniforms:{pr:{value:renderer.getPixelRatio()}},
      vertexShader:`attribute vec3 pcol;attribute float psize,palpha;uniform float pr;varying vec3 vC;varying float vA;
        void main(){vC=pcol;vA=palpha;vec4 mv=modelViewMatrix*vec4(position,1.0);gl_PointSize=psize*pr*(420.0/max(0.1,-mv.z));gl_Position=projectionMatrix*mv;}`,
      fragmentShader:additive?`varying vec3 vC;varying float vA;void main(){vec2 c=gl_PointCoord-0.5;float d=length(c);float a=smoothstep(0.5,0.0,d);a*=a;gl_FragColor=vec4(vC*a*vA*1.6,1.0);}`
        :`varying vec3 vC;varying float vA;void main(){vec2 c=gl_PointCoord-0.5;float d=length(c);float a=smoothstep(0.5,0.15,d);gl_FragColor=vec4(vC,a*vA);}`});
    if(additive)m.blending=THREE.AdditiveBlending;
    this.pts=new THREE.Points(g,m);this.pts.frustumCulled=false;this.pts.renderOrder=5;scene.add(this.pts);
  }
  emit(x,y,z,vx,vy,vz,life,s0,s1,col,grav=0,drag=0,alpha=1){if(this.q<1&&Math.random()>this.q)return;const i=this.i;this.i=(i+1)%this.cap;
    this.p[i*3]=x;this.p[i*3+1]=y;this.p[i*3+2]=z;this.v[i*3]=vx;this.v[i*3+1]=vy;this.v[i*3+2]=vz;this.life[i]=life;this.max[i]=life;this.s0[i]=s0;this.s1[i]=s1;
    this.c[i*3]=col.r;this.c[i*3+1]=col.g;this.c[i*3+2]=col.b;this.g[i]=grav;this.dr[i]=drag;this.a0[i]=alpha;this.s[i]=s0;this.a[i]=alpha;}
  update(dt){const{p,v,life,max,s,a}=this;for(let i=0;i<this.cap;i++){if(life[i]<=0)continue;life[i]-=dt;if(life[i]<=0){s[i]=0;a[i]=0;continue;}
    const k=Math.max(0,1-this.dr[i]*dt);v[i*3]*=k;v[i*3+1]=v[i*3+1]*k-this.g[i]*dt;v[i*3+2]*=k;p[i*3]+=v[i*3]*dt;p[i*3+1]+=v[i*3+1]*dt;p[i*3+2]+=v[i*3+2]*dt;
    const t=1-life[i]/max[i];s[i]=this.s0[i]+(this.s1[i]-this.s0[i])*t;a[i]=this.a0[i]*Math.min(1,t*12)*Math.min(1,(1-t)*2.2);}
    this.ap.needsUpdate=this.ac.needsUpdate=this.as.needsUpdate=this.aa.needsUpdate=true;}
  clear(){this.life.fill(0);this.s.fill(0);this.a.fill(0);}
}
const PA=new Particles(9000,true),PN=new Particles(3000,false);
const _c=new THREE.Color();
const col=h=>_c.set(h);
class Debris{
  constructor(cap){this.cap=cap;this.i=0;this.mesh=new THREE.InstancedMesh(new THREE.BoxGeometry(1,1,1),new THREE.MeshLambertMaterial(),cap);this.mesh.frustumCulled=false;
    this.mesh.castShadow=true;this.o=new THREE.Object3D();this.d=[];const c=new THREE.Color(1,1,1);
    for(let i=0;i<cap;i++){this.d.push({life:0,max:1,p:new V3(),v:new V3(),r:new V3(),rv:new V3(),s:0.2});this.o.scale.set(0,0,0);this.o.updateMatrix();this.mesh.setMatrixAt(i,this.o.matrix);this.mesh.setColorAt(i,c);}
    scene.add(this.mesh);}
  emit(x,y,z,color,size,vx,vy,vz,life){const i=this.i;this.i=(i+1)%this.cap;const q=this.d[i];q.life=q.max=life;q.p.set(x,y,z);q.v.set(vx,vy,vz);q.r.set(rand(0,6),rand(0,6),rand(0,6));
    q.rv.set(rand(-9,9),rand(-9,9),rand(-9,9));q.s=size;this.mesh.setColorAt(i,_c.set(color));this.mesh.instanceColor.needsUpdate=true;}
  burst(pos,colors,n,speed,size=0.18,up=5){for(let k=0;k<n;k++){const a=rand(0,TAU),sp=rand(0.3,1)*speed;this.emit(pos.x+rand(-0.3,0.3),pos.y+rand(0,0.6),pos.z+rand(-0.3,0.3),Array.isArray(colors)?pick(colors):colors,size*rand(0.6,1.3),Math.cos(a)*sp,rand(0.4,1)*up,Math.sin(a)*sp,rand(0.8,1.6));}}
  update(dt){const o=this.o;for(let i=0;i<this.cap;i++){const q=this.d[i];if(q.life<=0)continue;q.life-=dt;
    if(q.life<=0){o.scale.set(0,0,0);o.updateMatrix();this.mesh.setMatrixAt(i,o.matrix);continue;}
    q.v.y-=24*dt;q.p.addScaledVector(q.v,dt);if(world.solid(q.p.x,q.p.y-q.s*0.5,q.p.z)&&q.v.y<0){q.p.y=Math.floor(q.p.y-q.s*0.5)+1+q.s*0.5;q.v.y*=-0.32;q.v.x*=0.6;q.v.z*=0.6;q.rv.multiplyScalar(0.6);}
    q.r.addScaledVector(q.rv,dt);const t=q.life/q.max;const sc=q.s*Math.min(1,t*3);o.position.copy(q.p);o.rotation.set(q.r.x,q.r.y,q.r.z);o.scale.set(sc,sc,sc);o.updateMatrix();this.mesh.setMatrixAt(i,o.matrix);}
    this.mesh.instanceMatrix.needsUpdate=true;}
  clear(){for(const q of this.d)q.life=0.0001;}
}
const DB=new Debris(700);

const lights=[];for(let i=0;i<6;i++){const l=new THREE.PointLight(0xffffff,0,12,2);scene.add(l);lights.push({l,t:0,max:1,i0:0});}
let lightIdx=0;
function flashLight(pos,color,intensity,dist,life){const L=lights[lightIdx];lightIdx=(lightIdx+1)%lights.length;L.l.color.set(color);L.l.position.copy(pos);L.l.distance=dist;L.i0=intensity;L.t=L.max=life;L.l.intensity=intensity;return L;}

const FXL=[];
const RING=new THREE.RingGeometry(0.9,1,56);RING.rotateX(-Math.PI/2);
const DISK=new THREE.CircleGeometry(1,48);DISK.rotateX(-Math.PI/2);
const CYL=new THREE.CylinderGeometry(1,1,1,28,1,true);CYL.translate(0,0.5,0);
const SPH=new THREE.SphereGeometry(1,24,16);const OCT=new THREE.OctahedronGeometry(0.14);const ROCK=new THREE.DodecahedronGeometry(0.7);
const arcGeos={};
function arcGeo(r,len){const k=r.toFixed(1)+'_'+len.toFixed(2);if(!arcGeos[k]){const g=new THREE.RingGeometry(r*0.3,r,24,1,-Math.PI/2-len/2,len);g.rotateX(-Math.PI/2);arcGeos[k]=g;}return arcGeos[k];}
const pillarMat=c=>new THREE.ShaderMaterial({transparent:true,depthWrite:false,blending:THREE.AdditiveBlending,side:THREE.DoubleSide,uniforms:{c:{value:new THREE.Color(c)},o:{value:1}},
  vertexShader:`varying vec2 vU;void main(){vU=uv;gl_Position=projectionMatrix*modelViewMatrix*vec4(position,1.0);}`,
  fragmentShader:`uniform vec3 c;uniform float o;varying vec2 vU;void main(){float a=(1.0-vU.y)*(0.55+0.45*sin(vU.x*62.83+vU.y*8.0));gl_FragColor=vec4(c*a*o*1.4,1.0);}`});
const bubbleMat=c=>new THREE.ShaderMaterial({transparent:true,depthWrite:false,blending:THREE.AdditiveBlending,uniforms:{c:{value:new THREE.Color(c)},o:{value:1}},
  vertexShader:`varying vec3 vN;varying vec3 vV;void main(){vec4 mv=modelViewMatrix*vec4(position,1.0);vN=normalize(normalMatrix*normal);vV=normalize(-mv.xyz);gl_Position=projectionMatrix*mv;}`,
  fragmentShader:`uniform vec3 c;uniform float o;varying vec3 vN;varying vec3 vV;void main(){float f=pow(1.0-abs(dot(vN,vV)),2.2);gl_FragColor=vec4(c*(f*1.2+0.06)*o,1.0);}`});
function addFx(mesh,life,upd){if(FXL.length>=CAP.fx){const o=FXL.shift();scene.remove(o.mesh);o.mesh.traverse(q=>{if(q.material&&!q.material.userData.shared)q.material.dispose();});}scene.add(mesh);const f={mesh,t:0,life,upd};FXL.push(f);return f;}
function basicAdd(c,op=1){return new THREE.MeshBasicMaterial({color:c,transparent:true,opacity:op,blending:THREE.AdditiveBlending,depthWrite:false,side:THREE.DoubleSide});}
const fx={
  burst(pos,c,n,speed,life=0.6,size=0.35,o={}){const cc=new THREE.Color(c);for(let i=0;i<n;i++){const u=rand(-1,1),a=rand(0,TAU),r=Math.sqrt(1-u*u),sp=speed*rand(0.3,1);
    PA.emit(pos.x,pos.y,pos.z,Math.cos(a)*r*sp,(o.up??0)+u*sp,Math.sin(a)*r*sp,life*rand(0.6,1.2),size,o.s1??size*0.2,cc,o.g??0,o.drag??2.5,o.a??1);}},
  smoke(pos,c,n,speed=1.5,life=1.2,size=0.9){const cc=new THREE.Color(c);for(let i=0;i<n;i++)PN.emit(pos.x+rand(-0.4,0.4),pos.y+rand(0,0.5),pos.z+rand(-0.4,0.4),rand(-1,1)*speed,rand(0.4,1.4)*speed,rand(-1,1)*speed,life*rand(0.7,1.2),size*0.5,size*1.6,cc,-0.6,1.5,0.55);},
  ring(pos,c,r0,r1,life,y=0.12,op=0.9){const m=new THREE.Mesh(RING,basicAdd(c,op));m.position.set(pos.x,pos.y+y,pos.z);m.scale.setScalar(r0);m.renderOrder=6;
    return addFx(m,life,(f,t)=>{const e=1-Math.pow(1-t,3);m.scale.setScalar(lerp(r0,r1,e));m.material.opacity=op*(1-t);});},
  disk(pos,c,r,life,op=0.35){const m=new THREE.Mesh(DISK,basicAdd(c,op));m.position.set(pos.x,pos.y+0.1,pos.z);m.scale.setScalar(r);m.renderOrder=6;
    return addFx(m,life,(f,t)=>{m.material.opacity=op*(1-t);});},
  telegraph(pos,c,r,life){const g=new THREE.Group();g.position.set(pos.x,pos.y+0.12,pos.z);const edge=new THREE.Mesh(RING,basicAdd(c,0.9));edge.scale.setScalar(r);
    const fill=new THREE.Mesh(DISK,basicAdd(c,0.28));g.add(edge,fill);g.renderOrder=6;
    return addFx(g,life,(f,t)=>{fill.scale.setScalar(Math.max(0.01,r*t));edge.material.opacity=0.5+0.5*Math.sin(t*30);});},
  pillar(pos,c,r,h,life){const m=new THREE.Mesh(CYL,pillarMat(c));m.position.copy(pos);m.scale.set(r,h,r);m.renderOrder=7;
    return addFx(m,life,(f,t)=>{m.material.uniforms.o.value=t<0.15?t/0.15:1-(t-0.15)/0.85;m.scale.x=m.scale.z=r*(1+t*0.3);});},
  arc(pos,yaw,r,len,c,life=0.2,tilt=0){const m=new THREE.Mesh(arcGeo(r,len),basicAdd(c,0.85));m.position.copy(pos);m.rotation.order='YXZ';m.rotation.set(tilt,yaw,0);m.renderOrder=7;
    const y0=yaw;return addFx(m,life,(f,t)=>{m.rotation.y=y0+(t-0.5)*0.5;m.material.opacity=0.85*(1-t);m.scale.setScalar(0.85+t*0.25);});},
  bubble(obj,c,r,life){const m=new THREE.Mesh(SPH,bubbleMat(c));m.scale.setScalar(r);m.renderOrder=8;
    return addFx(m,life,(f,t)=>{m.position.set(obj.pos.x,obj.pos.y+0.95,obj.pos.z);m.material.uniforms.o.value=(t<0.08?t/0.08:1)*(t>0.85?(1-t)/0.15:1)*(0.8+0.2*Math.sin(f.t*12));});},
  /* jagged bolt from a to b: a soft coloured sheath around a white-hot core that flickers out */
  lightning(a,b,c=0xa8dcff,life=0.24,w=0.26){const g=new THREE.Group();const sheath=basicAdd(c,0.9),core=basicAdd(0xffffff,1);const n=Math.max(4,Math.min(12,Math.ceil(a.distanceTo(b)/2.2)));let p=a.clone();
    for(let i=1;i<=n;i++){const t=i/n,q=new V3(lerp(a.x,b.x,t),lerp(a.y,b.y,t),lerp(a.z,b.z,t));if(i<n){const j=Math.min(1.1,a.distanceTo(b)*0.08);q.x+=rand(-j,j);q.y+=rand(-j,j)*0.5;q.z+=rand(-j,j);}
      const len=p.distanceTo(q)+0.05;for(const[m,s]of[[sheath,w],[core,w*0.35]]){const sg=new THREE.Mesh(BOXG,m);sg.scale.set(s,s,len);sg.position.copy(p).lerp(q,0.5);sg.lookAt(q);g.add(sg);}
      if(i<n&&Math.random()<0.35){const br=q.clone().add(new V3(rand(-2,2),rand(-1.5,0.5),rand(-2,2)));const sg=new THREE.Mesh(BOXG,sheath);sg.scale.set(w*0.5,w*0.5,q.distanceTo(br));sg.position.copy(q).lerp(br,0.5);sg.lookAt(br);g.add(sg);}p=q;}
    g.renderOrder=7;return addFx(g,life,(f,t)=>{const k=(1-t)*(Math.random()<0.25?0.35:1);sheath.opacity=0.9*k;core.opacity=k;});},
  /* a rising translucent wall: the visible "shove" of a shockwave */
  shock(pos,c,r0,r1,life=0.4,h=1.6){const m=new THREE.Mesh(CYL,pillarMat(c));m.position.copy(pos);m.scale.set(r0,h,r0);m.renderOrder=7;
    return addFx(m,life,(f,t)=>{const e=1-Math.pow(1-t,2.5);const r=lerp(r0,r1,e);m.scale.set(r,h*(1-t*0.6),r);m.material.uniforms.o.value=(1-t)*0.8;});},
  /* fast bright streaks flying outward: sparks off a hit */
  sparks(pos,c,n,speed=9,life=0.3){const cc=new THREE.Color(c);for(let i=0;i<n;i++){const u=rand(-0.3,1),a=rand(0,TAU),r=Math.sqrt(Math.max(0,1-u*u)),sp=speed*rand(0.5,1);
    PA.emit(pos.x,pos.y,pos.z,Math.cos(a)*r*sp,u*sp,Math.sin(a)*r*sp,life*rand(0.6,1),0.22,0.02,cc,14,3,1);}},
};
function updateFx(dt){for(let i=FXL.length-1;i>=0;i--){const f=FXL[i];f.t+=dt;const t=Math.min(1,f.t/f.life);f.upd(f,t);if(f.t>=f.life){scene.remove(f.mesh);f.mesh.traverse(o=>{if(o.material&&!o.material.userData.shared)o.material.dispose();});FXL.splice(i,1);}}
  for(const L of lights){if(L.t>0){L.t-=dt;L.l.intensity=Math.max(0,L.i0*(L.t/L.max));}else L.l.intensity=0;}}

/* floating combat text */
const fxLayer=$('#fx');const TXT=[];for(let i=0;i<48;i++){const e=document.createElement('div');e.className='dmg';e.style.display='none';fxLayer.appendChild(e);TXT.push({e,t:0,p:new V3(),vy:0,on:false});}
let txtI=0;
function popText(pos,text,cls=''){const T=TXT[txtI];txtI=(txtI+1)%TXT.length;T.e.textContent=text;T.e.className='dmg '+cls;T.p.set(pos.x+rand(-0.3,0.3),pos.y,pos.z+rand(-0.3,0.3));T.t=0;T.on=true;T.e.style.display='block';T.vy=cls==='crit'?2.6:2;}
const _pv=new V3();
function updateText(dt){const W=innerWidth,H=innerHeight;for(const T of TXT){if(!T.on)continue;T.t+=dt;T.p.y+=T.vy*dt;T.vy*=0.95;if(T.t>0.9){T.on=false;T.e.style.display='none';continue;}
  _pv.copy(T.p).project(camera);if(_pv.z>1){T.e.style.display='none';continue;}T.e.style.display='block';
  const sc=T.t<0.12?1+(0.12-T.t)*5:1;T.e.style.transform=`translate(${(_pv.x+1)/2*W}px,${(1-_pv.y)/2*H}px) translate(-50%,-50%) scale(${sc})`;T.e.style.opacity=T.t>0.6?(0.9-T.t)/0.3:1;}}

/* ---------------- models ---------------- */
const BOXG=new THREE.BoxGeometry(1,1,1);
function mk(parent,w,h,d,mat,x=0,y=0,z=0){const m=new THREE.Mesh(BOXG,mat);m.scale.set(w,h,d);m.position.set(x,y,z);m.castShadow=true;parent.add(m);return m;}
function pivot(parent,x=0,y=0,z=0){const g=new THREE.Group();g.position.set(x,y,z);parent.add(g);return g;}
function matL(mats,c,o){const m=new THREE.MeshLambertMaterial(Object.assign({color:c},o||{}));mats.push(m);return m;}
function matB(mats,c){const m=new THREE.MeshBasicMaterial({color:c});mats.push(m);return m;}
function humanoid(o){
  const mats=[];const root=new THREE.Group();const body=pivot(root);
  const skin=matL(mats,o.skin),shirt=matL(mats,o.shirt),pants=matL(mats,o.pants);const th=o.thin?0.6:1;
  const legL=pivot(body,-0.13,0.78,0),legR=pivot(body,0.13,0.78,0);
  mk(legL,0.22*th,0.78,0.24*th,pants,0,-0.39,0);mk(legR,0.22*th,0.78,0.24*th,pants,0,-0.39,0);
  if(o.boots){const bm=matL(mats,o.boots);mk(legL,0.25,0.2,0.29,bm,0,-0.69,0.02);mk(legR,0.25,0.2,0.29,bm,0,-0.69,0.02);}
  const torso=pivot(body,0,0.78,0);mk(torso,0.52*(o.thin?0.75:1),0.7,0.3*(o.thin?0.7:1),shirt,0,0.35,0);
  if(o.robe)mk(torso,0.56,0.6,0.34,matL(mats,o.robe),0,-0.15,0);
  if(o.belt)mk(torso,0.55,0.08,0.32,matL(mats,o.belt),0,0.06,0);
  const head=pivot(torso,0,0.7,0);mk(head,0.44,0.44,0.44,skin,0,0.22,0);
  const em=o.eyeGlow?matB(mats,o.eyeGlow):matL(mats,o.eye||0x1a1a22);mk(head,0.08,0.06,0.02,em,-0.1,0.24,0.225);mk(head,0.08,0.06,0.02,em,0.1,0.24,0.225);
  const sl=o.sleeve?matL(mats,o.sleeve):shirt;
  const armL=pivot(torso,-0.35,0.64,0),armR=pivot(torso,0.35,0.64,0);
  mk(armL,0.18*th,0.6,0.2*th,sl,0,-0.28,0);mk(armR,0.18*th,0.6,0.2*th,sl,0,-0.28,0);mk(armL,0.16*th,0.12,0.18*th,skin,0,-0.62,0);mk(armR,0.16*th,0.12,0.18*th,skin,0,-0.62,0);
  const handL=pivot(armL,0,-0.62,0),handR=pivot(armR,0,-0.62,0);
  return{root,body,torso,legL,legR,armL,armR,head,handL,handR,mats};
}
function tipOf(g,z,x=0,y=0){const t=new THREE.Object3D();t.position.set(x,y,z);g.add(t);g.userData.tip=t;}
const WMODEL={
  /* held in the left hand with the arm pointing at the target: local -y is forward, +y is back towards the archer.
     Limbs run along z (vertical in hand) and sweep back towards the archer to recurved tips; draw(k) pulls the string. */
  longbow(mats){const g=new THREE.Group();const w=matL(mats,0x6a4224),wl=matL(mats,0x8a5a32),s=matL(mats,0xeee6cc),gr=matL(mats,0x3a2a1a),gd=matL(mats,0xc9a24a);
    const P=t=>[0.26*t*t-(t>0.86?(t-0.86)*0.9:0),0.04+0.68*t];const N=7;let tip=null;
    for(const sg of[1,-1]){for(let i=0;i<N;i++){const[y0,z0]=P(i/N),[y1,z1]=P((i+1)/N);const len=Math.hypot(y1-y0,z1-z0);const th=0.075-i*0.006;
      const m=mk(g,0.06+(N-i)*0.004,th,len+0.02,i%2?w:wl,0,(y0+y1)/2,sg*(z0+z1)/2);m.rotation.x=Math.atan2(-(y1-y0),(z1-z0))*sg;}
      const[ty,tz]=P(1);mk(g,0.05,0.05,0.07,gd,0,ty,sg*tz);if(sg>0)tip=[ty,tz];}
    mk(g,0.09,0.11,0.2,gr,0,0.01,0);mk(g,0.1,0.04,0.04,gd,0,0.01,0.11);mk(g,0.1,0.04,0.04,gd,0,0.01,-0.11);
    const st1=mk(g,0.016,0.016,1,s),st2=mk(g,0.016,0.016,1,s);const arrow=new THREE.Group();g.add(arrow);
    mk(arrow,0.03,0.7,0.03,matL(mats,0x7a5a3a),0,-0.35,0);mk(arrow,0.06,0.1,0.06,matL(mats,0xd8dde4),0,-0.72,0);mk(arrow,0.1,0.12,0.012,matL(mats,0xf0f0f0),0,-0.04,0);
    const draw=k=>{const ny=tip[0]+0.03+k*0.42;for(const[m,sg]of[[st1,1],[st2,-1]]){const dy=tip[0]-ny,dz=sg*tip[1];const len=Math.hypot(dy,dz);m.scale.z=len;m.position.set(0,(tip[0]+ny)/2,dz/2);m.rotation.x=Math.atan2(-dy,dz);}
      arrow.position.y=ny;arrow.visible=k>0.02;};draw(0);g.userData.draw=draw;tipOf(g,0,0,-0.15);return g;},
  daggers(mats){const g=new THREE.Group();const b=matL(mats,0xd8dde4),h=matL(mats,0x3a2a1c);mk(g,0.05,0.03,0.42,b,0,0,0.28);mk(g,0.16,0.04,0.05,h,0,0,0.06);mk(g,0.05,0.05,0.14,h,0,0,-0.03);tipOf(g,0.5);return g;},
  firestaff(mats){const g=new THREE.Group();const w=matL(mats,0x4a2f1c),gd=matL(mats,0xd9a640),o=matB(mats,0xff7a22);mk(g,0.07,0.07,1.5,w,0,0,0.35);mk(g,0.14,0.14,0.08,gd,0,0,1.06);
    mk(g,0.05,0.18,0.05,gd,0.08,0,1.15);mk(g,0.05,0.18,0.05,gd,-0.08,0,1.15);const orb=mk(g,0.17,0.17,0.17,o,0,0,1.2);orb.rotation.set(0.6,0.6,0);tipOf(g,1.22);return g;},
  frostwand(mats){const g=new THREE.Group();const w=matL(mats,0xcfd9e8),c=matB(mats,0x8fe6ff);mk(g,0.05,0.05,0.62,w,0,0,0.2);const cr=mk(g,0.11,0.11,0.24,c,0,0,0.58);cr.rotation.z=Math.PI/4;tipOf(g,0.72);return g;},
  greatsword(mats){const g=new THREE.Group();const b=matL(mats,0xc9d0d8),gd=matL(mats,0x8a6a3a),h=matL(mats,0x2e1f16);
    mk(g,0.14,0.04,1.35,b,0,0,0.88);mk(g,0.06,0.05,0.2,b,0,0,1.62);mk(g,0.5,0.08,0.09,gd,0,0,0.18);mk(g,0.06,0.06,0.34,h,0,0,0);mk(g,0.1,0.1,0.1,gd,0,0,-0.18);tipOf(g,1.6);return g;},
  sword(mats){const g=new THREE.Group();const b=matL(mats,0xd5dbe2),gd=matL(mats,0xb08a3a),h=matL(mats,0x3a2618);
    mk(g,0.1,0.035,0.8,b,0,0,0.55);mk(g,0.36,0.07,0.07,gd,0,0,0.12);mk(g,0.05,0.05,0.22,h,0,0,0);tipOf(g,0.95);return g;},
  shield(mats){const g=new THREE.Group();const w=matL(mats,0x8a3a2a),r=matL(mats,0xb9bec5),e=matL(mats,0xe6c15a);
    mk(g,0.08,0.66,0.54,w,-0.1,0,0.05);mk(g,0.1,0.7,0.06,r,-0.1,0,0.33);mk(g,0.1,0.7,0.06,r,-0.1,0,-0.23);mk(g,0.1,0.06,0.58,r,-0.1,0.34,0.05);mk(g,0.1,0.06,0.58,r,-0.1,-0.34,0.05);mk(g,0.12,0.14,0.14,e,-0.13,0,0.05);return g;},
  mace(mats){const g=new THREE.Group();const h=matL(mats,0x5a3a22),m=matL(mats,0xd9b24a),s=matL(mats,0xeae4d0);
    mk(g,0.06,0.06,0.62,h,0,0,0.22);mk(g,0.22,0.22,0.22,m,0,0,0.6);for(const[x,y,z]of[[0.14,0,0],[-0.14,0,0],[0,0.14,0],[0,-0.14,0],[0,0,0.14]])mk(g,0.07,0.07,0.07,s,x,y,0.6+z);tipOf(g,0.7);return g;},
  scepter(mats){const g=new THREE.Group();const gd=matL(mats,0xe0b84a),gem=matB(mats,0xfff0a0);mk(g,0.05,0.05,0.85,gd,0,0,0.3);mk(g,0.2,0.04,0.04,gd,0,0,0.72);mk(g,0.04,0.2,0.04,gd,0,0,0.72);
    const c=mk(g,0.13,0.13,0.13,gem,0,0,0.82);c.rotation.set(0.7,0.7,0);tipOf(g,0.86);return g;},
};
function playerModel(cls){
  const L=CLASSES[cls].look;const h=humanoid(L);const{mats,head,torso,armL,armR}=h;
  if(cls==='ranger'){const hm=matL(mats,0x355e2c);mk(head,0.5,0.14,0.5,hm,0,0.47,-0.02);mk(head,0.5,0.46,0.1,hm,0,0.24,-0.25);mk(head,0.08,0.46,0.42,hm,-0.25,0.24,-0.02);mk(head,0.08,0.46,0.42,hm,0.25,0.24,-0.02);
    mk(torso,0.5,0.9,0.05,matL(mats,0x2c4d26),0,0.2,-0.18);mk(torso,0.12,0.5,0.12,matL(mats,0x6a4528),0.12,0.45,-0.22).rotation.z=0.5;}
  if(cls==='mage'){const hm=matL(mats,0x3a2a78);mk(head,0.66,0.06,0.66,hm,0,0.46,0);mk(head,0.42,0.2,0.42,hm,0,0.58,-0.02);mk(head,0.3,0.2,0.3,hm,0,0.76,-0.06);mk(head,0.18,0.2,0.18,hm,0,0.93,-0.12);mk(head,0.1,0.14,0.1,hm,0,1.06,-0.18);
    mk(head,0.36,0.26,0.06,matL(mats,0xe8e4dc),0,0.04,0.23);mk(head,0.44,0.05,0.45,matL(mats,0xf0c95a),0,0.5,0);}
  if(cls==='warrior'){const st=matL(mats,0xa7afb8),dk=matL(mats,0x1b1b22),rd=matL(mats,0xb3342a);mk(head,0.5,0.32,0.5,st,0,0.34,0);mk(head,0.5,0.22,0.08,st,0,0.12,-0.22);
    mk(head,0.38,0.05,0.02,dk,0,0.28,0.255);mk(head,0.06,0.18,0.3,rd,0,0.56,-0.04);mk(armL,0.28,0.16,0.3,st,0,0.02,0);mk(armR,0.28,0.16,0.3,st,0,0.02,0);mk(torso,0.3,0.52,0.02,rd,0,0.32,0.16);}
  if(cls==='priest'){const hm=matL(mats,0xf2eee4),gd=matL(mats,0xe0b84a);mk(head,0.5,0.14,0.5,hm,0,0.47,-0.02);mk(head,0.5,0.46,0.1,hm,0,0.24,-0.25);mk(torso,0.12,0.7,0.02,gd,0,0.35,0.16);mk(torso,0.4,0.12,0.02,gd,0,0.48,0.16);
    const halo=new THREE.Mesh(new THREE.TorusGeometry(0.26,0.03,6,24),new THREE.MeshBasicMaterial({color:0xffe48a}));halo.rotation.x=Math.PI/2;halo.position.set(0,0.66,-0.06);head.add(halo);}
  // capes hang from the shoulders and stream out behind when running
  const capeC={warrior:[0x9a2a22,0xd8b04a],mage:[0x2c2266,0xf0c95a],priest:[0xf2eee4,0xe0b84a]}[cls];
  if(capeC){const cape=pivot(torso,0,0.66,-0.17);const cm_=matL(mats,capeC[0]);mk(cape,0.5,0.86,0.04,cm_,0,-0.43,0);mk(cape,0.52,0.06,0.05,matL(mats,capeC[1]),0,-0.85,0);mk(cape,0.56,0.08,0.08,cm_,0,0,0.02);h.cape=cape;}
  return h;
}
function quadModel(o){const mats=[];const root=new THREE.Group();const body=pivot(root,0,o.legH,0);
  const bm=matL(mats,o.body);mk(body,o.bw,o.bh,o.bl,bm,0,o.bh/2,0);
  const head=pivot(body,0,o.bh*0.75,o.bl/2);mk(head,o.hw,o.hh,o.hl,matL(mats,o.head),0,0,o.hl/2-0.05);
  if(o.snout)mk(head,o.hw*0.5,o.hh*0.4,0.1,matL(mats,o.snout),0,-o.hh*0.15,o.hl-0.02);
  const ey=matL(mats,0x111111);mk(head,0.06,0.06,0.02,ey,-o.hw*0.3,o.hh*0.15,o.hl-0.04);mk(head,0.06,0.06,0.02,ey,o.hw*0.3,o.hh*0.15,o.hl-0.04);
  if(o.comb)mk(head,0.06,0.12,0.16,matL(mats,0xd83a3a),0,o.hh/2+0.05,o.hl*0.5);
  const lm=matL(mats,o.leg);const legs=[];for(const[x,z]of[[-1,1],[1,1],[-1,-1],[1,-1]]){if(o.two&&z<0)continue;const lp=pivot(root,x*o.bw*0.3,o.legH,z*o.bl*0.32*(o.two?0:1));mk(lp,o.lw,o.legH,o.lw,lm,0,-o.legH/2,0);legs.push(lp);}
  return{root,body,head,legs,mats};}
function slimeModel(c,sz){const mats=[];const root=new THREE.Group();const body=pivot(root);const m=matL(mats,c,{transparent:true,opacity:0.78});
  mk(body,sz,sz*0.9,sz,m,0,sz*0.45,0);mk(body,sz*0.45,sz*0.4,sz*0.45,matL(mats,new THREE.Color(c).multiplyScalar(0.6)),0,sz*0.4,0);
  const e=matL(mats,0x10161a);mk(body,0.12*sz,0.16*sz,0.02,e,-0.2*sz,sz*0.6,sz*0.5);mk(body,0.12*sz,0.16*sz,0.02,e,0.2*sz,sz*0.6,sz*0.5);mk(body,0.3*sz,0.06*sz,0.02,e,0,sz*0.35,sz*0.5);
  return{root,body,mats};}
function spiderModel(){const mats=[];const root=new THREE.Group();const body=pivot(root,0,0.45,0);const bm=matL(mats,0x2a2228),hm=matL(mats,0x3a2e33);
  mk(body,0.85,0.5,0.95,bm,0,0.1,-0.35);const head=pivot(body,0,0.05,0.3);mk(head,0.55,0.4,0.45,hm,0,0,0.1);const em=matB(mats,0xff3030);
  for(const[x,y]of[[-0.14,0.06],[0.14,0.06],[-0.07,0.12],[0.07,0.12]])mk(head,0.07,0.07,0.02,em,x,y,0.335);
  const legs=[];for(let i=0;i<8;i++){const side=i<4?-1:1,k=i%4;const lp=pivot(body,side*0.35,0,0.25-k*0.22);const l=mk(lp,0.62,0.07,0.07,hm,side*0.3,0.05,0);l.rotation.z=side*0.5;
    const l2=mk(lp,0.07,0.5,0.07,hm,side*0.6,-0.2,0);legs.push(lp);}
  return{root,body,head,legs,mats};}
function golemModel(){const mats=[];const root=new THREE.Group();const body=pivot(root);const st=matL(mats,0x7d8088),dk=matL(mats,0x5c5f66),ms=matL(mats,0x5c7a42),gl=matB(mats,0x6ff3ff);
  const legL=pivot(body,-0.45,1.3,0),legR=pivot(body,0.45,1.3,0);mk(legL,0.62,1.3,0.66,dk,0,-0.65,0);mk(legR,0.62,1.3,0.66,dk,0,-0.65,0);
  const torso=pivot(body,0,1.3,0);mk(torso,1.7,1.3,1.0,st,0,0.7,0);mk(torso,1.2,0.3,0.9,ms,0.1,1.42,-0.05);mk(torso,0.3,0.3,0.05,gl,0,0.75,0.51);
  const head=pivot(torso,0,1.35,0.25);mk(head,0.72,0.6,0.62,dk,0,0.25,0);mk(head,0.14,0.08,0.02,gl,-0.16,0.3,0.32);mk(head,0.14,0.08,0.02,gl,0.16,0.3,0.32);
  const armL=pivot(torso,-1.05,1.15,0),armR=pivot(torso,1.05,1.15,0);mk(armL,0.52,1.5,0.56,st,0,-0.75,0);mk(armR,0.52,1.5,0.56,st,0,-0.75,0);mk(armL,0.66,0.55,0.66,dk,0,-1.55,0);mk(armR,0.66,0.55,0.66,dk,0,-1.55,0);
  mk(armL,0.4,0.2,0.4,ms,0,-0.05,0);for(const[x,y]of[[-0.5,1.1],[0.55,0.4],[-0.3,0.25]])mk(torso,0.12,0.12,0.04,gl,x,y,0.51);
  mk(torso,0.5,0.35,0.45,ms,-0.6,1.3,0);mk(torso,0.3,0.25,0.3,dk,0.65,1.5,-0.1);return{root,body,torso,legL,legR,armL,armR,head,mats};}
function batModel(){const mats=[];const root=new THREE.Group();const body=pivot(root,0,0.3,0);const fur=matL(mats,0x3a2a40),wing=matL(mats,0x5c3a62),mem=matL(mats,0x7a4a6a),em=matB(mats,0xff4a5a);
  mk(body,0.34,0.3,0.42,fur,0,0,0);const head=pivot(body,0,0.1,0.24);mk(head,0.28,0.24,0.24,fur,0,0,0.04);mk(head,0.07,0.16,0.05,mem,-0.09,0.17,0);mk(head,0.07,0.16,0.05,mem,0.09,0.17,0);
  mk(head,0.06,0.05,0.02,em,-0.065,0.03,0.165);mk(head,0.06,0.05,0.02,em,0.065,0.03,0.165);mk(head,0.04,0.05,0.02,matB(mats,0xffffff),-0.03,-0.08,0.165);mk(head,0.04,0.05,0.02,matB(mats,0xffffff),0.03,-0.08,0.165);
  const wingL=pivot(body,-0.15,0.06,0),wingR=pivot(body,0.15,0.06,0);
  for(const[w,s]of[[wingL,-1],[wingR,1]]){mk(w,0.5,0.04,0.38,wing,s*0.27,0,0);mk(w,0.36,0.04,0.26,mem,s*0.66,0,-0.06);mk(w,0.06,0.07,0.5,fur,s*0.5,0.03,0.04);}
  return{root,body,head,wingL,wingR,mats};}
function bloaterModel(){const h=humanoid({skin:0x87a856,shirt:0x5d7a3a,pants:0x3a4a2a,eye:0x10200a});const{mats,torso,head}=h;
  const belly=pivot(torso,0,0.32,0.05);const sk=matL(mats,0x9ab860),gl=matB(mats,0xc8ff5a),dk=matL(mats,0x5a7a32);
  mk(belly,0.86,0.82,0.7,sk,0,0,0);mk(belly,0.5,0.46,0.06,gl,0,-0.02,0.35);for(const[x,y,z]of[[-0.34,0.24,0.3],[0.3,-0.22,0.32],[0.38,0.2,-0.1],[-0.3,-0.25,-0.2]])mk(belly,0.16,0.16,0.16,dk,x,y,z);
  mk(head,0.1,0.05,0.02,gl,0,0.06,0.225);h.belly=belly;return h;}
function bruteModel(){const h=humanoid({skin:0x9a7a62,shirt:0x6a4a34,pants:0x3a2a22,boots:0x2a1e18,belt:0x2a1e18,eyeGlow:0xffb03a});const{mats,torso,head,armL,armR,handR}=h;
  const pad=matL(mats,0x5a5f66),horn=matL(mats,0xe8dcc0),wd=matL(mats,0x5a3a22),ir=matL(mats,0x8a8f96);
  mk(torso,0.66,0.5,0.42,h.mats[1],0,0.48,0);mk(armL,0.36,0.22,0.36,pad,0,0.04,0);mk(armR,0.36,0.22,0.36,pad,0,0.04,0);mk(head,0.06,0.18,0.06,horn,-0.2,0.5,0.05).rotation.z=0.5;mk(head,0.06,0.18,0.06,horn,0.2,0.5,0.05).rotation.z=-0.5;
  mk(head,0.3,0.08,0.04,matL(mats,0x3a2a22),0,0.1,0.23);const club=new THREE.Group();mk(club,0.12,0.12,0.9,wd,0,0,0.3);mk(club,0.3,0.3,0.42,wd,0,0,0.82);
  for(const[x,y]of[[0.16,0],[-0.16,0],[0,0.16],[0,-0.16]])mk(club,0.08,0.08,0.08,ir,x,y,0.86);club.rotation.x=Math.PI/2;club.position.y=-0.05;handR.add(club);return h;}
function urnModel(){const mats=[];const root=new THREE.Group();const body=pivot(root);const cl=matL(mats,0xb0683e),dk=matL(mats,0x7a4428),gd=matL(mats,0xe0b84a),gl=matB(mats,0xffd36a);
  mk(body,0.5,0.18,0.5,dk,0,0.09,0);mk(body,0.72,0.5,0.72,cl,0,0.42,0);mk(body,0.76,0.08,0.76,gd,0,0.56,0);mk(body,0.5,0.24,0.5,cl,0,0.78,0);mk(body,0.36,0.1,0.36,dk,0,0.94,0);
  mk(body,0.14,0.14,0.14,gl,0,1.02,0).rotation.set(0.6,0.6,0);for(const s of[-1,1])mk(body,0.08,0.26,0.12,dk,s*0.4,0.62,0);return{root,body,mats};}
/* merge each pivot's plain boxes into one vertex-coloured mesh (glowing parts into a second unlit one).
   An animated enemy drops from ~15 draw calls to ~7, which is what lets a horde of 100+ stay smooth. */
function bakeModel(m,shadow=true){
  const lam=new THREE.MeshLambertMaterial({vertexColors:true}),glow=new THREE.MeshBasicMaterial({vertexColors:true});
  const bp=BOXG.attributes.position.array,bn=BOXG.attributes.normal.array,bi=BOXG.index.array,v=new V3(),n=new V3(),nm=new THREE.Matrix3();
  const walk=g=>{const sets=[{p:[],n:[],c:[],i:[]},{p:[],n:[],c:[],i:[]}];
    for(const ch of g.children.slice()){
      if(ch.isMesh&&ch.geometry===BOXG&&!ch.material.transparent){const s=ch.material.isMeshBasicMaterial?sets[1]:sets[0];ch.updateMatrix();nm.getNormalMatrix(ch.matrix);const base=s.p.length/3,cc=ch.material.color;
        for(let k=0;k<bp.length;k+=3){v.set(bp[k],bp[k+1],bp[k+2]).applyMatrix4(ch.matrix);n.set(bn[k],bn[k+1],bn[k+2]).applyMatrix3(nm).normalize();s.p.push(v.x,v.y,v.z);s.n.push(n.x,n.y,n.z);s.c.push(cc.r,cc.g,cc.b);}
        for(let k=0;k<bi.length;k++)s.i.push(base+bi[k]);g.remove(ch);}
      else if(!ch.isMesh)walk(ch);}
    sets.forEach((s,k)=>{if(!s.p.length)return;const geo=new THREE.BufferGeometry();geo.setAttribute('position',new THREE.Float32BufferAttribute(s.p,3));geo.setAttribute('normal',new THREE.Float32BufferAttribute(s.n,3));
      geo.setAttribute('color',new THREE.Float32BufferAttribute(s.c,3));geo.setIndex(s.i);geo.computeBoundingSphere();geo.userData.own=true;const mesh=new THREE.Mesh(geo,k?glow:lam);mesh.castShadow=shadow&&!k;g.add(mesh);});};
  walk(m.root);for(const mt of m.mats)mt.dispose();m.mats=[lam,glow];return m;}

/* ---------------- classes ---------------- */
const CLASSES={
  ranger:{name:'Ranger',color:'#74c454',hp:100,speed:1.08,role:'Fast skirmisher who wins from range and never stands still.',
    look:{skin:0xe0b08a,shirt:0x4b7a3c,pants:0x5a4632,boots:0x3a2a1c,belt:0x6a4528},
    weapons:['longbow','daggers'],abilities:['volley','roll','snare','storm']},
  mage:{name:'Mage',color:'#a183ff',hp:85,speed:1.0,role:'Glass cannon. Fire for damage, frost for control.',
    look:{skin:0xe8c09a,shirt:0x4b3a8f,pants:0x2d2550,robe:0x3f3180,belt:0xf0c95a,sleeve:0x4b3a8f},
    weapons:['firestaff','frostwand'],abilities:['blink','nova','orb','meteor']},
  warrior:{name:'Warrior',color:'#e25b45',hp:140,speed:0.98,role:'Armored brawler who closes distance and refuses to fall.',
    look:{skin:0xd9a47a,shirt:0x8d959e,pants:0x4a4038,boots:0x2a221c,belt:0x5a3a22,sleeve:0x6d757e},
    weapons:['greatsword','swordshield'],abilities:['charge','whirl','slam','cry']},
  priest:{name:'Priest',color:'#f2cf63',hp:110,speed:1.0,role:'Holy bruiser who heals through anything and burns the dark.',
    look:{skin:0xe6bd98,shirt:0xece6d8,pants:0xb9ae98,robe:0xe4dccb,belt:0xe0b84a,sleeve:0xece6d8},
    weapons:['mace','scepter'],abilities:['mend','smite','shield','consecrate']},
};
const WEAPONS={
  longbow:{name:'Longbow',type:'bow',rate:0.25,icon:'bow',desc:'Hold to draw, release to fire. Full draw pierces.'},
  daggers:{name:'Twin Daggers',type:'melee',rate:0.26,dmg:9,range:2.7,arc:1.9,icon:'daggers',desc:'Rapid strikes. Every third hit deals double.',color:0xd8f0ff},
  firestaff:{name:'Fire Staff',type:'cast',rate:0.62,icon:'fire',desc:'Fireballs that explode and set enemies alight.'},
  frostwand:{name:'Frost Wand',type:'cast',rate:0.17,icon:'ice',desc:'Streams of ice shards. Five hits freeze a target.'},
  greatsword:{name:'Greatsword',type:'melee',rate:0.72,dmg:27,range:3.7,arc:2.9,icon:'gsword',desc:'Huge, slow cleaves that knock enemies back.',color:0xfff1d6,heavy:true},
  swordshield:{name:'Sword & Shield',type:'melee',rate:0.38,dmg:14,range:2.9,arc:2.0,icon:'shieldi',desc:'Quick cuts. The shield blocks 30% of all damage.',color:0xe8f4ff},
  mace:{name:'Holy Mace',type:'melee',rate:0.5,dmg:16,range:3.0,arc:2.1,icon:'mace',desc:'Blessed blows that heal you a little on every hit.',color:0xffe08a},
  scepter:{name:'Sacred Scepter',type:'cast',rate:0.42,icon:'scepter',desc:'Seeking bolts of holy light.'},
};
const ABIL={
  volley:{name:'Volley',cd:5,icon:'fan',color:'#9be07a',desc:'Fan of 9 piercing arrows.'},
  roll:{name:'Evasive Roll',cd:3,icon:'dash',color:'#c9e6b0',desc:'Dash and dodge all damage briefly.'},
  snare:{name:'Snare Trap',cd:8,icon:'trap',color:'#d9b46a',desc:'Throw a trap that roots and wounds.'},
  storm:{name:'Arrow Storm',cd:13,icon:'rain',color:'#7fd46a',desc:'Rain arrows on an area for 4s.'},
  blink:{name:'Blink',cd:4.5,icon:'blink',color:'#c4a8ff',desc:'Teleport 11m where you aim.'},
  nova:{name:'Frost Nova',cd:8,icon:'nova',color:'#8fe6ff',desc:'Freeze everything around you.'},
  orb:{name:'Arcane Orb',cd:5,icon:'orb',color:'#d08aff',desc:'Slow orb that pierces and bursts.'},
  meteor:{name:'Meteor',cd:14,icon:'meteor',color:'#ff8a3d',desc:'Call down a crater-making meteor.'},
  charge:{name:'Charge',cd:5.5,icon:'charge',color:'#ff8a6a',desc:'Rush forward, stunning all you hit.'},
  whirl:{name:'Whirlwind',cd:9,icon:'spin',color:'#ffd0a0',desc:'Spin for 2.5s, cutting everything near.'},
  slam:{name:'Ground Slam',cd:8,icon:'slam',color:'#d9a46a',desc:'Leap to a spot and shatter the ground.'},
  cry:{name:'Battle Cry',cd:18,icon:'shout',color:'#ff5b4c',desc:'+40% damage and lifesteal for 8s.'},
  mend:{name:'Mend',cd:7,icon:'heal',color:'#8dff9e',desc:'Heal 35% health plus regen.'},
  smite:{name:'Smite',cd:4.5,icon:'pillar',color:'#ffe48a',desc:'Pillar of light at your aim. Stuns.'},
  shield:{name:'Divine Shield',cd:16,icon:'shieldb',color:'#fff3b8',desc:'Become invulnerable for 3s.'},
  consecrate:{name:'Consecration',cd:11,icon:'circle',color:'#ffd35a',desc:'Holy ground: burns foes, heals you.'},
};

/* icons drawn on canvas */
function drawIcon(cv,kind,color){const c=cv.getContext('2d'),S=cv.width;c.clearRect(0,0,S,S);c.save();c.scale(S/48,S/48);c.strokeStyle=color;c.fillStyle=color;c.lineWidth=3.4;c.lineCap='round';c.lineJoin='round';
  const L=(...p)=>{c.beginPath();c.moveTo(p[0],p[1]);for(let i=2;i<p.length;i+=2)c.lineTo(p[i],p[i+1]);c.stroke();};
  const C=(x,y,r,f)=>{c.beginPath();c.arc(x,y,r,0,TAU);f?c.fill():c.stroke();};
  switch(kind){
    case'bow':c.beginPath();c.arc(12,24,17,-1.15,1.15);c.stroke();c.lineWidth=1.5;L(18,8,18,40);c.lineWidth=3;L(10,24,42,24);L(36,19,42,24,36,29);break;
    case'daggers':L(10,38,30,18);L(30,18,37,11);L(16,32,20,36);L(38,38,18,18);L(18,18,11,11);L(32,32,28,36);break;
    case'fire':c.beginPath();c.moveTo(24,5);c.bezierCurveTo(38,18,38,32,24,43);c.bezierCurveTo(10,32,12,20,19,13);c.bezierCurveTo(19,22,23,25,24,5);c.fill();break;
    case'ice':for(let i=0;i<3;i++){const a=i*Math.PI/3+Math.PI/2;L(24+Math.cos(a)*18,24+Math.sin(a)*18,24-Math.cos(a)*18,24-Math.sin(a)*18);}C(24,24,4,1);break;
    case'gsword':L(10,38,38,10);c.lineWidth=5;L(38,10,24,24);c.lineWidth=3.4;L(12,26,22,36);break;
    case'shieldi':c.beginPath();c.moveTo(24,6);c.lineTo(40,12);c.lineTo(38,28);c.quadraticCurveTo(34,38,24,43);c.quadraticCurveTo(14,38,10,28);c.lineTo(8,12);c.closePath();c.stroke();L(24,14,24,34);L(17,20,31,20);break;
    case'mace':L(12,38,28,22);C(32,17,8,1);L(32,5,32,8);L(44,17,41,17);L(40,9,38,11);break;
    case'scepter':L(14,40,30,20);c.beginPath();c.moveTo(34,6);c.lineTo(41,14);c.lineTo(34,22);c.lineTo(27,14);c.closePath();c.fill();break;
    case'fan':for(let a=-3;a<=3;a++){const an=-Math.PI/2+a*0.28;L(24,42,24+Math.cos(an)*30,42+Math.sin(an)*30);}break;
    case'dash':L(6,30,26,30);L(10,22,30,22);L(6,14,22,14);C(36,24,7,1);break;
    case'trap':L(6,34,42,34);for(let i=0;i<5;i++)L(10+i*7,34,13+i*7,20);C(24,36,4,1);break;
    case'rain':for(let i=0;i<5;i++){const x=9+i*7.5,y=6+(i%2)*7;L(x,y,x-4,y+20);L(x-7,y+15,x-4,y+20,x+1,y+16);}L(4,43,44,43);break;
    case'blink':c.setLineDash([3,4]);C(13,26,7,0);c.setLineDash([]);C(34,22,8,1);L(20,25,26,23);break;
    case'nova':C(24,24,5,1);C(24,24,14,0);for(let i=0;i<8;i++){const a=i*TAU/8;L(24+Math.cos(a)*18,24+Math.sin(a)*18,24+Math.cos(a)*22,24+Math.sin(a)*22);}break;
    case'orb':C(24,24,9,1);c.lineWidth=2;C(24,24,15,0);c.beginPath();c.arc(24,24,20,0.3,1.6);c.stroke();c.beginPath();c.arc(24,24,20,3.4,4.7);c.stroke();break;
    case'meteor':C(31,31,10,1);L(6,6,22,22);L(15,4,27,17);L(4,15,17,27);break;
    case'charge':L(6,24,34,24);L(26,14,38,24,26,34);L(6,16,16,16);L(6,32,16,32);break;
    case'spin':c.beginPath();c.arc(24,24,15,0.4,TAU-0.6);c.stroke();L(36,10,39,17,32,18);C(24,24,4,1);break;
    case'slam':L(24,6,24,26);L(16,18,24,26,32,18);L(6,40,42,40);L(10,34,14,40);L(38,34,34,40);L(24,32,24,40);break;
    case'shout':c.beginPath();c.moveTo(8,18);c.lineTo(18,18);c.lineTo(28,8);c.lineTo(28,40);c.lineTo(18,30);c.lineTo(8,30);c.closePath();c.fill();c.beginPath();c.arc(30,24,8,-0.9,0.9);c.stroke();c.beginPath();c.arc(30,24,15,-0.9,0.9);c.stroke();break;
    case'heal':c.lineWidth=7;L(24,9,24,39);L(9,24,39,24);break;
    case'pillar':c.globalAlpha=0.5;c.fillRect(16,4,16,36);c.globalAlpha=1;c.fillRect(20,4,8,36);L(8,42,40,42);break;
    case'shieldb':c.beginPath();c.moveTo(24,5);c.lineTo(40,11);c.lineTo(38,28);c.quadraticCurveTo(34,38,24,43);c.quadraticCurveTo(14,38,10,28);c.lineTo(8,11);c.closePath();c.fill();break;
    case'circle':C(24,24,17,0);C(24,24,9,0);L(24,7,24,41);L(7,24,41,24);break;
    case'food':c.beginPath();c.ellipse(22,26,13,10,-0.5,0,TAU);c.fill();L(32,16,40,8);C(41,7,3,1);break;
    case'blades':C(24,24,4,1);for(let i=0;i<3;i++){const a=i*TAU/3;c.save();c.translate(24+Math.cos(a)*13,24+Math.sin(a)*13);c.rotate(a+1.9);c.beginPath();c.moveTo(-9,0);c.lineTo(0,-3);c.lineTo(9,0);c.lineTo(0,3);c.closePath();c.fill();c.restore();}
      c.lineWidth=1.6;c.setLineDash([3,4]);C(24,24,13,0);c.setLineDash([]);break;
    case'bolt':c.beginPath();c.moveTo(28,3);c.lineTo(12,26);c.lineTo(23,26);c.lineTo(18,45);c.lineTo(37,19);c.lineTo(25,19);c.lineTo(31,3);c.closePath();c.fill();break;
    case'aura':C(24,24,6,1);c.lineWidth=2.4;C(24,24,13,0);for(let i=0;i<8;i++){const a=i*TAU/8;c.beginPath();c.arc(24+Math.cos(a)*19,24+Math.sin(a)*19,2.6,0,TAU);c.fill();}break;
    case'wisp':C(26,20,9,1);c.beginPath();c.moveTo(18,24);c.quadraticCurveTo(12,36,4,40);c.quadraticCurveTo(16,38,24,29);c.fill();c.lineWidth=2;L(34,22,44,18);L(35,16,43,10);break;
    case'axe':c.save();c.translate(24,24);c.rotate(-0.6);c.fillRect(-2,-18,4,36);c.beginPath();c.moveTo(2,-16);c.quadraticCurveTo(18,-14,16,0);c.quadraticCurveTo(10,-6,2,-4);c.closePath();c.fill();c.restore();
      c.lineWidth=2;c.beginPath();c.arc(24,24,20,2.2,3.8);c.stroke();break;
    case'flask':c.beginPath();c.moveTo(20,6);c.lineTo(28,6);c.lineTo(28,17);c.lineTo(38,36);c.quadraticCurveTo(40,42,34,42);c.lineTo(14,42);c.quadraticCurveTo(8,42,10,36);c.lineTo(20,17);c.closePath();c.stroke();
      c.beginPath();c.moveTo(15,30);c.lineTo(33,30);c.lineTo(37,38);c.quadraticCurveTo(38,40,34,40);c.lineTo(14,40);c.quadraticCurveTo(10,40,11,38);c.closePath();c.fill();break;
    case'halo':for(let i=0;i<8;i++){const a=i*TAU/8;L(24+Math.cos(a)*8,24+Math.sin(a)*8,24+Math.cos(a)*20,24+Math.sin(a)*20);}C(24,24,4,1);break;
    case'chaos':for(let i=0;i<8;i++){const a=i/8*TAU;c.save();c.translate(24,24);c.rotate(a);c.beginPath();c.moveTo(0,-4);c.lineTo(3,-12);c.lineTo(0,-21);c.lineTo(-3,-12);c.closePath();c.fill();c.restore();}C(24,24,5,1);break;
    case'reroll':c.beginPath();c.arc(24,24,14,-0.3,TAU-1.2);c.stroke();L(34,6,37,14,29,15);break;
  }c.restore();}


/* ---------------- physics ---------------- */
function collides(x,y,z,w,h){const x0=Math.floor(x-w),x1=Math.floor(x+w),y0=Math.floor(y),y1=Math.floor(y+h-0.01),z0=Math.floor(z-w),z1=Math.floor(z+w);
  for(let yy=y0;yy<=y1;yy++)for(let zz=z0;zz<=z1;zz++)for(let xx=x0;xx<=x1;xx++)if(world.get(xx,yy,zz))return true;return false;}
function moveEnt(e,dt,grav=26){
  e.inWater=e.pos.y+0.5<SEA+0.88;
  if(!e.fly){e.vel.y-=grav*(e.inWater?0.35:1)*dt;if(e.inWater){e.vel.y=Math.max(e.vel.y,-3);e.vel.multiplyScalar(1-dt*1.5);}}
  e.vel.y=Math.max(e.vel.y,-40);const wasGround=e.onGround;e.onGround=false;e.hitWall=false;
  const dx=e.vel.x*dt,dy=e.vel.y*dt,dz=e.vel.z*dt;const n=Math.max(1,Math.ceil(Math.max(Math.abs(dx),Math.abs(dy),Math.abs(dz))/0.35));
  for(let s=0;s<n;s++){
    e.pos.y+=dy/n;if(collides(e.pos.x,e.pos.y,e.pos.z,e.w,e.h)){if(dy<0){e.pos.y=Math.floor(e.pos.y)+1;e.onGround=true;}else{e.pos.y=Math.floor(e.pos.y+e.h)-e.h-0.001;}e.vel.y=0;}
    e.pos.x+=dx/n;if(collides(e.pos.x,e.pos.y,e.pos.z,e.w,e.h)){if((wasGround||e.onGround)&&!collides(e.pos.x,e.pos.y+1.02,e.pos.z,e.w,e.h)){e.pos.y=Math.floor(e.pos.y)+1.0;}else{e.pos.x-=dx/n;e.vel.x=0;e.hitWall=true;}}
    e.pos.z+=dz/n;if(collides(e.pos.x,e.pos.y,e.pos.z,e.w,e.h)){if((wasGround||e.onGround)&&!collides(e.pos.x,e.pos.y+1.02,e.pos.z,e.w,e.h)){e.pos.y=Math.floor(e.pos.y)+1.0;}else{e.pos.z-=dz/n;e.vel.z=0;e.hitWall=true;}}
  }
  if(e.pos.y<-10){e.pos.y=world.topAt(Math.floor(e.pos.x),Math.floor(e.pos.z))+2;e.vel.set(0,0,0);}
}
