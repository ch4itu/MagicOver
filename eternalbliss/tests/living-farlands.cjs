const t = require('./browser.cjs');
const {writeFileSync} = require('node:fs');
const path = require('node:path');
const base=process.env.EB_LIFE_BASELINE==='1';
const harborOnly=process.env.EB_LIFE_HARBOR==='1';
const noon=(1767225600+4*480+240)*1000;
const phone={viewport:{width:390,height:844},isMobile:true,hasTouch:true};
const siteAt=async(page,id)=>page.evaluate(id=>{
    const s=Farlands.byId.get(id);
    ebFarlands.leave();
    ebFarlands.restore({version:1,active:true,place:{x:s.x,y:s.y+1,sailing:0}});
},id);
const modalClose=page=>page.keyboard.press('Escape');

async function accept(page, title) {
    const tab=page.getByRole('tab',{name:/Quests/});
    if(await tab.isVisible())await tab.click();
    await page.locator('#modalContent > section').filter({hasText:title}).getByRole('button',{name:'Accept quest'}).click();
    await modalClose(page);
}

async function talk(page,id){
    await page.evaluate(id=>{
        const name=Farlands.byId.get(id).name+' Waykeeper';
        const npc=npcs.find(n=>n.name===name);
        if(!npc)throw Error(name+' missing');
        Object.assign(gameState.player,{x:npc.x,y:npc.y});
        useWorldActor('npc',npc);
    },id);
    await page.locator('#interactionModal').waitFor({state:'visible'});
}

async function relocate(page,id){
    await page.evaluate(id=>{
        const s=Farlands.byId.get(id);
        Object.assign(gameState.player,{x:s.x,y:s.y+1,targetX:s.x,targetY:s.y+1});
        ebFarlands.refreshScene(true);
        renderWorld();
    },id);
}

