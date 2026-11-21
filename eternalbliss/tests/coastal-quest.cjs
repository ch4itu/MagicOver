const t = require('./browser.cjs');
(async () => {
    const browser=await t.launch();
    try {
        const page=await t.page(browser,undefined,(1767225600+4*480+240)*1000);
        await page.click('#eb-view-toggle');
        await page.evaluate(()=>{
            const site=Farlands.byId.get('settlement-lantern-0');
            applyAdventureMap(captureAdventureMap(),{...gameState.player,x:site.x,y:site.y+1});
        });
        await page.click('#eb-farlands-quests');
        const card=page.locator('#modalContent section').filter({hasText:'Sound the Lantern Channel'});
        await card.getByRole('button',{name:'Accept quest'}).click();
        await page.locator('#interactionModal').getByRole('button',{name:'Close',exact:true}).click();
        const passage=await page.evaluate(()=>{
            const shellwick=Farlands.coastalDocks['settlement-lantern-0'].water;
            const quay=Farlands.coastalDocks.lantern.water;
            const directions=[[1,0],[-1,0],[0,1],[0,-1]];
            const states=[{point:shellwick,parent:null,step:null}];
            const visited=new Set([shellwick.join(',')]);
            let destination;
            for(const state of states){
                const [x,y]=state.point;
                if(Math.hypot(x-quay[0],y-quay[1])<=3){destination=state;break;}
                for(const [dx,dy] of directions){
                    const nx=x+dx,ny=y+dy,key=nx+','+ny;
                    if(nx<8460||nx>8600||ny<8530||ny>8630||visited.has(key)||Farlands.sample(nx,ny)!=='water')continue;
                    visited.add(key);
                    states.push({point:[nx,ny],parent:state,step:[dx,dy]});
                }
            }
            if(!destination)throw Error('No local water route to the quay');
            const steps=[];
            for(let state=destination;state.parent;state=state.parent)steps.push(state.step);
            steps.reverse();
            // Fixture starts aboard; each step goes through the real move and terrain checks.
            gameState.sailingMoves=15;
            applyAdventureMap(captureAdventureMap(),{...gameState.player,x:shellwick[0],y:shellwick[1]});
            for(const step of steps)if(!movePlayer(...step))throw Error('Sailing step was rejected');
            return {steps:steps.length,pin:ebFarlands.pinned(),afloat:gameState.sailingMoves};
        });
        t.assert(passage.steps>30);
        t.assert.equal(passage.pin,'settlement-lantern-0');
        t.assert.equal(passage.afloat,15);
        await page.evaluate(()=>{
            soloSaves.queue();soloSaves.flush();
        });
        await page.reload({waitUntil:'domcontentloaded'});
        await page.click('#eb-menu-solo');
        await page.waitForFunction(()=>ebFarlands.surface()&&gameState.sailingMoves===15);
        const before=await page.evaluate(()=>{
            if(!ebFarlands.storyFlag('rq-step2-d-settlement-lantern-0'))throw Error('Sailing progress was not restored');
            const site=Farlands.byId.get('settlement-lantern-0');
            // Fixture handles the return leg. Report through the real source NPC and dialog.
            gameState.sailingMoves=0;
            applyAdventureMap(captureAdventureMap(),{...gameState.player,x:site.x,y:site.y+1});
            const keeper=npcs.find(npc=>npc.name==='Shellwick Waykeeper');
            if(!keeper)throw Error('The Shellwick waykeeper is missing at noon');
            const near=[[0,1],[0,-1],[1,0],[-1,0]].map(([dx,dy])=>[keeper.x+dx,keeper.y+dy])
                .find(([x,y])=>canEntityMoveTo(x,y));
            if(!near)throw Error('No legal approach to the waykeeper');
            Object.assign(gameState.player,{x:near[0],y:near[1]});
            useWorldActor('npc',keeper);
            return {gold:gameState.inventory.gold,xp:gameState.player.xp};
        });
        await page.getByRole('button',{name:'Report back · collect 24 gold'}).click();
        t.assert.deepEqual(await page.evaluate(()=>({
            gold:gameState.inventory.gold,xp:gameState.player.xp,
            done:ebFarlands.storyFlag('rq-done-d-settlement-lantern-0')
        })),{gold:before.gold+24,xp:before.xp+25,done:true});
        t.assert.equal(await page.getByRole('button',{name:'Report back · collect 24 gold'}).count(),0);
        t.assert.deepEqual(page.errors,[]);
        t.record('Shellwick channel quest',{result:'PASS acceptance, real sailing steps, save/reload, actual NPC report and single reward',passage});
        console.log('PASS Shellwick channel quest',passage);
    } finally { await browser.close(); }
})().catch(error=>{console.error(error);process.exitCode=1;});
