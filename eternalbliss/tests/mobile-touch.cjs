const t=require('./browser.cjs');
const profiles=[['portrait',390,844],['landscape',844,390]];
(async()=>{
    const browser=await t.launch();
    try {
        for(const [name,width,height] of profiles){
            if(process.env.EB_TOUCH_PROFILE && process.env.EB_TOUCH_PROFILE!==name)continue;
            const page=await t.page(browser,{viewport:{width,height},hasTouch:true,isMobile:true},(1767225600+4*480+240)*1000);
            const cdp=await page.context().newCDPSession(page);
            const touch=(type,points)=>cdp.send('Input.dispatchTouchEvent',{type,touchPoints:points});
            await page.evaluate(()=>{
                window.groundEvents=[];
                document.addEventListener('click',event=>{
                    if(event.pointerType==='touch')window.groundEvents.push(event.target.id||event.target.className);
                },true);
            });
            await page.waitForFunction(()=>__view.cameraPosition.distanceTo(new THREE.Vector2(gameState.player.x,gameState.player.y))<.5);
            const moved=[];
            for(const mode of ['3d','2d']){
                if(mode==='2d'){
                    await page.tap('#eb-view-toggle');
                    await page.waitForFunction(()=>classicView());
                    await page.waitForTimeout(350);
                }
                const point=await page.evaluate(mode=>{
                    const p=gameState.player;
                    for(const radius of [2,3,4,5,6]){
                        for(const [dx,dy] of [[0,1],[-1,0],[1,0],[0,-1],[1,1],[-1,1]]){
                            const x=p.x+dx*radius,y=p.y+dy*radius;
                            if(!findWalkRoute(x,y).length)continue;
                            let left,top,target;
                            if(mode==='2d'){
                                target=[...document.querySelectorAll('#worldGrid .tile')].find(
                                    el=>parseFloat(el.style.left)===x*32&&parseFloat(el.style.top)===y*32);
                                if(!target)continue;
                                const r=target.getBoundingClientRect();
                                left=r.x+r.width/2;top=r.y+r.height/2;
                            }else{
                                const v=__view,r=v.canvas.getBoundingClientRect();
                                const q=new THREE.Vector3(x,.055,y).project(v.camera);
                                left=r.x+(q.x+1)*r.width/2;top=r.y+(1-q.y)*r.height/2;target=v.canvas;
                                if(v.aim({clientX:left,clientY:top}))continue;
                            }
                            if(document.elementFromPoint(left,top)===target)return {left,top,x,y};
                        }
                    }
                    return null;
                },mode);
                t.assert(point,'visible empty '+mode+' tile');
                const before=await page.evaluate(()=>gameState.player.moveCount);
                await touch('touchStart',[{x:point.left,y:point.top}]);
                await touch('touchMove',[{x:point.left+23,y:point.top}]);
                await touch('touchEnd',[]);
                await page.waitForTimeout(200);
                t.assert.equal(await page.evaluate(()=>gameState.player.moveCount),before,'swipe does not walk');
                await page.touchscreen.tap(point.left, point.top);
                await page.waitForFunction(({x,y})=>gameState.player.x===x&&gameState.player.y===y,point);
                moved.push({mode,point,steps:await page.evaluate(n=>gameState.player.moveCount-n,before)});
            }

            const button=await page.locator('#eb-joystick').boundingBox();
            const x=button.x+button.width/2,y=button.y+button.height/2;
            const before=await page.evaluate(()=>gameState.player.moveCount);
            for(const [dx,dy] of [[0,1],[1,0],[-1,0],[0,-1]]){
                if(!await page.evaluate(([dx,dy])=>canEntityMoveTo(gameState.player.x+dx,gameState.player.y+dy),[dx,dy]))continue;
                await touch('touchStart',[{x,y}]);
                await touch('touchMove',[{x:x+dx*button.width*.34,y:y+dy*button.height*.34}]);
                try {await page.waitForFunction(before=>gameState.player.moveCount>before,before,{timeout:5000});}
                finally {await touch('touchEnd',[]);}
                break;
            }
            t.assert(await page.evaluate(before=>gameState.player.moveCount>before,before),'native touch D-pad accepted movement');

            const panel=page.locator('#eb-actions-collapse');
            if(await panel.getAttribute('aria-expanded')==='false')await panel.tap();
            const atlasButton=page.locator('#eb-farlands-atlas-open');
            if(await atlasButton.isHidden())await page.tap('#eb-actions-more');
            await atlasButton.tap();
            await page.locator('#eb-farlands-atlas[open]').waitFor();
            await page.tap('#eb-atlas-zoom-in');
            await t.shot(page,'touch-atlas-'+name+'-'+t.sha.slice(0,8));
            await page.tap('#eb-farlands-atlas-close');
            const clicks=await page.evaluate(()=>window.groundEvents);
            t.assert(clicks.includes('eb-scene')&&clicks.some(id=>String(id).startsWith('tile')));
            t.assert(clicks.includes('eb-farlands-atlas-open'));
            t.assert.deepEqual(page.errors,[]);
            t.record('Native mobile touch '+name,{result:'PASS 3D/2D ground taps, swipe rejected, D-pad and Atlas',moved,clicks});
            console.log('PASS native touch',name);
            await page.close();
        }
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
