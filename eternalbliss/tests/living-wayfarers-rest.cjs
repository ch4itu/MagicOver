const t = require('./browser.cjs');
const {writeFileSync} = require('node:fs');
const {join} = require('node:path');

const phone = {viewport:{width:390,height:844},hasTouch:true,isMobile:true};
const desktop = {viewport:{width:1024,height:700}};
const dateAt = hour => new Date((1767225600 + 4*480 + hour*20)*1000);
const before = process.env.EB_REST_BASELINE === '1';
const quantile = (values,p) => [...values].sort((a,b)=>a-b)[Math.ceil(values.length*p)-1];

async function measureMobile(browser) {
    const results=[];
    const page=await t.page(browser,phone,dateAt(14));
    await page.evaluate(()=>{
        const {x,y}=Farlands.start;
        if(!canEntityMoveTo(x+2,y+4))throw Error('blocked market camera fixture');
        Object.assign(gameState.player,{x:x+2,y:y+4,targetX:x+2,targetY:y+4});
        renderWorld();
    });
    for(const mode of ['overhead','first-person']){
        if(mode==='first-person'){
            await page.tap('#eb-immersive-toggle');
            await page.waitForFunction(()=>__view.eyeActive());
            await page.evaluate(()=>{__view.eyeYaw=-.32;__view.eyeRenderYaw=-.32;});
        }
        const frames=await page.evaluate(async()=>{
            const frames=[];
            let last=null, frame=__view.renderer.info.render.frame, warm=15;
            while(frames.length<96){
                await new Promise(requestAnimationFrame);
                const next=__view.renderer.info.render.frame;
                if(next===frame)continue;
                frame=next;
                const now=performance.now();
                if(warm>0)warm--;
                else frames.push({ms:now-last,calls:__view.renderer.info.render.calls});
                last=now;
            }
            return frames;
        });
        const ms=frames.map(f=>f.ms);
        const summary={mode,samples:ms.length,p50:quantile(ms,.5),p95:quantile(ms,.95),max:Math.max(...ms),
            gapsOver100:ms.filter(ms=>ms>=100).length,medianDrawCalls:quantile(frames.map(f=>f.calls),.5)};
        console.log('mobile',summary);
        await t.shot(page,'rest-market-'+mode+'-'+t.sha.slice(0,8));
        results.push({...summary,frames});
    }
    t.assert.deepEqual(page.errors,[]);
    await page.close();
    writeFileSync(join(t.out,'market-mobile.json'),JSON.stringify({sha:t.sha,results},null,2));
    t.record('Wayfarer’s Rest mobile market frame comparison',results.map(({frames,...summary})=>summary));
}

async function capture(browser) {
    const results = [];
    for (const [hour,label] of [[9,'morning'],[14,'afternoon'],[19,'evening'],[23,'night']]) {
        const page = await t.page(browser,desktop,dateAt(hour));
        await page.click('#eb-capture-toggle');
        const metrics = await page.evaluate(async () => {
            const {x,y}=Farlands.start;
            const actors=npcs.filter(n=>!n.fauna&&Math.hypot(n.x-x,n.y-y)<=12);
            const started=__view.renderer.info.render.frame;
            const frames=[];
            let last=performance.now();
            while(frames.length<16){
                await new Promise(requestAnimationFrame);
                const now=performance.now();
                if(__view.renderer.info.render.frame>started+4)frames.push(now-last);
                last=now;
            }
            return {clock:worldClimate.time,actors:actors.map(n=>n.name),drawCalls:__view.renderer.info.render.calls,frames};
        });
        metrics.p50=quantile(metrics.frames,.5);
        metrics.p95=quantile(metrics.frames,.95);
        await t.shot(page,'rest-'+label+'-'+t.sha.slice(0,8));
        t.assert.deepEqual(page.errors,[]);
        results.push(metrics);
        console.log(label,metrics.clock,metrics.actors.length,'residents',metrics.drawCalls,'calls',metrics.p50,metrics.p95);
        await page.close();
    }
    const page = await t.page(browser,phone,dateAt(14));
    await page.tap('#eb-immersive-toggle');
    await page.waitForFunction(()=>__view.eyeActive());
    results.push(await page.evaluate(async()=>{
        const frames=[];
        let last=performance.now();
        for(let i=0;i<21;i++){
            await new Promise(requestAnimationFrame);
            const now=performance.now();
            if(i>=5)frames.push(now-last);
            last=now;
        }
        return {mode:'emulated portrait first person',frames,drawCalls:__view.renderer.info.render.calls};
    }));
    const mobile=results.at(-1);
    mobile.p50=quantile(mobile.frames,.5);
    mobile.p95=quantile(mobile.frames,.95);
    console.log(mobile.mode,mobile.drawCalls,'calls',mobile.p50,mobile.p95);
    await t.shot(page,'rest-mobile-first-person-'+t.sha.slice(0,8));
    t.assert.deepEqual(page.errors,[]);
    await page.close();
    writeFileSync(join(t.out,'profiles.json'),JSON.stringify({sha:t.sha,results},null,2));
    t.record('Wayfarer’s Rest four-hour comparison and mobile first person',results);
}

