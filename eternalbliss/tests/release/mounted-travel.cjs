// Profile the shipped purchase, mount, held-input and dismount paths on a fixed Farlands lane.
const t=require('../browser.cjs');
const fs=require('node:fs');
const path=require('node:path');
const classic=process.env.EB_RIDE_VIEW==='2d';
const eye=process.env.EB_RIDE_VIEW==='first';
const trial=process.env.EB_RIDE_TRIAL||'normal';
const mobile=process.env.EB_RIDE_VIEW==='first';
(async()=>{
    const b=await t.launch();
    try{
        const p=await t.page(b,mobile?{viewport:{width:390,height:844},isMobile:true,hasTouch:true}:{viewport:{width:1024,height:700}},(1767225600+4*480+240)*1000);
        if(classic)await p.click('#eb-view-toggle');
        const horse=p.locator('#eb-farlands-horse');
        async function showHorse(){
            const panel=p.locator('#eb-actions-collapse');
            if(!await horse.isVisible() && await panel.isVisible() && await panel.getAttribute('aria-expanded')==='false')await panel.click();
            if(!await horse.isVisible())await p.click('#eb-actions-more');
        }
        await showHorse();
        for(let n=0;n<2;n++){await horse.click();await p.click('#eb-vehicle-accept');}
        if(eye)await p.click('#eb-immersive-toggle');
        const route=await p.evaluate(()=>{
            const x=4967,y=4604;
            const safe=t=>['grass','road','sand','stone','mud','snow'].includes(t);
            if(!Array.from({length:85},(_,n)=>safe(Farlands.sample(x+n,y))).every(Boolean))throw Error('Fixture lane changed');
            applyAdventureMap(captureAdventureMap(),{...gameState.player,x,y,targetX:x,targetY:y});
            __view.eyeYaw=__view.eyeRenderYaw=0;
            return {x,y,owned:ebFarlands.horseState().owned,mounted:ebHorseMounted};
        });
        t.assert(route.owned&&route.mounted);
        await p.waitForTimeout(900);
        await p.evaluate(({trial,classic})=>{
            const v=__view;
            window.ride={frames:[],steps:[],patches:[]};
            const renderer=v.renderer;
            if(trial==='pixels')renderer.setPixelRatio(renderer.getPixelRatio()*.65);
            if(trial==='shadow')renderer.shadowMap.enabled=false;
            const build=v.buildTerrain;
            v.buildTerrain=function(...args){
                const start=performance.now(),group=build.apply(this,args);
                ride.patches.push({t:start,ms:performance.now()-start});return group;
            };
            const render=renderer.render;
            let submitted=0;
            renderer.render=function(...args){submitted++;return render.apply(this,args);};
            const move=movePlayer;
            movePlayer=function(...args){
                const start=performance.now(),ok=move.apply(this,args);
                if(ok)ride.steps.push({t:start,ms:performance.now()-start,x:gameState.player.x,y:gameState.player.y});
                return ok;
            };
            const sample=()=>{
                const f={t:performance.now(),x:gameState.player.x};
                if(classic){
                    const rider=document.querySelector('.main-player-avatar'),mount=document.querySelector('#eb-horse-presence');
                    const css=node=>parseFloat(getComputedStyle(node).left);
                    if(rider && !mount.hidden)f.seatOffset=css(rider)-css(mount)-16;
                }else{
                    f.horse=v.farlandsHorse.position.x;f.rider=v.hero.group.position.x;
                    f.camera=eyeCameraX();f.phase=v.horseGaitPhase;f.draws=submitted;
                    if(v.terrainPatches){
                        const key=Math.floor(gameState.player.y/24)*gameState.world.width+Math.floor(gameState.player.x/24);
                        f.covered=!!v.terrainPatches.get(key)?.some(mesh=>mesh.visible);
                    }
                }
                ride.frames.push(f);requestAnimationFrame(sample);
            };
            function eyeCameraX(){return v.eyeActive()?v.eyeCamera.position.x:v.cameraPosition.x;}
            requestAnimationFrame(sample);
        },{trial,classic});
        if(mobile){
            const pad=await p.locator('#eb-joystick').boundingBox(),cx=pad.x+pad.width/2,cy=pad.y+pad.height/2;
            const cdp=await p.context().newCDPSession(p);
            await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x:cx,y:cy}]});
            await cdp.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:cx+pad.width*.4,y:cy}]});
            await p.waitForFunction(x=>gameState.player.x>=x+48,route.x,{timeout:90000,polling:100});
            await cdp.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});
        }else{
            await p.keyboard.down('ArrowRight');
            try{await p.waitForFunction(x=>gameState.player.x>=x+48,route.x,{timeout:90000,polling:100});}
            finally{await p.keyboard.up('ArrowRight');}
        }
        await p.waitForTimeout(800);
        const data=await p.evaluate(()=>({routeEnd:[gameState.player.x,gameState.player.y],...ride,
            software:__view.swiftShader,active:__view.active,eye:__view.eyeActive(),mounted:ebHorseMounted}));
        const name=[process.env.EB_RIDE_VIEW,trial,t.sha.slice(0,8),process.env.EB_RIDE_ROUND||1].join('-');
        fs.writeFileSync(path.join(t.out,name+'.json'),JSON.stringify({sha:t.sha,route,...data}));
        t.assert(data.mounted);t.assert(data.steps.length>=48);t.assert.equal(data.routeEnd[1],route.y);
        t.assert(data.frames.every(f=>f.covered!==false));
        if(!classic)t.assert(data.frames.every(f=>Math.abs(f.horse-f.rider)<.001));
        await t.shot(p,name);await showHorse();await horse.click();await p.click('#eb-vehicle-accept');
        t.assert(await p.evaluate(()=>!ebHorseMounted&&ebFarlands.horseState().owned));
        t.assert.deepEqual(p.errors,[]);console.log('PASS',name,data.steps.length,data.frames.length);
    }finally{await b.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
