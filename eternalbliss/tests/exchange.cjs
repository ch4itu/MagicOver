// Exchange comparison: fixed summer clock and camera, unchanged footprint/entrance contract.
const t=require('./browser.cjs'),fs=require('node:fs'),path=require('node:path');
(async()=>{const b=await t.launch();try{
 const label=process.env.EB_LABEL||'exchange';let contract,p;
 for(const hour of [12,22]){
 if(p)await p.close();p=await t.page(b,undefined,(1767225600+4*480+hour*20)*1000);await p.click('#eb-capture-toggle');
 await p.evaluate(()=>{
  refreshWorldClimate();ebFarlands.refreshScene(true);const inn=buildings.find(b=>b.realmId==='rest-service-1');if(!inn)throw Error('named inn missing');
  gameState.player.x=inn.x+1;gameState.player.y=inn.y+4;gameState.player.targetX=gameState.player.x;gameState.player.targetY=gameState.player.y;renderWorld();
  __view.eyeYaw=0;__view.eyePitch=.08;__view.eyeRenderYaw=0;__view.eyeRenderPitch=.08;__view.lightLevel=worldClimate.daylight;__view.terrainSignature='';__view.lastSync=0;__view.cameraPosition.set(gameState.player.x,gameState.player.y);
 });await p.waitForTimeout(400);
 for(const eye of [false,true]){
 if(await p.evaluate(()=>__view.eyeMode)!==eye){await p.click('#eb-capture-toggle');await p.click('#eb-immersive-toggle');await p.click('#eb-capture-toggle');}const frame=await p.evaluate(()=>__view.renderer.info.render.frame);await p.waitForFunction(f=>__view.renderer.info.render.frame>=f+3,frame);
 await p.evaluate(()=>{__view.setActive(false);__view.active=true;__view.draw(performance.now());cancelAnimationFrame(__view.frame);__view.renderer.getContext().finish();});
 const color=await p.evaluate(()=>__view.scene.background.toArray());t.assert(color.every(v=>Number.isFinite(v)&&v>=0&&v<=1),'valid sky, no clock discontinuity');
 console.log('CAPTURE',hour,eye,await p.evaluate(()=>({climate:worldClimate,background:__view.scene.background.toArray(),fog:__view.scene.fog?.color.toArray(),camera:__view.eyeCamera.position.toArray()})));
 await t.shot(p,`${label}-${hour}-${eye?'fp':'overhead'}`);await p.evaluate(()=>__view.setActive(true));
 }
 }
 if(process.env.EB_CAPTURE_ONLY){t.record(label,'PASS recaptured warm WebGL frames in Clean View; capture-only baseline; entrance contract intentionally deferred to candidate');return;}
 await p.click('#eb-capture-toggle');
 contract=await p.evaluate(()=>{const inn=buildings.find(b=>b.realmId==='rest-service-1'),a=buildingFootprint(inn);return {realmId:inn.realmId,x:inn.x,y:inn.y,footprint:a,blocked:Array.from({length:a.h},(_,dy)=>Array.from({length:a.w},(_,dx)=>!canEntityMoveTo(a.x0+dx,a.y0+dy))),door:canEntityMoveTo(inn.x,inn.y+2)};});
 t.assert(contract.blocked.flat().every(Boolean));t.assert(contract.door);
 // Approach the existing door and use the actual first-person Use button, then Enter building.
 await p.evaluate(()=>{const inn=buildings.find(b=>b.realmId==='rest-service-1');gameState.player.x=inn.x+1;gameState.player.y=inn.y+3;__view.eyeYaw=0;__view.eyePitch=0;__view.eyeRenderYaw=0;__view.eyeRenderPitch=0;renderWorld();});await p.waitForTimeout(400);
 await p.click('#eb-eye-use');await p.click('.eb-enter-room');t.assert(await p.evaluate(()=>Boolean(buildingInterior)&&ebFarlands.inRealm()));
 await p.click('#eb-room-exit');t.assert(await p.evaluate(()=>!buildingInterior&&ebFarlands.surface()));
 t.assert.deepEqual(p.errors,[]);fs.writeFileSync(path.join(t.out,label+'-contract.json'),JSON.stringify(contract,null,2));
 t.record(label,{result:'PASS fixed summer noon/22:00 FP/overhead captures, footprint blocks, walkable approach, actual Use/Enter/Exit',contract});console.log('PASS',label,contract);
}finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1});
