const t=require('./browser.cjs');
(async()=>{
    const browser=await t.launch();
    try {
        const page=await t.page(browser,undefined,(1767225600+4*480+240)*1000);
        await page.click('#eb-view-toggle');
        const started=Date.now();
        t.assert.equal(await page.evaluate(()=>gameState.player.moveCount),0);
        await page.click('#eb-farlands-quests');
        const quest=page.locator('#modalContent section').filter({hasText:'The millwright’s tally'});
        await quest.getByRole('button',{name:'Accept quest'}).click();
        const close=()=>page.locator('#interactionModal').getByRole('button',{name:'Close',exact:true}).click();
        await close();
        async function walkTo(target){
            for(let i=0;i<110;i++){
                const step=await page.evaluate(([x,y])=>{
                    const p=gameState.player;
                    if(p.x===x&&p.y===y)return {done:true};
                    const route=findWalkRoute(x,y), next=route[0];
                    if(!next)throw Error('No walkable route to '+x+','+y);
                    const keys={'1,0':'ArrowRight','-1,0':'ArrowLeft','0,1':'ArrowDown','0,-1':'ArrowUp'};
                    return {from:[p.x,p.y],key:keys[[next[0]-p.x,next[1]-p.y].join(',')]};
                },target);
                if(step.done)return;
                t.assert(step.key);
                await page.evaluate(()=>document.activeElement?.blur());
                await page.keyboard.press(step.key);
                await page.waitForFunction(([x,y])=>gameState.player.x!==x||gameState.player.y!==y,step.from,{timeout:4000});
                await page.waitForTimeout(135);
            }
            throw Error('Walking step limit exceeded');
        }
        const approach=await page.evaluate(()=>{
            const crate=[...Farlands.authoredProps.values()].find(p=>p.name==='Ivo’s axle crate');
            const candidates=[];
            for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){
                const x=crate.x+dx,y=crate.y+dy;
                if(!canEntityMoveTo(x,y)||Farlands.propNear(x,y)!==crate||
                    buildings.some(b=>buildingDistance(b,x,y)<=2)||
                    npcs.some(n=>Math.hypot(x-n.x,y-n.y)<=1.5))continue;
                const route=findWalkRoute(x,y);
                if(route.length)candidates.push({point:[x,y],length:route.length});
            }
            return candidates.sort((a,b)=>a.length-b.length)[0]?.point;
        });
        t.assert(approach,'a legal approach reaches the named crate');
        await walkTo(approach);
        await page.keyboard.press('Space');
        await page.getByRole('status').filter({hasText:'Tally recorded'}).waitFor();
        await t.shot(page,'fresh-axle-crate-'+t.sha.slice(0,8));
        await close();
        const mill=await page.evaluate(()=>{const s=Farlands.byId.get('mill');return [s.x-3,s.y+2];});
        await walkTo(mill);
        t.assert(await page.evaluate(()=>ebFarlands.storyFlag('rq-step2-e-rest')));
        const nearest=await page.evaluate(()=>{
            const n=npcs.find(n=>n.name==='Ivo, the Millwright');
            const routeOptions=[[0,1],[0,-1],[1,0],[-1,0]].map(([dx,dy])=>[Math.round(n.x)+dx,Math.round(n.y)+dy]);
            return routeOptions.find(([x,y])=>canEntityMoveTo(x,y)&&findWalkRoute(x,y).length);
        });
        t.assert(nearest);
        await walkTo(nearest);
        const before=await page.evaluate(()=>gameState.inventory.gold);
        await page.click('#eb-farlands-people');
        const row=page.locator('#modalContent > div').filter({has:page.getByText('Ivo, the Millwright',{exact:true})});
        await row.getByRole('button',{name:'Talk',exact:true}).click();
        const reward=page.getByRole('button',{name:'Report back · collect 15 gold'});
        if(await reward.isHidden()){
            const tab=page.getByRole('tab',{name:/Quests/});
            if(await tab.isVisible())await tab.click();
        }
        console.log('REPORT',await page.locator('#modalTitle').textContent());
        await reward.click();
        t.assert.equal(await page.evaluate(()=>gameState.inventory.gold),before+15);
        t.assert.equal(await reward.count(),0);
        await close();
        await page.reload({waitUntil:'domcontentloaded'});
        await page.click('#eb-menu-solo');
        await page.waitForFunction(()=>ebFarlands.surface());
        t.assert(await page.evaluate(()=>ebFarlands.storyFlag('rq-done-e-rest')));
        t.assert.equal(await page.evaluate(()=>gameState.inventory.gold),before+15);
        t.assert.deepEqual(page.errors,[]);
        t.record('Fresh guest millwright tally',{result:'PASS real key walking, crate inspection, Millbend discovery, Ivo dialogue, reward once and reload',elapsedMs:Date.now()-started});
        console.log('PASS fresh guest millwright route');
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
