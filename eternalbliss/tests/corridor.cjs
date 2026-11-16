// New regression: the dragon's guard distance exceeds the old 2.1-tile clearance check.
const t=require('./browser.cjs'),{fixture}=require('./combat.cjs');
(async()=>{const b=await t.launch();try{
 for(const [name,profile]of [['desktop',{viewport:{width:1280,height:800}}],['phone',{viewport:{width:390,height:844},hasTouch:true,isMobile:true}]]){
 if(process.env.EB_CORRIDOR_CASE&&process.env.EB_CORRIDOR_CASE!==name)continue;const p=await t.page(b,profile);await fixture(p,false,true);
 const before=await p.evaluate(()=>{endBattle(false);const inn=buildings.find(b=>b.realmId==='rest-service-0');Object.assign(gameState.player,{x:inn.x,y:inn.y+3,targetX:inn.x,targetY:inn.y+3});
 const v=__view;v.hero.group.position.set(gameState.player.x,.055,gameState.player.y);v.hero.lastX=gameState.player.x;v.hero.lastZ=gameState.player.y;v.eyeYaw=0;v.eyeRenderYaw=0;renderWorld();
 return {x:gameState.player.x,y:gameState.player.y,hp:gameState.player.hp,mp:gameState.player.mp};});
 await p.waitForTimeout(200);await p.evaluate(()=>startBattle({name:'Dragon corridor fixture',class:'enemy-spawn enemy-dragon',x:gameState.player.x,y:gameState.player.y-1,hp:500,maxHp:500,attack:10,xpReward:1,goldReward:1}));
 await p.waitForFunction(()=>__view.combat?.foe.name==='Dragon corridor fixture');await p.waitForTimeout(300);
 const stage=await p.evaluate(()=>{const c=__view.combat,e=c.model.group.position;return {eye:__view.eyeActive(),rest:c.rest,dir:c.dir.toArray(),enemy:e.toArray(),blocked:!canEntityMoveTo(Math.round(e.x),Math.round(e.z)),player:[gameState.player.x,gameState.player.y],hp:gameState.player.hp,mp:gameState.player.mp};});
 console.log(name,JSON.stringify(stage));await t.shot(p,'corridor-'+name+'-'+t.sha.slice(0,8));
 t.assert(!stage.blocked,'dragon guard center must not occupy the inn footprint');t.assert(stage.eye);t.assert.deepEqual(stage.player,[before.x,before.y]);t.assert.equal(stage.hp,before.hp);t.assert.equal(stage.mp,before.mp);
 await p.locator('[data-eb-combat="attack"]').click();await p.waitForFunction(()=>!ebCombatHUD.waiting());
 t.assert.deepEqual(await p.evaluate(()=>[gameState.player.hp,gameState.currentEnemy.hp]),[73,487]);
 // A clear field retains the established guard distance and forward-facing view.
 await fixture(p,false,true);const open=await p.evaluate(()=>({rest:__view.combat.rest,dir:__view.combat.dir.toArray()}));t.assert.equal(open.rest,2.1);t.assert(open.dir.every((v,i)=>Math.abs(v-[0,0,-1][i])<1e-12),'open field keeps forward aim (signed zero is equivalent)');
 t.assert.deepEqual(p.errors,[]);t.record('dragon corridor '+name,{result:'PASS real inn footprint avoided, authoritative position/HP/MP unchanged by staging, actual attack/reply rules retained; open-field guard unchanged',stage});await p.close();
 }
}finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1});
