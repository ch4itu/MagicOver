const t=require('./browser.cjs'),{fixture}=require('./combat.cjs');
(async()=>{const b=await t.launch();try{
 const profiles=process.env.EB_WEAPON_BASELINE?[['phone',390,844]]:[['phone',390,844],['landscape',844,390],['desktop',1280,800]];
 for(const [name,width,height]of profiles){if(process.env.EB_WEAPON_CASE&&process.env.EB_WEAPON_CASE!==name)continue;const p=await t.page(b,{viewport:{width,height},hasTouch:name!=='desktop',isMobile:name!=='desktop'});await fixture(p,false,true);
 await p.evaluate(()=>{__view.setActive(false);__view.active=true;window.__oldVisual=ebCombatHUD.visual;});
 for(const [action,age]of [['guard',0],['attack',.34],['magic',.43],['heal',.4],['enemy',.49]]){
 const q=await p.evaluate(({action,age})=>{ebCombatHUD.visual=()=>action==='guard'?null:{action,age};const v=__view;v.draw(performance.now());cancelAnimationFrame(v.frame);v.renderer.getContext().finish();v.eyeCamera.updateMatrixWorld(true);const sword=v.eyeSwordArm.children.find(n=>n.isGroup),blade=sword.children[2];const at=new THREE.Vector3();blade.getWorldPosition(at);at.project(v.eyeCamera);return {point:at.toArray(),scale:v.eyeHands.scale.x};},{action,age});
 console.log(name,action,JSON.stringify(q));t.assert(Math.abs(q.point[0])<.95&&Math.abs(q.point[1])<.95&&q.point[2]<1,'weapon blade must be in the eye frustum, not just marked visible');
 if(action==='guard'||action==='attack')await t.shot(p,'weapon-'+name+'-'+action+'-'+t.sha.slice(0,8));
 }
 await p.evaluate(()=>{ebCombatHUD.visual=window.__oldVisual;__view.setActive(true);endBattle(false);});
 await p.keyboard.press('KeyQ');await p.keyboard.press('ArrowDown');t.assert.deepEqual(p.errors,[]);t.record('weapon frustum '+name,'PASS blade projected inside view for guard/attack/magic/heal/enemy; FP keyboard input and JS smoke');await p.close();
 }
}finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1});
