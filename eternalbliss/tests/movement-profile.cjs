const t=require('./browser.cjs');
const fs=require('node:fs');
(async()=>{const b=await t.launch();try{
for(const [name,profile,eye,horse,keys] of [
 ['desktop-overhead',{viewport:{width:1024,height:700}},false,false,['ArrowRight']],
 ['desktop-first',{viewport:{width:1024,height:700}},true,false,['ArrowRight']],
 ['mobile-overhead',{viewport:{width:844,height:390},hasTouch:true,isMobile:true},false,false,['ArrowRight','ArrowDown']],
 ['mobile-mounted-first',{viewport:{width:390,height:844},hasTouch:true,isMobile:true},true,true,['ArrowRight']]
]){
 if(process.env.EB_PROFILE && !name.includes(process.env.EB_PROFILE))continue;
 const p=await t.page(b,profile,(1767225600+4*480+240)*1000);
 if(horse){
   const panel=p.locator('#eb-actions-collapse');
   if(await panel.getAttribute('aria-expanded')==='false')await panel.tap();
   if(await p.locator('#eb-farlands-horse').isHidden())await p.tap('#eb-actions-more');
   await p.tap('#eb-farlands-horse');await p.tap('#eb-vehicle-accept');
   await p.tap('#eb-farlands-horse');await p.tap('#eb-vehicle-accept');
   t.assert(await p.evaluate(()=>ebHorseMounted&&ebFarlands.horseState().owned));
 }
 if(eye)await p.click('#eb-immersive-toggle');
 const route=await p.evaluate(({diag,horse})=>{
  const safe=(x,y)=>['grass','road','sand','stone','mud','snow'].includes(Farlands.sample(x,y));
  for(let dy=-260;dy<260;dy+=3)for(let dx=-280;dx<280;dx+=3){
   const x=Farlands.start.x+dx,y=Farlands.start.y+dy;
   if(Farlands.nearbySite(x,y))continue;
   let count=0;
   for(;count<90;count++){
    const a=x+count,c=y+(diag?count:0);
    if(!safe(a,c)||!safe(a+1,c)||diag&&!safe(a,c+1))break;
   }
   if(count<(horse?75:50))continue;
   applyAdventureMap(captureAdventureMap(),{...gameState.player,x,y,targetX:x,targetY:y});
   __view.eyeYaw=__view.eyeRenderYaw=0;
   return [x,y,count];
  }
 },{diag:keys.length===2,horse});
 t.assert(route,'free terrain for sustained movement');
 await p.waitForTimeout(2000);
 await p.evaluate(noSubmit=>{
   const v=__view;window.measure={frames:[],steps:[],errors:[]};let results;
   for(const name of ['updateTerrain','syncActors','renderFrame','streamTerrain']){
     const orig=v[name];if(!orig)continue;
     v[name]=function(...a){let start=performance.now();try{return orig.apply(this,a);}finally{if(results)results[name]=(results[name]||0)+performance.now()-start;}};
   }
   const render=v.renderer.render;
   v.renderer.render=function(...args){if(noSubmit)return;const start=performance.now();try{return render.apply(this,args);}finally{if(results)results.submit=performance.now()-start;}};
   let time;const draw=v.draw;
   v.draw=function(now){const start=performance.now();results={time:start,dt:time===undefined?null:start-time,x:gameState.player.x,y:gameState.player.y,layout:v.terrainSignature};time=start;draw.call(this,now);
    if(v.terrainPatches&&adventureTile(gameState.player.x,gameState.player.y)!=='water'){
        const key=Math.floor(gameState.player.y/24)*gameState.world.width+Math.floor(gameState.player.x/24);
        if(!v.terrainPatches.get(key)?.some(mesh=>mesh.visible))measure.errors.push('Missing terrain at the player');
    }
    results.newLayout=v.terrainSignature!==results.layout;results.hero=[v.hero.group.position.x,v.hero.group.position.z];results.cost=performance.now()-start;measure.frames.push(results);results=null;};
   const move=movePlayer;movePlayer=function(...a){const s=performance.now(),ok=move(...a);if(ok)measure.steps.push([s,performance.now()-s,gameState.player.x,gameState.player.y]);return ok;};
 },process.env.EB_NO_SUBMIT==='1');
 const duration=Number(process.env.EB_DURATION||10000);
 if(keys.length===2){
   const button=await p.locator('#eb-joystick').boundingBox();
   const x=button.x+button.width/2,y=button.y+button.height/2;
   const cdp=await p.context().newCDPSession(p);
   await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x,y}]});
   await cdp.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:x+button.width*.29,y:y+button.height*.29}]});
   await p.waitForTimeout(duration);
   await cdp.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});
 } else {
   await p.keyboard.down(keys[0]);
   await p.waitForTimeout(duration);
   await p.keyboard.up(keys[0]);
 }
 const data=await p.evaluate(()=>measure);
 t.assert(data.steps.length>=5,'accepted movement across the fixture');
 t.assert.deepEqual(data.errors,[]);
 t.assert.deepEqual(p.errors,[]);
 if(keys.length===2)t.assert(data.steps.some((step,i)=>i>0&&step[2]!==data.steps[i-1][2]&&step[3]!==data.steps[i-1][3]),'native diagonal steps');
 if(name==='desktop-first' && process.env.EB_NO_SUBMIT!=='1'){
   const yaw=await p.evaluate(()=>__view.eyeYaw);
   await p.keyboard.press('KeyQ');await p.waitForFunction(yaw=>__view.eyeYaw!==yaw,yaw);
   await t.shot(p,'movement-turn-'+t.sha.slice(0,8));
 }
 fs.writeFileSync(t.out+'/'+name+'.json',JSON.stringify({name,sha:t.sha,route,...data,errors:p.errors}));
 console.log(name,data.steps.length,data.frames.length,JSON.stringify(data.frames.filter(f=>f.newLayout).map(f=>[f.updateTerrain,f.syncActors,f.submit])));
 await p.close();
}
}finally{await b.close()}})().catch(e=>{console.error(e);process.exitCode=1});
