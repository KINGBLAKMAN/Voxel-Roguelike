'use strict';
/* Emberwild renderer: the scene is drawn into an HDR buffer, then a few cheap full-screen passes add
   bloom (a dual-filter blur chain, so fire, magic and glowing blocks bleed light), sun shafts through
   gaps in the terrain, a soft filmic tone curve, colour grading, a vignette and FXAA.
   Low graphics skips all of it and draws straight to the screen. */
const POST=(()=>{
  const ext=renderer.extensions,webgl2=renderer.capabilities.isWebGL2;
  const hdr=(webgl2&&!!ext.get('EXT_color_buffer_float'))||!!ext.get('EXT_color_buffer_half_float');
  const type=hdr?THREE.HalfFloatType:THREE.UnsignedByteType;
  const rtOpts={type,minFilter:THREE.LinearFilter,magFilter:THREE.LinearFilter,depthBuffer:false,stencilBuffer:false};
  const S={on:true,level:2,w:0,h:0,main:null,down:[],up:[],shaft:null,levels:5};
  const quadScene=new THREE.Scene(),quadCam=new THREE.OrthographicCamera(-1,1,1,-1,0,1);
  const quad=new THREE.Mesh(new THREE.PlaneGeometry(2,2),null);quad.frustumCulled=false;quadScene.add(quad);
  const V=`varying vec2 vUv;void main(){vUv=uv;gl_Position=vec4(position.xy,0.0,1.0);}`;
  const mat=(frag,u)=>new THREE.ShaderMaterial({uniforms:u,vertexShader:V,fragmentShader:frag,depthTest:false,depthWrite:false});
  // 1. bright pass with a soft knee, so only genuinely hot pixels bloom
  const bright=mat(`uniform sampler2D tIn;uniform float thr,knee;varying vec2 vUv;
    void main(){vec3 c=texture2D(tIn,vUv).rgb;float l=max(c.r,max(c.g,c.b));float s=clamp(l-thr+knee,0.0,2.0*knee);s=s*s/(4.0*knee+1e-4);
      float w=max(s,l-thr)/max(l,1e-4);gl_FragColor=vec4(c*w,1.0);}`,{tIn:{value:null},thr:{value:1},knee:{value:0.3}});
  // 2. dual-filter (Kawase) down and up sampling: a wide, smooth blur for very few texture reads
  const down=mat(`uniform sampler2D tIn;uniform vec2 px;varying vec2 vUv;
    void main(){vec3 s=texture2D(tIn,vUv).rgb*4.0;s+=texture2D(tIn,vUv-px).rgb;s+=texture2D(tIn,vUv+px).rgb;s+=texture2D(tIn,vUv+vec2(px.x,-px.y)).rgb;s+=texture2D(tIn,vUv-vec2(px.x,-px.y)).rgb;gl_FragColor=vec4(s/8.0,1.0);}`,
    {tIn:{value:null},px:{value:new THREE.Vector2()}});
  const up=mat(`uniform sampler2D tIn,tAdd;uniform vec2 px;varying vec2 vUv;
    void main(){vec3 s=texture2D(tIn,vUv+vec2(-px.x*2.0,0.0)).rgb;s+=texture2D(tIn,vUv+vec2(-px.x,px.y)).rgb*2.0;s+=texture2D(tIn,vUv+vec2(0.0,px.y*2.0)).rgb;s+=texture2D(tIn,vUv+vec2(px.x,px.y)).rgb*2.0;
      s+=texture2D(tIn,vUv+vec2(px.x*2.0,0.0)).rgb;s+=texture2D(tIn,vUv+vec2(px.x,-px.y)).rgb*2.0;s+=texture2D(tIn,vUv+vec2(0.0,-px.y*2.0)).rgb;s+=texture2D(tIn,vUv+vec2(-px.x,-px.y)).rgb*2.0;
      gl_FragColor=vec4(s/12.0+texture2D(tAdd,vUv).rgb,1.0);}`,{tIn:{value:null},tAdd:{value:null},px:{value:new THREE.Vector2()}});
  // 3. sun shafts: march from each pixel towards the sun, gathering glow wherever open sky shows through
  const shaft=mat(`uniform sampler2D tDepth;uniform vec2 sun;uniform float aspect,amt;varying vec2 vUv;
    void main(){vec2 d=(sun-vUv)/40.0;vec2 p=vUv;float acc=0.0,w=1.0;
      for(int i=0;i<40;i++){float sky=step(0.99999,texture2D(tDepth,p).r);vec2 q=(p-sun)*vec2(aspect,1.0);acc+=sky*pow(max(0.0,1.0-length(q)*1.5),2.0)*w;w*=0.965;p+=d;}
      gl_FragColor=vec4(vec3(acc/40.0*amt),1.0);}`,{tDepth:{value:null},sun:{value:new THREE.Vector2()},aspect:{value:1},amt:{value:0}});
  // 4. composite: FXAA on the scene, add bloom and shafts, filmic shoulder, grade, vignette, dither
  const comp=mat(`uniform sampler2D tScene,tBloom,tShaft;uniform vec2 texel;uniform float bloom,exposure,sat,contrast,vig,aa,time,useShaft;uniform vec3 shaftCol,lift,gain;varying vec2 vUv;
    vec3 tm(vec3 c){c*=exposure;vec3 k=vec3(0.78);vec3 o=k+(1.0-k)*(1.0-exp(-(c-k)/(1.0-k)));return mix(c,o,step(k,c));}
    float lu(vec3 c){return dot(tm(c),vec3(0.299,0.587,0.114));}
    vec3 fxaa(vec2 uv){vec3 M=texture2D(tScene,uv).rgb;if(aa<0.5)return M;
      float nw=lu(texture2D(tScene,uv+vec2(-1.0,-1.0)*texel).rgb),ne=lu(texture2D(tScene,uv+vec2(1.0,-1.0)*texel).rgb),sw=lu(texture2D(tScene,uv+vec2(-1.0,1.0)*texel).rgb),se=lu(texture2D(tScene,uv+vec2(1.0,1.0)*texel).rgb),m=lu(M);
      float mn=min(m,min(min(nw,ne),min(sw,se))),mx=max(m,max(max(nw,ne),max(sw,se)));if(mx-mn<max(0.04,mx*0.12))return M;
      vec2 dir=vec2(-((nw+ne)-(sw+se)),(nw+sw)-(ne+se));float red=max((nw+ne+sw+se)*0.03125,1.0/128.0);dir=clamp(dir/(min(abs(dir.x),abs(dir.y))+red),-8.0,8.0)*texel;
      vec3 A=0.5*(texture2D(tScene,uv-dir/6.0).rgb+texture2D(tScene,uv+dir/6.0).rgb);vec3 B=A*0.5+0.25*(texture2D(tScene,uv-dir*0.5).rgb+texture2D(tScene,uv+dir*0.5).rgb);
      float lb=lu(B);return(lb<mn||lb>mx)?A:B;}
    void main(){vec3 c=fxaa(vUv);c+=texture2D(tBloom,vUv).rgb*bloom;if(useShaft>0.5)c+=texture2D(tShaft,vUv).r*shaftCol;
      c=tm(c);float l=dot(c,vec3(0.299,0.587,0.114));c=mix(vec3(l),c,sat);c=(c-0.5)*contrast+0.5;
      c=c*gain+lift*(1.0-l);vec2 q=vUv-0.5;c*=1.0-dot(q,q)*vig;
      c+=(fract(sin(dot(vUv*vec2(12.9898,78.233)+time,vec2(1.0)))*43758.5453)-0.5)/255.0;
      gl_FragColor=vec4(clamp(c,0.0,1.0),1.0);}`,
    {tScene:{value:null},tBloom:{value:null},tShaft:{value:null},texel:{value:new THREE.Vector2()},bloom:{value:0.85},exposure:{value:1},sat:{value:1.1},contrast:{value:1.04},vig:{value:0.9},aa:{value:1},time:{value:0},
     useShaft:{value:1},shaftCol:{value:new THREE.Color()},lift:{value:new THREE.Color(0.012,0.018,0.035)},gain:{value:new THREE.Color(1.02,1.0,0.97)}});
  const pass=(m,target)=>{quad.material=m;renderer.setRenderTarget(target);renderer.render(quadScene,quadCam);};
  function alloc(){const pr=renderer.getPixelRatio();const w=Math.max(2,Math.floor(innerWidth*pr)),h=Math.max(2,Math.floor(innerHeight*pr));
    if(w===S.w&&h===S.h&&S.main)return;S.w=w;S.h=h;
    if(S.main){S.main.depthTexture.dispose();S.main.dispose();for(const t of[...S.down,...S.up,S.shaft])t&&t.dispose();}
    S.main=new THREE.WebGLRenderTarget(w,h,Object.assign({},rtOpts,{depthBuffer:true}));S.main.depthTexture=new THREE.DepthTexture(w,h);S.main.depthTexture.type=THREE.UnsignedIntType;
    S.down=[];S.up=[];let bw=w,bh=h;for(let i=0;i<S.levels;i++){bw=Math.max(1,bw>>1);bh=Math.max(1,bh>>1);S.down.push(new THREE.WebGLRenderTarget(bw,bh,rtOpts));S.up.push(new THREE.WebGLRenderTarget(bw,bh,rtOpts));}
    S.shaft=new THREE.WebGLRenderTarget(Math.max(1,w>>2),Math.max(1,h>>2),rtOpts);}
  const _s=new THREE.Vector3();
  /* sunDir (world) and sunCol come from the day cycle; night simply lowers the shaft strength */
  function render(sunDir,sunCol,shaftAmt){
    if(!S.on){renderer.setRenderTarget(null);renderer.render(scene,camera);return;}
    alloc();renderer.setRenderTarget(S.main);renderer.render(scene,camera);
    bright.uniforms.tIn.value=S.main.texture;bright.uniforms.thr.value=hdr?1.2:0.9;pass(bright,S.down[0]);
    for(let i=1;i<S.levels;i++){down.uniforms.tIn.value=S.down[i-1].texture;down.uniforms.px.value.set(1/S.down[i-1].width,1/S.down[i-1].height);pass(down,S.down[i]);}
    let src=S.down[S.levels-1];for(let i=S.levels-2;i>=0;i--){up.uniforms.tIn.value=src.texture;up.uniforms.tAdd.value=S.down[i].texture;up.uniforms.px.value.set(0.5/src.width,0.5/src.height);pass(up,S.up[i]);src=S.up[i];}
    let useShaft=0;
    if(S.level>=2&&sunDir&&shaftAmt>0.01){_s.copy(camera.position).addScaledVector(sunDir,400).project(camera);
      const vis=_s.z<1&&Math.abs(_s.x)<1.6&&Math.abs(_s.y)<1.6;
      if(vis){shaft.uniforms.tDepth.value=S.main.depthTexture;shaft.uniforms.sun.value.set(_s.x*0.5+0.5,_s.y*0.5+0.5);shaft.uniforms.aspect.value=S.w/S.h;
        shaft.uniforms.amt.value=shaftAmt*clamp(1.6-Math.max(Math.abs(_s.x),Math.abs(_s.y)),0,1);pass(shaft,S.shaft);useShaft=1;comp.uniforms.shaftCol.value.copy(sunCol);}}
    const u=comp.uniforms;u.tScene.value=S.main.texture;u.tBloom.value=src.texture;u.tShaft.value=S.shaft.texture;u.texel.value.set(1/S.w,1/S.h);u.useShaft.value=useShaft;u.aa.value=S.level>=1?1:0;u.time.value=(u.time.value+0.37)%100;
    pass(comp,null);}
  function setLevel(l){S.level=l;S.on=l>0;}
  return{render,setLevel,hdr,S,uniforms:comp.uniforms};
})();