async function verify(browser){
    const page=await t.page(browser,undefined,noon);
    const coverage=await page.evaluate(()=>{
        const towns=Farlands.sites.filter(s=>s.station);
        const cells=new Map();
        let ports=0,buyers=0,roadWalkers=0,maxLocal=0,farmMarkets=0;
        for(const site of towns){
            const cx=Math.floor(site.x/Farlands.cellSize),cy=Math.floor(site.y/Farlands.cellSize);
            const key=cx+','+cy;
            if(!cells.has(key))cells.set(key,Farlands.sceneCell(cx,cy));
            const npcs=cells.get(key).npcs.filter(n=>n.ebSite===site.id);
            maxLocal=Math.max(maxLocal,npcs.length);
            if(npcs.some(n=>n.ebRole==='Fisher') && npcs.some(n=>n.ebRole==='Dockworker'))ports++;
            if(npcs.some(n=>n.ebRole==='Market Buyer'))buyers++;
            if(npcs.some(n=>n.ebRole==='Farmer'&&n.routine.some(stop=>stop.where==='market')))farmMarkets++;
        }
        for(const route of Farlands.roads)for(let i=1;i<route.length;i++){
            const x=(route[i][0]+route[i-1][0])/2,y=(route[i][1]+route[i-1][1])/2;
            const cx=Math.floor(x/Farlands.cellSize),cy=Math.floor(y/Farlands.cellSize),key=cx+','+cy;
            if(!cells.has(key))cells.set(key,Farlands.sceneCell(cx,cy));
        }
        for(const cell of cells.values()){
            const travelers=cell.npcs.filter(n=>n.realmId?.startsWith('road-r16-'));
            if(travelers.length>1)throw Error('more than one traveler in a cell');
            roadWalkers+=travelers.filter(n=>n.routine?.some(s=>s.where==='roadB')).length;
        }
        return {stations:towns.length,ports,buyers,farmMarkets,roadWalkers,maxLocal};
    });
    t.assert.equal(coverage.stations,54);
    t.assert(coverage.ports>=3 && coverage.buyers>0 && coverage.farmMarkets>0 && coverage.roadWalkers>0);
    await siteAt(page,'lantern');
    const harbor=await page.evaluate(()=>{
        const roles=ebLife.roster().filter(n=>n.ebSite==='lantern'&&['Fisher','Dockworker'].includes(n.ebRole));
        const start=performance.now(),original=worldClimate.time;
        const records=roles.map(n=>({name:n.name,start:[n.x,n.y],walked:0}));
        worldClimate.time='16:00';
        let blocked=0;
        const initial=roles.map(n=>[n.x,n.y]);
        for(let i=0;i<250;i++){
            ebLife.tick(start+i*170);
            roles.forEach((n,j)=>{
                records[j].walked+=Math.hypot(n.x-initial[j][0],n.y-initial[j][1]);
                initial[j]=[n.x,n.y];
                if(!n.indoors&&!canEntityMoveTo(n.x,n.y))blocked++;
            });
        }
        const at=roles.map(n=>[n.life.slot.where,n.x,n.y,ebLife.activity(n,start+50000)?.kind]);
        worldClimate.time=original;
        return {at,records,blocked};
    });
    console.log('coverage',coverage,'harbor',harbor);
    t.assert(harbor.records.every(r=>r.walked>2) && harbor.blocked===0);
    t.assert(harbor.at.every(a=>a[0]==='landing'));
    t.assert.deepEqual(harbor.at.map(a=>a[3]),['sortCatch','carry']);
    await page.close();

    const inn=await t.page(browser,undefined,new Date(noon+11*20*1000));
    await siteAt(inn,'lantern');
    const guests=await inn.evaluate(()=>{
        const b=buildings.find(b=>b.realmId==='lantern-service-0');
        openBuildingInterior(b);
        return npcs.filter(n=>n.realmId).map(n=>[n.name,canEntityMoveTo(n.x,n.y)]);
    });
    t.assert(guests.length>0&&guests.length<=2&&guests.every(guest=>guest[1]));
    await t.shot(inn,'phase2-lantern-inn-'+t.sha.slice(0,8));
    await inn.click('#eb-room-exit');
    t.assert(await inn.evaluate(()=>!buildingInterior));
    await inn.close();

    const quest=await t.page(browser,undefined,noon);
    await siteAt(quest,'settlement-lantern-0');
    const shell='settlement-lantern-0';
    const funds=await quest.evaluate(()=>gameState.inventory.gold);
    t.assert.equal(await quest.evaluate(()=>ebRpgQuests.plan(Farlands.byId.get('lantern'),'j')),null);
    for(const [origin,target,title] of [[shell,'lantern','A place at the quay'],['lantern',shell,'A dry load home']]){
        await relocate(quest,origin);
        await talk(quest,origin);
        await accept(quest,title);
        await relocate(quest,target);
        await talk(quest,target);
        await quest.getByRole('button',{name:'Discuss the loading arrangements'}).click();
        t.assert(await quest.evaluate(id=>ebFarlands.storyFlag('rq-step1-j-'+id),origin));
        await modalClose(quest);
        await relocate(quest,origin);
        await talk(quest,origin);
        await quest.locator('#modalContent > section').filter({hasText:title}).getByRole('button',{name:/collect 12 gold/}).click();
        t.assert(await quest.evaluate(id=>ebFarlands.storyFlag('rq-done-j-'+id),origin));
        t.assert.equal(await quest.getByRole('button',{name:/collect 12 gold/}).count(),0);
        await modalClose(quest);
    }
    await quest.reload({waitUntil:'domcontentloaded'});
    await quest.click('#eb-menu-solo');
    await quest.waitForFunction(()=>ebFarlands.surface());
    t.assert.equal(await quest.evaluate(()=>gameState.inventory.gold),funds+24);
    t.assert(await quest.evaluate(()=>['lantern','settlement-lantern-0'].every(id=>ebFarlands.storyFlag('rq-done-j-'+id))));
    t.assert.deepEqual(quest.errors,[]);
    await quest.close();
    t.record('Phase 2 regional schedules, inn and coastal trade errands',{coverage,harbor,guests,gold:24});
    console.log('PASS coast trade handoffs and reload, port jobs, night patrons, deterministic census');
}

