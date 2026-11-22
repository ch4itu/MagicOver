const t=require('./browser.cjs');
const profiles=[['desktop',{viewport:{width:1280,height:800}}],['phone',{viewport:{width:390,height:844},isMobile:true,hasTouch:true}],['landscape',{viewport:{width:844,height:390},isMobile:true,hasTouch:true}]];
async function fixture(p,mounted,eye){
 await p.evaluate(({mounted})=>{
  if(gameState.inBattle)endBattle(false);ebFarlands.leave();
  ebFarlands.restore({version:1,active:true,horseOwned:mounted,horsePlace:{x:Farlands.start.x,y:Farlands.start.y+5},place:{x:Farlands.start.x,y:Farlands.start.y+5,sailing:0}});
  ebHorseMounted=mounted;enemies=[];gameState.player.hp=80;gameState.player.maxHp=100;gameState.player.mp=50;gameState.player.attack=10;gameState.player.magic=15;gameState.player.defense=9;gameState.inventory.healthPotions=2;
  Math.random=()=>.5;renderWorld();
 },{mounted});
 if(eye&&!await p.evaluate(()=>__view.eyeMode))await p.click('#eb-immersive-toggle');await p.waitForTimeout(450);
 await p.evaluate(()=>startBattle({x:gameState.player.x,y:gameState.player.y-1,name:'Fixture Goblin',class:'enemy-spawn enemy-goblin',hp:500,maxHp:500,attack:10,defense:3,xpReward:1,goldReward:1}));
 await p.waitForTimeout(450);
}
async function state(p){return p.evaluate(()=>{const v=__view,c=v.combat;const e=c?.model.group.position;const q=e?.clone();if(q){q.y+=.65;q.project(v.eyeCamera);}const hud=document.getElementById('eb-battle-hud').getBoundingClientRect(),canvas=v.canvas.getBoundingClientRect();return {eye:v.eyeActive(),mode:v.eyeMode,combatEye:c?.eye,hand:v.eyeHands?.visible,enemy:q?.toArray(),camera:v.eyeCamera.position.toArray(),hero:v.hero.group.position.toArray(),hud:{top:hud.top,bottom:hud.bottom},canvas:{top:canvas.top,height:canvas.height},hp:gameState.player.hp,mp:gameState.player.mp,foe:gameState.currentEnemy?.hp,potions:gameState.inventory.healthPotions};});}
module.exports={fixture,state,profiles};
if(require.main===module)(async()=>{const b=await t.launch();try{
 for(const [name,profile]of profiles)for(const mounted of [false,true]){
  if(process.env.EB_COMBAT_CASE&&process.env.EB_COMBAT_CASE!==name+':'+mounted)continue;
  const p=await t.page(b,profile);await fixture(p,mounted,true);
  let s=await state(p);console.log(name,mounted,'guard',s);t.assert(s.eye&&s.combatEye&&s.hand);t.assert(Math.abs(s.enemy[0])<.95&&Math.abs(s.enemy[1])<.95&&s.enemy[2]<1,'foe in eye frustum');t.assert(s.hud.bottom<=profile.viewport.height,'HUD inside viewport');
  await t.shot(p,`combat-${name}-${mounted}-guard`);
  const results=[];
  for(const action of ['attack','magic','heal']){
   const control=p.locator(`[data-eb-combat="${action}"]`);
   if(profile.hasTouch)await control.tap();else await control.click();
   await p.waitForTimeout(230);
   s=await state(p);t.assert(s.eye&&s.hand,action+' retains eye camera/feedback');
   // Capture after replies separately: paused-pose evidence is not action/timer evidence.
   await p.waitForFunction(()=>!ebCombatHUD.waiting());s=await state(p);results.push({action,hp:s.hp,mp:s.mp,foe:s.foe,potions:s.potions});t.assert(s.eye,'enemy reply retains eye');
  }
  t.assert.deepEqual(results,[{action:'attack',hp:73,mp:50,foe:487,potions:2},{action:'magic',hp:66,mp:38,foe:470,potions:2},{action:'heal',hp:93,mp:38,foe:470,potions:1}]);
  // Ending restores the same eye preference; overhead combat is checked separately.
  await p.evaluate(()=>endBattle(false));await p.waitForTimeout(350);t.assert(await p.evaluate(()=>__view.eyeActive()&&!__view.eyeHands.visible));
  t.assert.deepEqual(p.errors,[]);t.record(`FP combat ${name} mounted=${mounted}`,{result:'PASS',input:profile.hasTouch?'native Chromium touch':'mouse',results});await p.close();
 }
 // Presentation-only paths must retain exact rules with the same fixed RNG.
 if(!process.env.EB_COMBAT_CASE||process.env.EB_COMBAT_CASE==='overhead'){const p=await t.page(b);await fixture(p,false,false);t.assert(!(await state(p)).eye);await p.locator('[data-eb-combat="attack"]').click();await p.waitForFunction(()=>!ebCombatHUD.waiting());t.assert.deepEqual(await p.evaluate(()=>[gameState.player.hp,gameState.currentEnemy.hp]),[73,487]);await t.shot(p,'combat-overhead');t.assert.deepEqual(p.errors,[]);t.record('Overhead combat unchanged fixed-RNG attack/reply','PASS');await p.close();
 }
 console.log('PASS combat',process.env.EB_COMBAT_CASE||'all',t.sha);
}finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1});
