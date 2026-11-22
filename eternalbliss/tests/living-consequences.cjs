const t = require('./browser.cjs');
const fs = require('node:fs');
const path = require('node:path');

const base=process.env.EB_CONSEQUENCE_BASELINE==='1';
const clock=hour=>new Date((1767225600+4*480+hour*20)*1000);
const phone={viewport:{width:390,height:844},isMobile:true,hasTouch:true};
const key=(site,kind,part)=>'rq-'+part+'-'+kind+'-'+site;
const close=page=>page.keyboard.press('Escape');

async function relocate(page,id){
    await page.evaluate(id=>{
        const site=Farlands.byId.get(id);
        Object.assign(gameState.player,{x:site.x,y:site.y+1,targetX:site.x,targetY:site.y+1});
        ebFarlands.refreshScene(true);
        renderWorld();
    },id);
}
async function talk(page,name){
    await page.evaluate(name=>{
        const npc=npcs.find(n=>n.name===name);
        if(!npc)throw Error(name+' is absent');
        Object.assign(gameState.player,{x:npc.x,y:npc.y});
        useWorldActor('npc',npc);
    },name);
    await page.locator('#interactionModal').waitFor({state:'visible'});
}
async function action(page,title,label){
    const tab=page.getByRole('tab',{name:/Quests/});
    if(await tab.isVisible()&&await tab.getAttribute('aria-selected')!=='true')await tab.click();
    await page.locator('#modalContent > section').filter({hasText:title}).getByRole('button',{name:label}).click();
}
async function readBoard(page,name){
    await page.evaluate(name=>{
        const prop=[...Farlands.authoredProps.values()].find(p=>p.name===name);
        Object.assign(gameState.player,{x:prop.x,y:prop.y});
        document.activeElement?.blur();
    },name);
    await page.keyboard.press('Space');
    await page.locator('#modalTitle').filter({hasText:name}).waitFor();
    await close(page);
}
async function checkReload(page, flags){
    await page.reload({waitUntil:'domcontentloaded'});
    await page.click('#eb-menu-solo');
    await page.waitForFunction(()=>ebFarlands.surface());
    t.assert(await page.evaluate(flags=>flags.every(flag=>ebFarlands.storyFlag(flag)),flags));
}
async function verify(browser){
    const p=await t.page(browser,undefined,clock(14));
    const schedule=(hour,site)=>p.evaluate(({hour,site})=>{
        const actor=ebLife.roster().filter(n=>site==='rest'?['nell','fen','piper','osric'].includes(n.restNeighbour):
            n.ebSite===site&&['Fisher','Dockworker'].includes(n.ebRole));
        const original=worldClimate.time;
        worldClimate.time=hour;
        const now=performance.now()+50000;
        let blocked=0;
        for(let i=0;i<220;i++){
            ebLife.tick(now+i*180);
            for(const n of actor)if(!n.indoors&&!canEntityMoveTo(n.x,n.y))blocked++;
        }
        const stops=actor.map(n=>({name:n.name,event:n.life.slot.event,where:n.life.slot.where,indoors:n.indoors,
            arrived:n.x===n.life.slot.x&&n.y===n.life.slot.y,x:n.x,y:n.y}));
        worldClimate.time=original;
        return {stops,blocked};
    },{hour,site});
    const initial=await schedule('19:00','rest');
    t.assert(initial.stops.every(n=>!n.event));
    t.assert(await p.evaluate(()=>!ebLife.followup(npcs.find(n=>n.restNeighbour==='nell'))));
    await schedule('14:00','rest');
    const dayGold=await p.evaluate(()=>gameState.inventory.gold);
    for(const [id,giver,recipient,title,board,amount] of [
        ['h','Nell, the Baker','Fen, the Market Carter','Bread for the common room','Exchange order slate',8],
        ['i','Fen, the Market Carter','Osric, the Watchman','A share for the watch','Inn’s supper docket',10]
    ]){
        await talk(p,giver);
        await action(p,title,'Accept quest');
        await close(p);
        await readBoard(p,board);
        await talk(p,recipient);
        t.assert(await p.evaluate(k=>ebFarlands.storyFlag(k),key('rest',id,'step2')));
        await close(p);
        await talk(p,giver);
        await action(p,title,new RegExp('collect '+amount+' gold'));
        t.assert.equal(await p.getByRole('button',{name:new RegExp('collect '+amount+' gold')}).count(),0);
        await close(p);
    }
    t.assert.equal(await p.evaluate(()=>gameState.inventory.gold),dayGold+18);
    const market=await schedule('19:00','rest');
    const supper=await schedule('17:30','rest');
    t.assert.equal(market.blocked+supper.blocked,0);
    t.assert.equal(market.stops.filter(n=>n.event==='late-market'&&n.where==='market'&&n.arrived).length,2);
    const gathering=supper.stops.filter(n=>n.event==='watch-supper'&&n.arrived);
    t.assert.equal(gathering.length,2);
    t.assert(Math.hypot(gathering[0].x-gathering[1].x,gathering[0].y-gathering[1].y)<=2.6);
    t.assert(market.stops.some((n,i)=>n.event==='late-market'&&(n.x!==initial.stops[i].x||n.y!==initial.stops[i].y)));
    await checkReload(p,[key('rest','h','done'),key('rest','i','done')]);
    const settled=await p.evaluate(()=>{
        const n=npcs.find(n=>n.restNeighbour==='nell');
        return ebLife.followup(n)?.line;
    });
    t.assert(settled.includes('You got the inn its bread'));
    await talk(p,'Osric, the Watchman');
    await p.locator('[data-eb-followup]').filter({hasText:'Hot bread on watch was your doing'}).waitFor();
    await p.getByRole('button',{name:'Mark Millbend on the Atlas'}).click();
    t.assert.equal(await p.evaluate(()=>ebFarlands.pinned()),'mill');
    await close(p);

    const shell='settlement-lantern-0';
    await relocate(p,'lantern');
    const closedCargo=await schedule('20:45','lantern');
    t.assert(closedCargo.stops.every(n=>!n.event));
    await schedule('14:00','lantern');
    for(const [from,to,giver,recipient,title] of [
        [shell,'lantern','Shellwick Waykeeper','Lantern Quay Waykeeper','A place at the quay'],
        ['lantern',shell,'Lantern Quay Waykeeper','Shellwick Waykeeper','A dry load home']
    ]){
        await relocate(p,from);
        await talk(p,giver);
        await action(p,title,'Accept quest');
        await close(p);
        await relocate(p,to);
        await talk(p,recipient);
        await p.getByRole('button',{name:'Discuss the loading arrangements'}).click();
        await close(p);
        await relocate(p,from);
        await talk(p,giver);
        await action(p,title,/collect 12 gold/);
        await close(p);
    }
    await checkReload(p,[key(shell,'j','done'),key('lantern','j','done')]);
    await relocate(p,'lantern');
    const loaded=await schedule('20:45','lantern');
    t.assert.equal(loaded.blocked,0);
    t.assert(loaded.stops.every(n=>n.event==='late-cargo'&&n.where==='landing'&&n.arrived&&!n.indoors));
    t.assert(loaded.stops.some((n,i)=>n.x!==closedCargo.stops[i].x||n.y!==closedCargo.stops[i].y));
    const over=await schedule('22:30','lantern');
    t.assert(over.stops.every(n=>!n.event));
    await schedule('14:00','lantern');
    await talk(p,'Lantern Quay Waykeeper');
    await p.locator('[data-eb-followup]').filter({hasText:'Your instructions reached Shellwick'}).waitFor();
    await t.shot(p,'phase3-revisited-quay-'+t.sha.slice(0,8));
    t.assert.deepEqual(p.errors,[]);
    await p.close();
    t.record('Earned Rest and Lantern consequences, expiry, recognition and Atlas lead',{initial,market,supper,closedCargo,loaded,over});
    console.log('PASS four earned quests, payout and reload, shifted routines, follow-up and Atlas lead');
}
async function render(browser){
    const page=await t.page(browser,phone,clock(19));
    await page.tap('#eb-immersive-toggle');
    await page.waitForFunction(()=>__view.eyeActive());
    if(!base)await page.evaluate(()=>{
        // Isolated render fixture only. The playthrough above earns these in the quest UI.
        for(const part of ['taken','step1','step2'])ebFarlands.markRpgQuest('rq-'+part+'-h-rest');
        ebFarlands.completeRpgQuest('rest','h');
    });
    await page.evaluate(()=>{
        const {x,y}=Farlands.start;
        if(!canEntityMoveTo(x+5,y+3))throw Error('blocked market view');
        Object.assign(gameState.player,{x:x+5,y:y+3,targetX:x+5,targetY:y+3});
        __view.eyeYaw=__view.eyeRenderYaw=0;
        renderWorld();
    });
    const frames=await page.evaluate(async()=>{
        let last=performance.now(),frame=__view.renderer.info.render.frame;
        const result=[];
        for(let i=0;i<54;){
            await new Promise(requestAnimationFrame);
            if(__view.renderer.info.render.frame===frame)continue;
            frame=__view.renderer.info.render.frame;
            const now=performance.now();
            if(i++>=6)result.push({ms:now-last,poses:__view.socialActors?.size??0,calls:__view.renderer.info.render.calls});
            last=now;
        }
        return result;
    });
    const elapsed=frames.map(f=>f.ms).sort((a,b)=>a-b);
    const summary={samples:frames.length,p50:elapsed[23],p95:elapsed[45],
        maxPoses:Math.max(...frames.map(f=>f.poses)),drawCalls:frames.map(f=>f.calls).sort((a,b)=>a-b)[23]};
    t.assert(summary.maxPoses<=6);
    t.assert.deepEqual(page.errors,[]);
    await t.shot(page,'phase3-mobile-'+t.sha.slice(0,8));
    fs.writeFileSync(path.join(t.out,'render.json'),JSON.stringify({sha:t.sha,summary,frames},null,2));
    t.record('Mobile Chromium first person at dusk',summary);
    console.log(summary);
    await page.close();
}

(async()=>{
    const browser=await t.launch();
    try{
        if(!base)await verify(browser);
        await render(browser);
    }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
