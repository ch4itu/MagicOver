const t=require('./browser.cjs');
(async()=>{const b=await t.launch();try{
 const cases=[['desktop',1280,800,false,false],['short-desktop',1280,450,false,false],['touch-desktop',1280,800,true,false],['phone',390,844,true,true],['landscape',844,390,true,true]];
 for(const [name,width,height,touch,visible]of cases){const p=await t.page(b,{viewport:{width,height},hasTouch:touch,isMobile:touch});
 for(const clean of [false,true]){
  if(clean)await p.click('#eb-capture-toggle');
  const shown=await p.locator('#mobile-controls').isVisible();t.assert.equal(shown,visible,name+' clean='+clean);
  if(visible){const hit=await p.evaluate(()=>{const n=document.getElementById('eb-joystick'),r=n.getBoundingClientRect();return n.contains(document.elementFromPoint(r.x+r.width/2,r.y+r.height/2));});t.assert(hit,'joystick receives input');}
 }
 await p.click('#eb-capture-toggle');
 await p.click('#eb-immersive-toggle');await p.keyboard.press('KeyQ');
 const before=await p.evaluate(()=>[gameState.player.x,gameState.player.y]);await p.keyboard.press('ArrowDown');
 await p.waitForTimeout(200);const after=await p.evaluate(()=>[gameState.player.x,gameState.player.y]);t.assert.notDeepEqual(after,before,'FP keyboard movement');
 if(visible){const j=await p.locator('#eb-joystick').boundingBox();const x=j.x+j.width/2,y=j.y+j.height/2;const before=await p.evaluate(()=>[gameState.player.x,gameState.player.y]);await p.mouse.move(x,y);await p.mouse.down();await p.mouse.move(x,y+j.height*.35);await p.waitForTimeout(250);await p.mouse.up();const after=await p.evaluate(()=>[gameState.player.x,gameState.player.y]);t.assert.notDeepEqual(after,before,'joystick pointer movement');}
 t.assert.deepEqual(p.errors,[]);await t.shot(p,'controls-'+name);t.record('controls '+name,'PASS visibility normal/Clean View, FP movement; pointer joystick where visible');console.log('PASS',name);await p.close();
 }
}finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1});
