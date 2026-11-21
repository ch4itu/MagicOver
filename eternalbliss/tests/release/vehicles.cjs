const t=require('../browser.cjs');
(async()=>{const b=await t.launch();try{for(const[name,profile]of [['desktop',undefined],['phone',{viewport:{width:390,height:844},hasTouch:true,isMobile:true}],['landscape',{viewport:{width:844,height:390},hasTouch:true,isMobile:true}]]){if(process.env.EB_VEHICLE_CASE&&process.env.EB_VEHICLE_CASE!==name)continue;const p=await t.page(b,profile,(1767225600+4*480+240)*1000);const click=s=>profile?p.tap(s):p.click(s);async function openPanel(){const panel=p.locator('#eb-actions-collapse');if(await panel.getAttribute('aria-expanded')==='false')await click('#eb-actions-collapse');}
 async function horse(){await openPanel();if(await p.locator('#eb-farlands-horse').isHidden())await click('#eb-actions-more');await click('#eb-farlands-horse');await p.locator('#eb-vehicle-confirm').waitFor({state:'visible'});}
 const gold=await p.evaluate(()=>gameState.inventory.gold);await horse();await click('#eb-vehicle-cancel');t.assert.equal(await p.evaluate(()=>gameState.inventory.gold),gold);t.assert.equal(await p.evaluate(()=>ebFarlands.horseState().owned),false);await horse();await click('#eb-vehicle-accept');t.assert.deepEqual(await p.evaluate(()=>[gameState.inventory.gold,ebFarlands.horseState().owned]),[gold-75,true]);await horse();await click('#eb-vehicle-accept');t.assert(await p.evaluate(()=>ebHorseMounted));await click('#eb-immersive-toggle');const start=await p.evaluate(()=>({x:gameState.player.x,y:gameState.player.y,moves:gameState.player.moveCount}));await p.evaluate(()=>document.activeElement?.blur());for(let n=0;n<2;n++){await p.keyboard.press('ArrowRight');await p.waitForTimeout(140);}await p.waitForFunction(()=>Math.hypot(__view.hero.group.position.x-gameState.player.x,__view.hero.group.position.z-gameState.player.y)<.01);t.assert.deepEqual(await p.evaluate(()=>[gameState.player.x,gameState.player.y,gameState.player.moveCount]),[start.x+2,start.y,start.moves+2]);await horse();await click('#eb-vehicle-accept');t.assert(!await p.evaluate(()=>ebHorseMounted));const parked=await p.evaluate(()=>ebFarlands.horseState());await p.reload({waitUntil:'domcontentloaded'});await p.click('#eb-menu-solo');await p.waitForFunction(()=>ebFarlands.surface());t.assert.deepEqual(await p.evaluate(()=>ebFarlands.horseState()),parked);t.assert.equal(await p.evaluate(()=>gameState.inventory.gold),gold-75);
 // Mill Creek's generated boat sits a water tile off the bank, within Use range.
 const use = click;
 const mooring = await p.evaluate(()=>{
   if(__view.coastalBoats.has('63,61')||!__view.mooringSpots.has('63,61'))
     throw Error('Pocket south of Millbend still exposes a trapped generated boat');
   const vessel = __view.coastalBoats.get('63,60');
   if(!vessel)throw Error('Missing generated Mill Creek mooring');
   const {x,y}=vessel.position;
   const land={x,y:y+2};
   if(adventureTile(x,y)!=='water'||adventureTile(x,y+1)!=='water'||!canEntityMoveTo(land.x,land.y))
     throw Error('Generated boat has no legal approach to the expected bank');
   const accessible=new Set([x+','+y]), queue=[[x,y]];
   for(const [wx,wy] of queue){
     for(const [dx,dy] of [[1,0],[-1,0],[0,1],[0,-1]]){
       const nx=wx+dx,ny=wy+dy,key=nx+','+ny;
       if(Math.hypot(nx-x,ny-y)>8||accessible.has(key)||adventureTile(nx,ny)!=='water'||solidStructure(nx,ny))continue;
       accessible.add(key);queue.push([nx,ny]);
     }
   }
   if(!queue.some(([wx,wy])=>Math.hypot(wx-x,wy-y)>7))throw Error('Mooring water is disconnected from the creek');
   applyAdventureMap(captureAdventureMap(),{...gameState.player,...land,targetX:land.x,targetY:land.y});
   return {x,y,land,wetTiles:queue.length};
 });
 await use('#eb-immersive-toggle');
 await p.waitForFunction(({land})=>__view.eyeActive()&&__view.eyeCamera.position.distanceTo(new THREE.Vector3(land.x,1.135,land.y))<.05,mooring);
 await p.evaluate(()=>{
   const view=__view, boat=view.coastalBoats.get('63,60').model;
   boat.updateWorldMatrix(true,true);
   const aim=new THREE.Box3().setFromObject(boat.children[0]).getCenter(new THREE.Vector3()).sub(view.eyeCamera.position);
   view.eyeYaw=view.eyeRenderYaw=Math.atan2(-aim.x,-aim.z);
   view.eyePitch=view.eyeRenderPitch=Math.atan2(aim.y,Math.hypot(aim.x,aim.z));
 });
 const ray = await p.waitForFunction(()=>{
   const view=__view, boat=view.coastalBoats.get('63,60').model;
   const ray=new THREE.Raycaster();
   ray.setFromCamera(new THREE.Vector2(),view.eyeCamera);
   const hull=ray.intersectObject(boat,true)[0];
   if(!hull||hull.distance>5.5)return false;
   return {distance:hull.distance, actor:ray.intersectObjects(view.targets,false)[0]?.distance??null};
 });
 console.log('MOORING RAY',name,JSON.stringify({mooring, hit:await ray.jsonValue()}));
 await use('#eb-eye-use');
 await p.locator('#eb-vehicle-title').filter({hasText:'Embark'}).waitFor();
 await click('#eb-vehicle-accept');await p.waitForFunction(()=>gameState.sailingMoves===15);console.log('BOARDED',name,mooring,await p.evaluate(()=>[gameState.player.x,gameState.player.y]));t.assert.deepEqual(await p.evaluate(()=>[gameState.player.x,gameState.player.y]),[mooring.x,mooring.y],'Use selects the intended real mooring');t.assert(!await p.evaluate(()=>ebHorseMounted));t.assert.equal(await p.evaluate(()=>adventureTile(gameState.player.x,gameState.player.y)),'water');await click('#eb-immersive-toggle');
 const water=await p.evaluate(()=>{const x=gameState.player.x,y=gameState.player.y;for(const[dx,dy,key,back]of [[0,-1,'ArrowUp','ArrowDown'],[1,0,'ArrowRight','ArrowLeft'],[0,1,'ArrowDown','ArrowUp'],[-1,0,'ArrowLeft','ArrowRight']])if(adventureTile(x+dx,y+dy)==='water'&&!solidStructure(x+dx,y+dy))return {key,back};throw Error('no water step');});await p.evaluate(()=>document.activeElement?.blur());await p.keyboard.press(water.key);await p.waitForTimeout(140);await p.keyboard.press(water.back);t.assert.equal(await p.evaluate(()=>gameState.sailingMoves),15,'Farlands ship remains unlimited');await p.waitForTimeout(2000);await p.reload({waitUntil:'domcontentloaded'});await p.click('#eb-menu-solo');await p.waitForFunction(()=>ebFarlands.surface());t.assert.equal(await p.evaluate(()=>gameState.sailingMoves),15);t.assert.equal(await p.evaluate(()=>adventureTile(gameState.player.x,gameState.player.y)),'water');await p.evaluate(()=>document.activeElement?.blur());
 await p.keyboard.press('ArrowDown');
 await p.waitForFunction(({x,y})=>gameState.player.x===x&&gameState.player.y===y+1,mooring);
 await openPanel();await click('#eb-farlands-dock');await click('#eb-vehicle-accept');await p.waitForFunction(()=>gameState.sailingMoves===0);t.assert(await p.evaluate(()=>canEntityMoveTo(gameState.player.x,gameState.player.y)));t.assert(await p.evaluate(()=>ebFarlands.horseState().owned));t.assert(await p.evaluate(()=>!!window.__ebParkedBoat));await t.shot(p,'rc-vehicles-'+name+'-'+t.sha.slice(0,8));t.assert.deepEqual(p.errors,[]);t.record('release actual vehicles '+name,{result:'PASS actual horse buy/cancel/mount/two accepted FP steps/dismount/ownership reload; fixture approach to real mooring, actual FP Use/embark, water keys/unlimited range, afloat reload and dock with owned horse',mooring});console.log('PASS vehicles',name);await p.close();}}finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1});
