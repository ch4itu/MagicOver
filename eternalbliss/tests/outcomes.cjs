const t=require('./browser.cjs'),{fixture,state}=require('./combat.cjs');
(async()=>{const b=await t.launch();try{
 for(const [name,profile]of [['desktop',{viewport:{width:1280,height:800}}],['phone',{viewport:{width:390,height:844},isMobile:true,hasTouch:true}]]){
 const modes=[];
 for(const eye of [false,true]){
  const p=await t.page(b,profile);if(eye)await p.click('#eb-immersive-toggle');const results=[];
  for(const action of ['attack','magic','heal']){await fixture(p,false,false);await p.locator(`[data-eb-combat="${action}"]`).click();await p.waitForTimeout(2900);const s=await state(p);results.push([action,s.hp,s.mp,s.foe,s.potions]);}
  // Victory: actual Attack; record rewards before the battle closure timer.
  await p.evaluate(()=>{endBattle(false);gameState.inventory.gold=10;gameState.player.xp=0;gameState.player.level=1;gameState.player.xpToNext=1000;startBattle({x:gameState.player.x,y:gameState.player.y-1,name:'Victory fixture',class:'enemy-spawn enemy-goblin',hp:1,maxHp:1,attack:10,xpReward:7,goldReward:9});});
  await p.locator('[data-eb-combat="attack"]').click();await p.waitForTimeout(300);results.push(await p.evaluate(()=>['victory',gameState.currentEnemy.hp,gameState.player.xp,gameState.inventory.gold]));
  await t.shot(p,`outcome-${name}-${eye}-victory`);await p.waitForFunction(()=>!gameState.inBattle);
  // Failed flee and death use actual buttons. Death is a downed presentation before respawn.
  await fixture(p,false,false);
  await p.evaluate(()=>{Math.random=()=>.9;gameState.player.hp=1;});await p.locator('[data-eb-combat="flee"]').click();await p.waitForTimeout(2300);
  results.push(await p.evaluate(()=>['death',gameState.player.hp,ebCombatHUD.downed()]));await t.shot(p,`outcome-${name}-${eye}-death`);
  if(eye)t.assert((await state(p)).eye,'death stays FP');
  await p.waitForFunction(()=>!gameState.inBattle);
  await fixture(p,false,false);
  await p.locator('[data-eb-combat="flee"]').click();results.push(await p.evaluate(()=>['escaped',gameState.inBattle]));
  t.assert.deepEqual(p.errors,[]);modes.push(results);await p.close();
 }
 t.assert.deepEqual(modes[0],modes[1]);t.record('outcomes '+name,{result:'PASS identical overhead/FP fixed-RNG results',outcomes:modes[0]});console.log('PASS outcomes',name,modes[0]);
 }
}finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1});
