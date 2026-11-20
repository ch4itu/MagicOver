// Deterministic final-state fixture, not evidence of the natural quest route.
const {launch}=require('./browser.cjs');
const assert=require('node:assert/strict'),fs=require('node:fs');
const out=process.env.EB_EVIDENCE||'/workspace/.magic-over-setup/eternalbliss-evidence';
(async()=>{const b=await launch();try{
 for(const time of ['12:00','19:00']) for(const side of [-1,1]) for(const nearby of [false,true]){
 if(process.env.EB_MIRA_CASE&&process.env.EB_MIRA_CASE!==`${time}:${side}:${nearby}`)continue;
 const p=await b.newPage({viewport:{width:1280,height:800}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.clock.setFixedTime(new Date((1767225600+4*480+(time==='12:00'?240:380))*1000));
 await p.goto('http://127.0.0.1:8091/eternalbliss/',{waitUntil:'domcontentloaded'});await p.click('#eb-menu-solo');
 const before=await p.evaluate(({time,side,nearby})=>{
   if(worldClimate.time!==time)throw Error('fixture clock mismatch'); const m=npcs.find(n=>n.name==='Mira, the Cartographer');
   if(!m)throw Error('Mira missing');
   gameState.player.x=m.x+side;gameState.player.y=m.y;gameState.player.xp=0;gameState.player.level=1;gameState.player.xpToNext=1000;
   gameState.inventory.gold=10;enemies=[];
   if(nearby)npcs.unshift({name:'Fixture resident',class:'npc npc-villager',dialogue:'Generic greeting',x:m.x+side,y:m.y+1,movable:false});
   ebFarlands.markStory('quest-vault-relic');renderWorld();return {gold:gameState.inventory.gold,xp:gameState.player.xp,legal:canEntityMoveTo(gameState.player.x,gameState.player.y)};
 },{time,side,nearby});
 assert(before.legal,'approach walkable');await p.waitForFunction(()=>!movementBlocked());await p.evaluate(()=>document.activeElement?.blur());await p.keyboard.press('Space');
 const result=await p.evaluate(()=>({dialog:document.getElementById('modalContent').textContent,title:document.getElementById('modalTitle').textContent,gold:gameState.inventory.gold,xp:gameState.player.xp,reward:ebFarlands.storyFlag('quest-vault-reward')}));
 console.log(JSON.stringify({time,side,nearby,before,result}));
 assert.match(result.dialog,/Harvest Bell restored/);assert.equal(result.gold-before.gold,65);assert.equal(result.xp-before.xp,85);
 await p.screenshot({path:`${out}/mira-${time.replace(':','')}-${side}-${nearby}.png`});
 await p.evaluate(()=>{closeModal();soloSaves.flush();});await p.reload({waitUntil:'domcontentloaded'});await p.click('#eb-menu-solo');
 await p.evaluate(()=>document.activeElement?.blur());await p.keyboard.press('Space');const restored=await p.evaluate(()=>({gold:gameState.inventory.gold,xp:gameState.player.xp,reward:ebFarlands.storyFlag('quest-vault-reward')}));
 assert.deepEqual(restored,{gold:result.gold,xp:result.xp,reward:true});assert.deepEqual(errors,[]);await p.close();
 }
 console.log('PASS Mira:',process.env.EB_MIRA_CASE||'8 final-state fixtures','actual Space, once-only reward and reload');
}finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1});