async function verify(browser) {
    const page = await t.page(browser,desktop,dateAt(14));
    const validate = await page.evaluate(() => {
        const residents=ebLife.roster().filter(n=>n.restNeighbour);
        const blocked=[];
        for(const npc of residents){
            for(const goal of [...npc.routine.filter(s=>!s.home),npc.homeDoor]){
                if(!canEntityMoveTo(goal.x,goal.y))blocked.push([npc.name,goal.x,goal.y]);
            }
        }
        return {roster:residents.length,blocked};
    });
    t.assert.deepEqual(validate,{roster:5,blocked:[]});
    const navigation=await page.evaluate(()=>{
        const {x,y}=Farlands.start;
        const npc=ebLife.roster().find(n=>n.restNeighbour==='fen');
        const from=[npc.x,npc.y];
        const wall=worldClimate.time;
        worldClimate.time='19:00';
        let walked=0, blocked=0;
        let last=[npc.x,npc.y];
        const start=performance.now();
        for(let i=0;i<200;i++){
            ebLife.tick(start+i*175);
            walked+=Math.hypot(npc.x-last[0],npc.y-last[1]);
            if(!canEntityMoveTo(npc.x,npc.y))blocked++;
            last=[npc.x,npc.y];
        }
        const group=ebLife.activity(npc,start+36000);
        worldClimate.time=wall;
        return {from,arrived:last[0]===x-1&&last[1]===y-5,walked,blocked,group:group?.kind};
    });
    console.log('navigation',navigation);
    t.assert(navigation.arrived&&navigation.walked>=8&&navigation.blocked===0);
    t.assert(['speak','listen'].includes(navigation.group));
    const departures=await page.evaluate(()=>{
        const roster=ebLife.roster().filter(n=>n.restNeighbour);
        const clock=worldClimate.time;
        const failures=[];
        let time=performance.now()+37000;
        for(const hour of ['23:00','07:00','09:00','11:00','13:00','15:00','16:30','19:00','21:30','23:00']){
            worldClimate.time=hour;
            for(let i=0;i<240;i++){
                ebLife.tick(time+=175);
                for(const npc of roster)
                    if(!npc.indoors&&!canEntityMoveTo(npc.x,npc.y))failures.push([hour,npc.name,'blocked']);
            }
            for(const npc of roster){
                const slot=npc.life.slot;
                if(slot.home?!npc.indoors:npc.indoors||npc.x!==slot.x||npc.y!==slot.y)
                    failures.push([hour,npc.name,'missed destination']);
            }
            for(const name of ['Exchange order slate','Inn’s supper docket']){
                const post=[...Farlands.authoredProps.values()].find(prop=>prop.name===name);
                const readable=[-1,0,1].some(dx=>[-1,0,1].some(dy=>{
                    const x=post.x+dx,y=post.y+dy;
                    const reaches=entity=>Math.hypot(entity.x-x,entity.y-y)<=1.5;
                    return canEntityMoveTo(x,y)&&Farlands.propNear(x,y)===post&&
                        !npcs.some(npc=>!npc.fauna&&reaches(npc))&&!enemies.some(reaches)&&
                        !buildings.some(building=>buildingDistance(building,x,y)<=2);
                }));
                if(!readable)failures.push([hour,name,'no interaction approach']);
            }
        }
        worldClimate.time=clock;
        return failures;
    });
    t.assert.deepEqual(departures,[]);
    await page.close();

    const quest = await t.page(browser,desktop,dateAt(14));
    const gold = () => quest.evaluate(()=>gameState.inventory.gold);
    const startingGold = await gold();
    const close = () => quest.locator('#interactionModal button[onclick="closeModal()"]').click();
    const talk = async id => {
        await quest.evaluate(id=>{
            const npc=npcs.find(n=>n.restNeighbour===id);
            Object.assign(gameState.player,{x:npc.x,y:npc.y});
            useWorldActor('npc',npc);
        },id);
        await quest.locator('#interactionModal').waitFor({state:'visible'});
    };
    const card = title => quest.locator('#modalContent > section').filter({hasText:title});
    const questTab = async () => {
        const tab=quest.getByRole('tab',{name:/Quests/});
        if(await tab.isVisible())await tab.click();
    };
    for (const [id,giver,recipient,title,prop,amount,line] of [
        ['h','nell','fen','Bread for the common room','Exchange order slate',8,'six loaves'],
        ['i','fen','osric','A share for the watch','Inn’s supper docket',10,'night watch'],
    ]) {
        await talk(giver);
        await questTab();
        await card(title).getByRole('button',{name:'Accept quest'}).click();
        await close();
        await talk(recipient);
        t.assert(!await quest.evaluate(id=>ebFarlands.storyFlag('rq-step2-'+id+'-rest'),id));
        await close();
        await quest.evaluate(name=>{
            const prop=[...Farlands.authoredProps.values()].find(p=>p.name===name);
            Object.assign(gameState.player,{x:prop.x,y:prop.y});
            if(!canEntityMoveTo(prop.x,prop.y)||Farlands.propAt(prop.x,prop.y,false))throw Error('invalid solo prop');
            document.activeElement?.blur();
        },prop);
        await quest.keyboard.press('Space');
        await quest.locator('#modalTitle').filter({hasText:prop}).waitFor();
        t.assert((await quest.locator('#modalContent').textContent()).includes('Order noted'));
        await close();
        await talk(recipient);
        t.assert((await quest.locator('#modalContent').textContent()).includes(line));
        await close();
        const funds=await gold();
        await talk(giver);
        await questTab();
        await card(title).getByRole('button',{name:new RegExp('collect '+amount+' gold')}).click();
        t.assert.equal(await gold(),funds+amount);
        await close();
        await talk(giver);
        await questTab();
        t.assert.equal(await card(title).getByRole('button',{name:/collect/}).count(),0);
        await close();
    }
    await quest.reload({waitUntil:'domcontentloaded'});
    await quest.click('#eb-menu-solo');
    await quest.waitForFunction(()=>ebFarlands.surface());
    t.assert.equal(await gold(),startingGold+18);
    t.assert(await quest.evaluate(()=>['h','i'].every(k=>ebFarlands.storyFlag('rq-done-'+k+'-rest'))));
    t.assert.deepEqual(quest.errors,[]);
    await quest.close();

    const night = await t.page(browser,desktop,dateAt(23));
    await night.evaluate(()=>{
        const inn=buildings.find(b=>b.realmId==='rest-service-0');
        Object.assign(gameState.player,{x:inn.x,y:inn.y+2});
        enterBuilding(inn);
    });
    await night.waitForFunction(()=>__view.room && npcs.filter(n=>n.realmId?.startsWith('rest-neighbour-')).length===3);
    t.assert(await night.evaluate(()=>npcs.filter(n=>n.realmId?.startsWith('rest-neighbour-')).every(n=>canEntityMoveTo(n.x,n.y))));
    await t.shot(night,'rest-inn-night-'+t.sha.slice(0,8));
    await night.click('#eb-room-exit');
    await night.waitForFunction(()=>!buildingInterior && ebFarlands.surface());
    t.assert(!await night.evaluate(()=>npcs.some(n=>n.restNeighbour&&n.restHome==='inn')));
    t.assert.deepEqual(night.errors,[]);
    await night.close();
    t.record('Wayfarer’s Rest navigation, chained errands and inn', {validate,navigation,departures,questGold:18,innPatrons:3});
    console.log('PASS five routes, quest ordering and reload, nighttime patrons');
}

(async()=>{
    const browser=await t.launch();
    try {
        if(process.env.EB_REST_MOBILE_ONLY==='1'){await measureMobile(browser);return;}
        if(!before)await verify(browser);
        if(process.env.EB_REST_VERIFY_ONLY!=='1')await capture(browser);
    }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