async function render(browser){
    const p=await t.page(browser,phone,noon+(harborOnly?80*1000:0));
    await siteAt(p,'lantern');
    await p.tap('#eb-immersive-toggle');
    await p.waitForFunction(()=>__view.eyeActive());
    if(harborOnly)await p.evaluate(()=>{
        const [x,y]=Farlands.coastalDocks.lantern.shore;
        const marker=[0,-1,-2].map(dx=>[x+dx,y-3]).find(([x,y])=>canEntityMoveTo(x,y));
        if(!marker)throw Error('No walkable harbor benchmark approach');
        Object.assign(gameState.player,{x:marker[0],y:marker[1],targetX:marker[0],targetY:marker[1]});
        const yaw=Math.atan2(marker[0]-x,marker[1]-y);
        __view.eyeYaw=__view.eyeRenderYaw=yaw;
        renderWorld();
    });
    const result=await p.evaluate(async()=>{
        const frames=[];
        let last=performance.now(),draw=__view.renderer.info.render.frame;
        for(let i=0;i<55;){
            await new Promise(requestAnimationFrame);
            const next=__view.renderer.info.render.frame;
            if(draw===next)continue;
            draw=next;
            const now=performance.now();
            if(i++>=7)frames.push({ms:now-last,calls:__view.renderer.info.render.calls,poses:__view.socialActors?.size??0});
            last=now;
        }
        return frames;
    });
    const sorted=result.map(r=>r.ms).sort((a,b)=>a-b);
    const summary={samples:result.length,p50:sorted[23],p95:sorted[45],maxPoses:Math.max(...result.map(r=>r.poses)),
        drawCalls:result.map(r=>r.calls).sort((a,b)=>a-b)[23]};
    if(!base)t.assert(summary.maxPoses<=6);
    console.log('mobile first person',summary);
    t.assert.deepEqual(p.errors,[]);
    await t.shot(p,'phase2-lantern-mobile-'+t.sha.slice(0,8));
    writeFileSync(path.join(t.out,'render.json'),JSON.stringify({sha:t.sha,summary,frames:result},null,2));
    t.record('Phase 2 mobile Chromium first-person rendering',summary);
    await p.close();
}

async function nightInn(browser){
    const page=await t.page(browser,undefined,new Date(noon+11*20*1000));
    await siteAt(page,'lantern');
    await page.evaluate(()=>openBuildingInterior(buildings.find(b=>b.realmId==='lantern-service-0')));
    await page.click('#eb-immersive-toggle');
    await page.waitForFunction(()=>__view.room&&__view.eyeActive());
    const frame=await page.evaluate(()=>__view.renderer.info.render.frame);
    await page.waitForFunction(frame=>__view.renderer.info.render.frame>=frame+4,frame);
    await t.shot(page,'phase2-lantern-inn-first-person-'+t.sha.slice(0,8));
    const visit=await page.evaluate(()=>{
        const guest=npcs.find(n=>n.realmId&&!n.roomHost);
        if(guest){
            Object.assign(gameState.player,{x:guest.x,y:guest.y});
            useWorldActor('npc',guest);
        }
        return {patrons:npcs.filter(n=>n.realmId).length,name:guest?.name,dialogue:guest?.dialogue};
    });
    if(!base){
        t.assert(visit.patrons>=1&&visit.patrons<=2);
        t.assert(visit.dialogue);
        await page.locator('#interactionModal').waitFor({state:'visible'});
        t.assert.equal(await page.locator('#modalTitle').textContent(),visit.name);
        t.assert((await page.locator('#modalContent').textContent()).includes(visit.dialogue));
        t.record('Night inn patron dialogue',visit);
    }
    t.assert.deepEqual(page.errors,[]);
    await page.close();
}

(async()=>{
    const browser=await t.launch();
    try{
        if(!base&&!harborOnly)await verify(browser);
        if(process.env.EB_LIFE_INN_ONLY!=='1')await render(browser);
        if(harborOnly)await nightInn(browser);
    }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
