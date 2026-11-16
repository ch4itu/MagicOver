const t=require('./browser.cjs'),{fixture}=require('./combat.cjs');
(async()=>{const b=await t.launch();try{const p=await t.page(b);await fixture(p,true,true);
 await p.evaluate(()=>{const v=__view,render=v.renderer.render.bind(v.renderer);v.renderer.render=(scene,camera)=>{window.__horseAtRender=v.farlandsHorse.visible;return render(scene,camera);};});await p.waitForTimeout(250);
 t.assert.equal(await p.evaluate(()=>__horseAtRender),false,'mount steps aside only for the FP fight');
 await p.evaluate(()=>endBattle(false));await p.waitForTimeout(350);t.assert.equal(await p.evaluate(()=>__horseAtRender),true,'normal mounted FP presentation preserved');
 t.assert(await p.evaluate(()=>ebHorseMounted&&ebFarlands.horseState().owned));t.assert.deepEqual(p.errors,[]);t.record('mounted FP presentation scope','PASS horse retained outside combat, combat view unobstructed, ownership/mounted state unchanged');console.log('PASS mounted presentation scope');
}finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1});
