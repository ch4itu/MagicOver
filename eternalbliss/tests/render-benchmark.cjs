const t = require('./browser.cjs');
const fs = require('node:fs');
const path = require('node:path');
const cases = [
    {name:'rest-overhead',eye:false,coast:false},
    {name:'rest-first-person',eye:true,coast:false},
    {name:'quay-afloat-first-person',eye:true,coast:true},
    {name:'bell-orchard-first-person',eye:true,site:'orchard',offset:[1,4]},
    {name:'shellwick-overhead',eye:false,site:'settlement-lantern-0',offset:[0,1]}
];

(async () => {
    const browser=await t.launch();
    try {
        const portrait=process.env.EB_RENDER_PROFILE==='portrait';
        const viewport=portrait?{width:390,height:844}:{width:844,height:390};
        const profile=portrait?'portrait':'landscape';
        const page=await t.page(browser,{viewport,hasTouch:true,isMobile:true},(1767225600+4*480+240)*1000);
        const renderer=await page.evaluate(()=>{
            const gl=__view.renderer.getContext();
            const debug=gl.getExtension('WEBGL_debug_renderer_info');
            return debug && gl.getParameter(debug.UNMASKED_RENDERER_WEBGL);
        });
        const results=[];
        const selected=process.env.EB_RENDER_SCENE;
        const skipSubmission=process.env.EB_RENDER_SKIP_SUBMIT==='1';
        const reuseShadow=process.env.EB_RENDER_REUSE_SHADOW==='1';
        const diagnostic=skipSubmission?'-no-submit':reuseShadow?'-reuse-shadow':'';
        const report={sha256:t.sha,renderer,profile:'Chromium '+profile+' viewport, '+viewport.width+' x '+viewport.height,results};
        const suffix=(portrait?'-portrait':'')+diagnostic;
        const output=path.join(t.out,'render-'+t.sha.slice(0,8)+suffix+'.json');
        for (const scene of cases) {
            if(selected ? !selected.split(',').includes(scene.name) : scene.site)continue;
            if (scene.eye!==await page.evaluate(()=>__view.eyeMode)) await page.tap('#eb-immersive-toggle');
            if (scene.coast) await page.evaluate(()=>{
                const [x,y]=Farlands.coastalDocks.lantern.water;
                gameState.sailingMoves=15;
                applyAdventureMap(captureAdventureMap(),{...gameState.player,x,y,targetX:x,targetY:y});
                __view.eyeYaw=__view.eyeRenderYaw=0;
            });
            if (scene.site) await page.evaluate(({site,offset})=>{
                const place=Farlands.byId.get(site);
                const x=place.x+offset[0],y=place.y+offset[1];
                gameState.sailingMoves=0;
                applyAdventureMap(captureAdventureMap(),{...gameState.player,x,y,targetX:x,targetY:y});
                __view.eyeYaw=__view.eyeRenderYaw=0;
            }, scene);
            const result=await page.evaluate(({samples,warmup,skipSubmission,reuseShadow})=>new Promise((resolve,reject)=>{
                const view=__view, renderer=view.renderer;
                const original={draw:view.draw,render:renderer.render,terrain:view.updateTerrain,actors:view.syncActors,
                    clone:THREE.Material.prototype.clone,shadowUpdates:renderer.shadowMap.autoUpdate};
                const intervals=[],drawMs=[],submitMs=[],terrainMs=[],actorMs=[];
                let last, frame=0, clones=0;
                let submitted, terrain, actors, frameClones, warmPrograms;
                const elapsed=(start)=>performance.now()-start;
                const summary=values=>{
                    const sorted=[...values].sort((a,b)=>a-b);
                    const percentile=p=>Number(sorted[Math.ceil(sorted.length*p)-1].toFixed(2));
                    return {median:percentile(.5),p90:percentile(.9),p95:percentile(.95),p99:percentile(.99),max:percentile(1)};
                };
                const restore=()=>{
                    view.draw=original.draw;view.updateTerrain=original.terrain;view.syncActors=original.actors;
                    renderer.render=original.render;THREE.Material.prototype.clone=original.clone;
                    renderer.shadowMap.autoUpdate=original.shadowUpdates;
                };
                const timeout=setTimeout(()=>{restore();reject(new Error('Render benchmark timed out'));},300000);
                THREE.Material.prototype.clone=function(){frameClones++;return original.clone.call(this);};
                renderer.render=function(...args){
                    if(skipSubmission && frame>=warmup)return;
                    if(reuseShadow && frame===warmup)renderer.shadowMap.autoUpdate=false;
                    const time=performance.now();
                    try{return original.render.apply(this,args);}finally{submitted+=elapsed(time);}
                };
                view.updateTerrain=function(){
                    const time=performance.now();
                    try{return original.terrain.call(this);}finally{terrain+=elapsed(time);}
                };
                view.syncActors=function(){
                    const time=performance.now();
                    try{return original.actors.call(this);}finally{actors+=elapsed(time);}
                };
                view.draw=function(now){
                    const time=performance.now();
                    submitted=terrain=actors=frameClones=0;
                    original.draw.call(this,now);
                    if(frame===warmup-1)warmPrograms=renderer.info.programs.length;
                    if (frame++>=warmup && last!==undefined) {
                        intervals.push(time-last);drawMs.push(elapsed(time));submitMs.push(submitted);
                        terrainMs.push(terrain);actorMs.push(actors);clones+=frameClones;
                    }
                    last=time;
                    if(intervals.length===samples){
                        clearTimeout(timeout);restore();
                        resolve({samples,warmup,intervalMs:summary(intervals),drawMs:summary(drawMs),
                            submitMs:summary(submitMs),terrainMs:summary(terrainMs),actorMs:summary(actorMs),materialClones:clones,
                            calls:renderer.info.render.calls,triangles:renderer.info.render.triangles,
                            shaderPrograms:{afterWarmup:warmPrograms,afterSamples:renderer.info.programs.length},
                            over100ms:intervals.filter(ms=>ms>100).length});
                    }
                };
            }),{samples:Number(process.env.EB_RENDER_SAMPLES||90),warmup:20,skipSubmission,reuseShadow});
            results.push({...scene,skipSubmission,reuseShadow,...result});
            console.log(JSON.stringify({renderer,...results.at(-1)}));
            fs.writeFileSync(output,JSON.stringify(report,null,2)+'\n');
            await t.shot(page,'benchmark-'+scene.name+'-'+t.sha.slice(0,8)+suffix);
        }
        t.assert.deepEqual(page.errors,[]);
        t.record('Warmed rendering benchmark',report);
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
