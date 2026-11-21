// A fresh-guest route using real keys/buttons. Read-only inspection plans paths; no position, stats,
// inventory, RNG, story flags, terrain or clock changes after startup. Resume only from its recorded local save.
const t=require('./browser.cjs'),fs=require('node:fs'),path=require('node:path');
const live=process.env.EB_LIVE_JOURNEY==='1';
const checkpoint=path.join(t.out,'natural-harvest-storage.json'),events=path.join(t.out,'natural-harvest-events.jsonl');
(async()=>{const b=await t.launch();let p;try{
 const profile={viewport:live?{width:900,height:540}:{width:1280,height:800}};if(process.env.EB_ROUTE_RESUME)profile.storageState=checkpoint;
 p=await b.newPage(profile);p.errors=[];p.on('pageerror',e=>p.errors.push(e.message));if(!live)await p.clock.setFixedTime(new Date((1767225600+4*480+240)*1000));
 const html=fs.readFileSync(process.env.EB_SOURCE||path.join(__dirname,'../index.html'),'utf8');await p.route('**/eternalbliss/',r=>r.fulfill({contentType:'text/html',body:html.replace('view = new WorldView(canvas, layer, world);','view = new WorldView(canvas, layer, world); window.__view=view;')}));
 await p.goto('http://127.0.0.1:8091/eternalbliss/',{waitUntil:'domcontentloaded'});await p.click('#eb-menu-solo');await p.waitForFunction(()=>window.__view?.active&&ebFarlands.inRealm());
 const started=Date.now();
 const state=()=>p.evaluate(()=>({x:gameState.player.x,y:gameState.player.y,hp:gameState.player.hp,mp:gameState.player.mp,gold:gameState.inventory.gold,xp:gameState.player.xp,keys:gameState.inventory.keys,room:!!buildingInterior,day:worldClimate.day,clock:worldClimate.time,moves:gameState.player.moveCount,afloat:gameState.sailingMoves,flags:['met','key','gate','guardian','relic','reward'].map(k=>[k,ebFarlands.storyFlag('quest-vault-'+k)]),discoveries:[...ebFarlands.discoveries()]}));
 async function mark(label){await p.waitForTimeout(1300);const s=await state();console.log(label,JSON.stringify(s));fs.appendFileSync(events,JSON.stringify({time:new Date().toISOString(),sha:t.sha,label,elapsedMs:Date.now()-started,state:s})+'\n');await p.context().storageState({path:checkpoint});await t.shot(p,'natural-'+label+'-'+t.sha.slice(0,8));}
 const flag=k=>p.evaluate(k=>ebFarlands.storyFlag('quest-vault-'+k),k);
 async function blur(){await p.evaluate(()=>document.activeElement?.blur());}
 async function face(dx,dy){const desired=Math.atan2(-dx,-dy);for(let n=0;n<16;n++){const delta=await p.evaluate(y=>Math.atan2(Math.sin(y-__view.eyeYaw),Math.cos(y-__view.eyeYaw)),desired);if(Math.abs(delta)<=.13)return;await p.keyboard.press(delta>0?'KeyQ':'KeyE');}throw Error('turn did not settle');}
 async function fight(){for(let turn=0;turn<30;turn++){const s=await p.evaluate(()=>({battle:gameState.inBattle,hp:gameState.player.hp,mp:gameState.player.mp,potions:gameState.inventory.healthPotions,enemyHp:gameState.currentEnemy?.hp,name:gameState.currentEnemy?.name}));if(!s.battle)return;if(s.enemyHp<=0){await p.waitForFunction(()=>!gameState.inBattle);return;}t.assert(s.hp>0,'natural guest must survive');const action=s.hp<35&&s.potions?'heal':s.mp>=12?'magic':'attack';await p.locator(`[data-eb-combat="${action}"]:enabled`).click();console.log('COMBAT',s.name,turn,action);await p.waitForFunction(()=>!gameState.inBattle||gameState.currentEnemy?.hp<=0||!ebCombatHUD.waiting());}throw Error('combat turn limit');}
 async function walk(gx,gy,{stopBattle=false}={}){await blur();if(await p.evaluate(()=>__view.eyeMode))await face(0,-1);let stuck=0;for(let step=0;step<450;step++){
  const r=await p.evaluate(({gx,gy})=>{if(gameState.inBattle)return {battle:true};const px=gameState.player.x,py=gameState.player.y;if(px===gx&&py===gy)return {done:true};const minX=Math.min(px,gx)-22,maxX=Math.max(px,gx)+22,minY=Math.min(py,gy)-22,maxY=Math.max(py,gy)+22,queue=[[px,py]],parent=new Map([[px+','+py,null]]);for(let i=0;i<queue.length&&i<18000;i++){const[x,y]=queue[i];for(const[dx,dy,key]of [[0,-1,'ArrowUp'],[1,0,'ArrowRight'],[0,1,'ArrowDown'],[-1,0,'ArrowLeft']]){const nx=x+dx,ny=y+dy,k=nx+','+ny;if(parent.has(k)||nx<minX||nx>maxX||ny<minY||ny>maxY||!canEntityMoveTo(nx,ny)||enemies.some(e=>Math.hypot(nx-e.x,ny-e.y)<(buildingInterior?1:2)))continue;parent.set(k,{previous:x+','+y,key});if(nx===gx&&ny===gy){let v=k;while(parent.get(v).previous!==px+','+py)v=parent.get(v).previous;return {key:parent.get(v).key,x:px,y:py};}queue.push([nx,ny]);}}return {unreachable:true,x:px,y:py,gx,gy};},{gx,gy});
  if(r.done)return;if(r.battle){if(stopBattle)return;await fight();continue;}t.assert(!r.unreachable,JSON.stringify(r));await p.keyboard.press(r.key);await p.waitForTimeout(170);const same=await p.evaluate(({x,y})=>gameState.player.x===x&&gameState.player.y===y,r);stuck=same?stuck+1:0;t.assert(stuck<6,'actual movement blocked at '+JSON.stringify(r));
 }throw Error('walk limit');}
 const initial=await state();
 if(!process.env.EB_ROUTE_RESUME){
  t.assert(initial.flags.every(([,earned])=>!earned));
  t.assert.equal(await p.evaluate(()=>gameState.player.moveCount),0);
 }
 console.log(process.env.EB_ROUTE_RESUME?'RESUME recorded natural local save':'FRESH guest',JSON.stringify(initial));
 if(!await flag('met')){
  const goal=p.locator('#eb-goal-chip').filter({hasText:'Speak to Mira by the square'});
  await goal.click();
  const row=p.locator('#modalContent > div').filter({has:p.getByText('Mira, the Cartographer',{exact:true})});
  await row.getByRole('button',{name:/^(Go & talk|Talk)$/}).click();await p.locator('#modalContent').filter({hasText:'The Silent Harvest'}).waitFor();await p.locator('#interactionModal button[onclick="closeModal()"]').click();t.assert(await flag('met'));await mark('mira-start');
  await p.locator('#eb-goal-chip').filter({hasText:'Find the ward key at Millbend'}).click();
  t.assert.equal(await p.evaluate(()=>ebFarlands.pinned()),'mill');
 }
 if(!await flag('key')){const mill=await p.evaluate(()=>{const s=Farlands.byId.get('mill');return {x:s.x-3,y:s.y+2};});await walk(mill.x,mill.y);t.assert(await flag('key'),'actual creekside key pickup');await mark('mill-key');}
 if(!await flag('relic')){
  if(!await p.evaluate(()=>buildingInterior?.questDungeon)){if(await p.evaluate(()=>__view.eyeMode))await p.click('#eb-immersive-toggle');const at=await p.evaluate(()=>{const s=Farlands.byId.get('toll');return {x:s.x+2,y:s.y};});await walk(at.x,at.y);await p.keyboard.press('Space');await p.locator('.eb-enter-room').click();await p.waitForFunction(()=>buildingInterior?.questDungeon);await mark('archive-entry');}
  if(!await p.evaluate(()=>__view.eyeMode))await p.click('#eb-immersive-toggle');
  if(!await flag('gate')){await walk(7,6);await face(1,0);await p.click('#eb-eye-use');t.assert(await flag('gate'));await mark('archive-gate');}
  if(!await flag('guardian')){await walk(10,7,{stopBattle:true});if(!await p.evaluate(()=>gameState.inBattle)){await face(1,0);await p.click('#eb-eye-use');}await p.waitForFunction(()=>gameState.inBattle);await fight();t.assert(await flag('guardian'));await mark('warden-defeated');}
  await walk(12,3);await face(1,0);await p.click('#eb-eye-use');t.assert(await flag('relic'));await mark('bell-recovered');
 }
 if(await p.evaluate(()=>buildingInterior)){if(!await p.evaluate(()=>__view.eyeMode))await p.click('#eb-immersive-toggle');await walk(4,12);await face(0,1);await p.click('#eb-eye-use');await p.waitForFunction(()=>!buildingInterior);await mark('archive-exit');}
 if(!await flag('reward')){if(await p.evaluate(()=>__view.eyeMode))await p.click('#eb-immersive-toggle');const rest=await p.evaluate(()=>{const s=Farlands.byId.get('rest');return {x:s.x,y:s.y+2};});await walk(rest.x,rest.y);const mira=await p.evaluate(()=>{const n=npcs.find(n=>n.name==='Mira, the Cartographer');return {x:n.x,y:n.y};});await walk(mira.x,mira.y+1);const before=await state();await p.keyboard.press('Space');await p.locator('#modalContent').filter({hasText:'The Harvest Bell restored!'}).waitFor();const after=await state();t.assert.equal(after.gold,before.gold+65);t.assert(await flag('reward'));await mark('mira-reward');await p.locator('#interactionModal button[onclick="closeModal()"]').click();await blur();await p.keyboard.press('Space');t.assert.equal((await state()).gold,after.gold);await p.locator('#interactionModal button[onclick="closeModal()"]').click();}
 await p.locator('#eb-goal-chip').waitFor({state:'visible'});
 const next=await p.locator('#eb-goal-chip').textContent();
 t.assert(next.trim() && !next.includes('Bring the Harvest Bell to Mira'),'completed quest hands off to another objective');
 if(!await p.evaluate(()=>ebFarlands.horseState().owned)){
  const panel=p.locator('#eb-actions-collapse');
  if(await panel.getAttribute('aria-expanded')==='false')await panel.click();
  const horse=p.locator('#eb-farlands-horse');
  if(await horse.isHidden())await p.click('#eb-actions-more');
  const gold=(await state()).gold;
  await horse.click();await p.click('#eb-vehicle-accept');
  t.assert.equal((await state()).gold,gold-75);
  await horse.click();await p.click('#eb-vehicle-accept');
  t.assert(await p.evaluate(()=>ebHorseMounted));
  const step=await p.evaluate(()=>{
   const {x,y}=gameState.player;
   return [[x,y+1],[x+1,y],[x,y-1],[x-1,y]].find(([a,b])=>canEntityMoveTo(a,b));
  });
  t.assert(step,'legal first mounted step');
  await walk(...step);
  await mark('first-ride');
 }
 const orchardFlag=part=>p.evaluate(part=>ebFarlands.storyFlag('rq-'+part+'-f-rest'),part);
 async function speakTo(name){
  if(live) await p.waitForFunction(name=>npcs.some(n=>n.name===name && !n.indoors &&
   Math.hypot(n.x-gameState.player.x,n.y-gameState.player.y)<12),name,{timeout:500000});
  await p.click('#eb-farlands-people');
  const row=p.locator('#modalContent > div').filter({has:p.getByText(name,{exact:true})});
  await row.getByRole('button',{name:/^(Go & talk|Talk)$/}).click();
  await p.locator('#modalTitle').filter({hasText:name}).waitFor();
  const tab=p.getByRole('tab',{name:/Quests/});
  if(await tab.isVisible())await tab.click();
 }
 const close=()=>p.locator('#interactionModal').getByRole('button',{name:'Close',exact:true}).click();
 if(!await orchardFlag('done')){
  const rest=await p.evaluate(()=>[Farlands.start.x,Farlands.start.y]);
  if(!await orchardFlag('taken')){
   if(live){
    await walk(...rest);
    await speakTo('Nessa, the Orchard Keeper');
   }else{
    await p.locator('#eb-goal-chip').filter({hasText:'Ask Nessa about the first picking'}).click();
    const row=p.locator('#modalContent > div').filter({has:p.getByText('Nessa, the Orchard Keeper',{exact:true})});
    await row.getByRole('button',{name:/^(Go & talk|Talk)$/}).click();
    await p.getByRole('tab',{name:/Quests/}).click();
   }
   await p.locator('#modalContent section').filter({hasText:'The first picking'}).getByRole('button',{name:'Accept quest'}).click();
   await close();
  }
  if(!await p.evaluate(()=>classicView()))await p.click('#eb-view-toggle');
  const orchard=await p.evaluate(()=>{const s=Farlands.byId.get('orchard');return [s.x,s.y];});
  await walk(...orchard);
  for(const [index,name] of ['North orchard tally','South orchard tally'].entries()){
   if(await orchardFlag('step'+(index+1)))continue;
   const approach=await p.evaluate(name=>{
    const prop=[...Farlands.authoredProps.values()].find(p=>p.name===name);
    for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){
     const x=prop.x+dx,y=prop.y+dy;
     if(Farlands.propNear(x,y)===prop&&canEntityMoveTo(x,y)&&
       !buildings.some(b=>buildingDistance(b,x,y)<=2)&&!npcs.some(n=>Math.hypot(n.x-x,n.y-y)<=1.5))
         return [x,y];
    }
   },name);
   t.assert(approach,'walkable approach to '+name);
   await walk(...approach);
   await p.keyboard.press('Space');
   await p.locator('#modalTitle').filter({hasText:name}).waitFor();
   t.assert(await orchardFlag('step'+(index+1)));
   await mark('orchard-tally-'+(index+1));
   await close();
  }
  await walk(...rest);
  await speakTo('Nessa, the Orchard Keeper');
  const gold=(await state()).gold;
  await p.getByRole('button',{name:'Pick the south rows'}).click();
  t.assert.equal((await state()).gold,gold,'wrong rows do not pay');
  t.assert.equal(await orchardFlag('done'),false);
  await p.getByRole('button',{name:'Pick the north rows'}).click();
  t.assert.equal((await state()).gold,gold+22);
  t.assert(await orchardFlag('done'));
  t.assert.equal(await p.getByRole('button',{name:'Pick the north rows'}).count(),0);
  await mark('nessa-reward');
  await close();
 }
 if(live){
  const site=async id=>p.evaluate(id=>{const s=Farlands.byId.get(id);return [s.x,s.y];},id);
  const rest=await site('rest');
  const sounding=part=>p.evaluate(part=>ebFarlands.storyFlag('rq-'+part+'-g-rest'),part);
  const hale='Hale, the Wheelwright';
  await walk(...rest);
  if(!await sounding('taken')){
   await speakTo(hale);
   await p.locator('#modalContent section').filter({hasText:'Sounding Mill Creek'}).getByRole('button',{name:'Accept quest'}).click();
   await close();
  }
  const board=await p.evaluate(()=>Farlands.creekPilotage.board);
  await walk(...board);
  if(!await sounding('step1')){
   await p.keyboard.press('Space');
   await p.getByRole('status').filter({hasText:'Sounding started'}).waitFor();
   await close();
  }
  if(await p.evaluate(()=>classicView()))await p.click('#eb-view-toggle');
  await p.click('#eb-immersive-toggle');
  await p.waitForFunction(()=>__view.eyeActive()&&__view.coastalBoats.has('63,60'));
  const berth=await p.evaluate(()=>{
   const view=__view, {model,position}=view.coastalBoats.get('63,60');
   model.updateWorldMatrix(true,true);
   const centre=new THREE.Box3().setFromObject(model.children[0]).getCenter(new THREE.Vector3());
   centre.sub(view.eyeCamera.position);
   view.eyeYaw=view.eyeRenderYaw=Math.atan2(-centre.x,-centre.z);
   view.eyePitch=view.eyeRenderPitch=Math.atan2(centre.y,Math.hypot(centre.x,centre.z));
   return [position.x,position.y];
  });
  await p.click('#eb-eye-use');
  await p.locator('#eb-vehicle-title').filter({hasText:'Embark'}).waitFor();
  await p.click('#eb-vehicle-accept');
  await p.waitForFunction(()=>gameState.sailingMoves>0);
  const boarded=await state();
  t.assert.deepEqual([boarded.x,boarded.y],berth);
  await p.click('#eb-immersive-toggle');
  const sail=async (key,x,y)=>{
   await blur();
   await p.keyboard.press(key);
   await p.waitForFunction(({x,y})=>gameState.player.x===x && gameState.player.y===y,{x,y},{timeout:5000});
   await p.waitForTimeout(170);
  };
  for(let i=1;i<=3;i++)await sail('ArrowLeft',berth[0]-i,berth[1]);
  t.assert(await sounding('step2'));
  await mark('sounding-afloat');
  await p.reload({waitUntil:'domcontentloaded'});
  await p.click('#eb-menu-solo');
  await p.waitForFunction(()=>ebFarlands.surface()&&gameState.sailingMoves>0);
  t.assert(await sounding('step2'));
  for(let i=2;i>=0;i--)await sail('ArrowRight',berth[0]-i,berth[1]);
  await sail('ArrowDown',berth[0],berth[1]+1);
  const panel=p.locator('#eb-actions-collapse');
  if(await panel.getAttribute('aria-expanded')==='false')await panel.click();
  await p.click('#eb-farlands-dock');
  await p.click('#eb-vehicle-accept');
  await p.waitForFunction(()=>gameState.sailingMoves===0);
  if(!await p.evaluate(()=>classicView()))await p.click('#eb-view-toggle');
  await walk(...rest);
  await speakTo(hale);
  const gold=(await state()).gold;
  await p.getByRole('button',{name:'Report back · collect 20 gold'}).click();
  t.assert.equal((await state()).gold,gold+20);
  t.assert(await sounding('done'));
  t.assert.equal(await p.getByRole('button',{name:'Report back · collect 20 gold'}).count(),0);
  await close();
  await mark('hale-reward');

  const neighbors=['mill','orchard','toll'];
  const travelers=['Ivo, the Millwright','Tamsin, the Herbalist','Wren, a Pilgrim'];
  const loops=[];
  do {
   const index=loops.length%neighbors.length;
   const before=await state();
   await walk(...await site(neighbors[index]));
   await walk(...rest);
   const available=await p.evaluate(names=>names.filter(name=>npcs.some(n=>n.name===name&&!n.indoors&&
     Math.hypot(n.x-gameState.player.x,n.y-gameState.player.y)<10)),travelers);
   if(available.length){await speakTo(available[loops.length%available.length]);await close();}
   const after=await state();
   t.assert(after.moves>before.moves,'travel leg accepted movement');
   loops.push({site:neighbors[index],before:before.clock,after:after.clock,day:after.day,moves:after.moves-before.moves,residents:available});
   await mark('clock-travel-'+loops.length);
  } while(Date.now()-started<30*60*1000);
  t.assert((await state()).day-initial.day>=3,'the unmodified game clock advanced at least three game days');
  t.record('Advancing-clock solo journey',{elapsedMs:Date.now()-started,initial,final:await state(),loops});
 }
 await p.reload({waitUntil:'domcontentloaded'});await p.click('#eb-menu-solo');await p.waitForFunction(()=>ebFarlands.surface());t.assert(await orchardFlag('done'));t.assert(await flag('reward'));t.assert(await flag('relic'));t.assert(await flag('guardian'));t.assert(await flag('gate'));t.assert(await p.evaluate(()=>ebFarlands.horseState().owned));t.assert.deepEqual(p.errors,[]);await mark('complete-reloaded');t.record('natural Harvest Bell route',{result:'PASS fresh guest onboarding, Atlas pin, actual walking, Harvest Bell quest and combat, horse purchase and ride, both orchard tallies, Nessa’s once-only reward and reload; no injected progress/position/stats/RNG',resumed:!!process.env.EB_ROUTE_RESUME,elapsedMs:Date.now()-started,next,state:await state()});
}catch(e){if(p){await t.shot(p,'natural-blocker-'+Date.now());await p.context().storageState({path:checkpoint});}throw e;}finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1});
