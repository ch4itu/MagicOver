const t = require('./browser.cjs');
const profiles = {
    desktop:{viewport:{width:1280,height:800}},
    portrait:{viewport:{width:390,height:844},hasTouch:true,isMobile:true},
    landscape:{viewport:{width:844,height:390},hasTouch:true,isMobile:true}
};
(async () => {
    const browser=await t.launch();
    try {
        for (const [name, profile] of Object.entries(profiles)) {
            if (process.env.EB_COAST_PROFILE && process.env.EB_COAST_PROFILE!==name) continue;
            const page=await t.page(browser,profile,(1767225600+4*480+240)*1000);
            const use = selector => profile.hasTouch ? page.tap(selector) : page.click(selector);
            for (const id of ['amber','lantern','settlement-lantern-0']) {
                const dock=await page.evaluate(id=>{
                    const dock=Farlands.coastalDocks[id], site=Farlands.byId.get(id);
                    applyAdventureMap(captureAdventureMap(),{...gameState.player,x:dock.pier[0],y:dock.pier[1]});
                    const directions=[[1,0],[-1,0],[0,1],[0,-1]];
                    const pending=[dock.shore], visited=new Set([dock.shore.join(',')]);
                    let townReached=false;
                    for(const [x,y] of pending){
                        if(Math.hypot(x-site.x,y-site.y)<=3){townReached=true;break;}
                        for(const [dx,dy] of directions){
                            const nx=x+dx,ny=y+dy,key=nx+','+ny;
                            if(visited.has(key)||Math.hypot(nx-site.x,ny-site.y)>30||!canEntityMoveTo(nx,ny))continue;
                            visited.add(key);pending.push([nx,ny]);
                        }
                    }
                    return {...dock, townReached, pierOpen:canEntityMoveTo(...dock.pier)};
                },id);
                t.assert(dock.townReached && dock.pierOpen,id+' real approach is clear');
            }

            const dock=await page.evaluate(()=>{
                const dock=Farlands.coastalDocks.lantern;
                const [x,y]=dock.pier;
                applyAdventureMap(captureAdventureMap(),{...gameState.player,x,y,targetX:x,targetY:y});
                return dock;
            });
            await page.waitForFunction(()=>__view.harborVessels.find(v=>v.id==='lantern').model.visible);
            await use('#eb-immersive-toggle');
            const aim=await page.evaluate(()=>{
                const v=__view, boat=v.harborVessels.find(v=>v.id==='lantern').model;
                boat.updateWorldMatrix(true,true);
                const hull = boat.children[0];
                const centre = new THREE.Box3().setFromObject(hull).getCenter(new THREE.Vector3());
                const from = new THREE.Vector3(gameState.player.x,1.135,gameState.player.y);
                const d=centre.clone().sub(from);
                v.eyeYaw=v.eyeRenderYaw=Math.atan2(-d.x,-d.z);
                v.eyePitch=v.eyeRenderPitch=Math.atan2(d.y,Math.hypot(d.x,d.z));
                const bounds=new THREE.Box3().setFromObject(hull).getSize(new THREE.Vector3());
                return {centre:centre.toArray(),bounds:bounds.toArray(),distance:d.length()};
            });
            const picked=await page.waitForFunction(()=>{
                const view=__view, boat=view.harborVessels.find(v=>v.id==='lantern').model;
                const ray=new THREE.Raycaster();
                ray.setFromCamera(new THREE.Vector2(0,0),view.eyeCamera);
                const hit=ray.intersectObject(boat,true)[0];
                if(!hit || hit.distance>5.5)return false;
                return {camera:view.eyeCamera.position.toArray(),distance:hit.distance,
                    actorHit:ray.intersectObjects(view.targets,false)[0]?.distance ?? null};
            },null,{timeout:8000});
            console.log(name, 'mooring ray', JSON.stringify({aim, hit:await picked.jsonValue()}));
            await t.shot(page,'lantern-first-person-'+name+'-'+t.sha.slice(0,8));
            await use('#eb-eye-use');
            await page.locator('#eb-vehicle-title').filter({hasText:'Embark'}).waitFor({timeout:8000});
            await use('#eb-vehicle-accept');
            await page.waitForFunction(()=>gameState.sailingMoves===15);
            t.assert.deepEqual(await page.evaluate(()=>[gameState.player.x,gameState.player.y]),dock.water);
            t.assert.deepEqual(page.errors,[]);
            const frames=await page.evaluate(()=>new Promise(resolve=>{
                const renderer=__view.renderer;
                const render=renderer.render;
                const intervals=[], submissions=[];
                let last;
                const summarize=values=>{
                    const sorted=[...values].sort((a,b)=>a-b);
                    return {medianMs:sorted[Math.floor(sorted.length/2)],p95Ms:sorted[Math.ceil(.95*sorted.length)-1],maxMs:sorted.at(-1)};
                };
                renderer.render=function(...args){
                    const start=performance.now();
                    if(last!==undefined) intervals.push(start-last);
                    last=start;
                    try { return render.apply(this,args); }
                    finally {
                        submissions.push(performance.now()-start);
                        if(intervals.length===30){
                            renderer.render=render;
                            resolve({samples:intervals.length,frameIntervals:summarize(intervals),cpuSubmission:summarize(submissions)});
                        }
                    }
                };
            }));
            await use('#eb-immersive-toggle');
            await page.evaluate(()=>document.activeElement?.blur());
            await page.keyboard.press('ArrowUp');
            await page.waitForFunction(([x,y])=>gameState.player.x===x && gameState.player.y===y-1,dock.water);
            const panel=page.locator('#eb-actions-collapse');
            if(await panel.getAttribute('aria-expanded')==='false') await use('#eb-actions-collapse');
            await use('#eb-farlands-dock');
            await use('#eb-vehicle-accept');
            await page.waitForFunction(()=>gameState.sailingMoves===0);
            t.assert.deepEqual(await page.evaluate(()=>[gameState.player.x,gameState.player.y]),dock.pier);
            t.assert.deepEqual(page.errors,[]);
            t.record('Coastal approaches and FP boarding '+name,{result:'PASS real Use, water step and disembark at Lantern Quay',aim,frames});
            console.log('PASS',name,JSON.stringify(frames));
            await page.close();
        }
    } finally { await browser.close(); }
})().catch(error=>{console.error(error);process.exitCode=1;});
