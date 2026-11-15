const t=require('./browser.cjs'),{fixture}=require('./combat.cjs');
(async()=>{const b=await t.launch();try{const p=await t.page(b);
 for(const kind of ['wolf','dragon','crawler']){
 await fixture(p,false,true);await p.evaluate(kind=>{endBattle(false);startBattle({x:gameState.player.x,y:gameState.player.y-1,name:'Fixture '+kind,class:'enemy-spawn enemy-'+kind,crawler:kind==='crawler',hp:50,maxHp:50,attack:5,xpReward:1,goldReward:1});},kind);
await p.waitForTimeout(400);
 const r=await p.evaluate(()=>{const v=__view,c=v.combat,pt=c.model.group.position.clone();pt.y+=c.model.height*.5;pt.project(v.eyeCamera);return {eye:v.eyeActive(),point:pt.toArray(),distance:c.model.group.position.distanceTo(v.eyeCamera.position)};});t.assert(r.eye&&r.point[2]<1&&Math.abs(r.point[0])<1&&r.distance>1);await t.shot(p,'edge-'+kind);t.record('FP enemy model '+kind,{result:'PASS eye frustum and guard separation',...r});
 }
 // Put a blocked tile immediately ahead: staging chooses a clear alternate line and eases the eye toward it.
 await p.evaluate(()=>{endBattle(false);const inn=buildings.find(b=>b.realmId==='rest-service-0');gameState.player.x=inn.x;gameState.player.y=inn.y+2;__view.eyeYaw=0;renderWorld();});await p.waitForTimeout(400);
 await p.evaluate(()=>startBattle({x:gameState.player.x+1,y:gameState.player.y,name:'Doorway fixture',class:'enemy-spawn enemy-goblin',hp:50,maxHp:50,attack:5,xpReward:1,goldReward:1}));await p.waitForTimeout(600);
 const clear=await p.evaluate(()=>{const c=__view.combat;return [.6,1.2,2.1].every(d=>canEntityMoveTo(Math.round(c.base.x+c.dir.x*d),Math.round(c.base.z+c.dir.z*d)));});t.assert(clear,'battle corridor avoids structure');await t.shot(p,'edge-doorway');t.assert.deepEqual(p.errors,[]);t.record('FP staging near inn doorway','PASS clear corridor; no JS errors');
 // Scope check only, not live multiplayer evidence: the existing HUD rejects shared-world presentation.
 const shared=await p.evaluate(()=>{endBattle(false);const old=sharedWorld.active;sharedWorld.active=()=>true;try{return ebCombatHUD.open({name:'scope fixture',hp:1,maxHp:1});}finally{sharedWorld.active=old;ebCombatHUD.close();}});t.assert.equal(shared,false);t.record('shared-world HUD scope guard','PASS stubbed scope check, no live connection');
 console.log('PASS model/structure/scope edges');
}finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1});
