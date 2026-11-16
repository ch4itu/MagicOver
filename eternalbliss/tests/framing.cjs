const t=require('./browser.cjs'),{fixture,state,profiles}=require('./combat.cjs');
(async()=>{const b=await t.launch();try{
 for(const [name,profile]of profiles)for(const mounted of [false,true]){
 const p=await t.page(b,profile);await fixture(p,mounted,true);let s=await state(p);
 const chest=s.canvas.top+(1-s.enemy[1])*s.canvas.height/2;t.assert(chest<s.hud.top-12,`${name} mounted=${mounted}: chest ${chest} behind HUD ${s.hud.top}`);
 await p.evaluate(()=>{__view.setActive(false);__view.active=true;window.__visual=ebCombatHUD.visual;});
 // Sample the production animation at fixed ages in a real renderer. These are pose captures, not timing measurements.
 for(const [action,age]of [['guard',0],['attack',.34],['magic',.43],['heal',.4],['enemy',.46]]){
 await p.evaluate(({action,age})=>{ebCombatHUD.visual=()=>action==='guard'?null:{action,age};__view.draw(performance.now());cancelAnimationFrame(__view.frame);},{action,age});
 await t.shot(p,`pose-${name}-${mounted}-${action}`);
 t.assert(await p.evaluate(()=>__view.eyeActive()&&Number.isFinite(__view.eyeCamera.position.y)));
 }
 await p.evaluate(()=>{ebCombatHUD.visual=window.__visual;__view.setActive(true);});
 const before=await p.evaluate(()=>[__view.eyeYaw,__view.eyePitch??-.055]);await p.keyboard.press('KeyQ');const during=await p.evaluate(()=>[__view.eyeYaw,__view.eyePitch??-.055]);t.assert.deepEqual(before,during,'combat keeps aim');
 await p.evaluate(()=>endBattle(false));await p.waitForTimeout(600);t.assert.deepEqual(await p.evaluate(()=>[__view.eyeYaw,__view.eyePitch??-.055]),before);
 t.assert.deepEqual(p.errors,[]);t.record(`framing ${name} mounted=${mounted}`,{result:'PASS',chest,hudTop:s.hud.top,poses:'guard/attack/magic/heal/enemy fixed-age renders; original look preference restored'});console.log('PASS framing',name,mounted);await p.close();
 }
}finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1});
