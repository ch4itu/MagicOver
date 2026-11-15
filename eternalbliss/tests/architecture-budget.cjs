// Geometry/resource contract in Node using the bundled real THREE library; not a browser or FPS benchmark.
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict'),crypto=require('node:crypto');
const html=fs.readFileSync(require('node:path').join(__dirname,'../index.html'),'utf8');
const script=name=>{const token=`<script data-source="${name}">`,start=html.indexOf(token);assert(start>=0);return html.slice(start+token.length,html.indexOf('</script>',start));};
const ctx=vm.createContext({console});vm.runInContext(script('three.min.js'),ctx);vm.runInContext(script('gui-models.js')+';globalThis.art=EternalBlissArt;',ctx);
const a=ctx.art.building('inn',0,{w:3,h:2},'hearth',true),b=ctx.art.building('inn',0,{w:3,h:2},'hearth',true);
assert.notEqual(a.group,b.group);assert(a.group.children.length<=32,'authored inn has a bounded material batch count');
let vertices=0;for(let i=0;i<a.group.children.length;i++){const mesh=a.group.children[i];assert.equal(mesh.geometry,b.group.children[i].geometry,'revisits reuse geometry');assert.equal(mesh.material,b.group.children[i].material);const p=mesh.geometry.attributes.position.array,n=mesh.geometry.attributes.normal.array;assert(Array.from(p).every(Number.isFinite));assert(Array.from(n).every(Number.isFinite));vertices+=p.length/3;}
assert(vertices<15000,'single asset vertex budget');a.group.position.x=10;assert.equal(b.group.position.x,0,'instance transform isolation');
const generic=ctx.art.building('inn',0,{w:3,h:2},'hearth',false);assert(generic.group.children.length>a.group.children.length);
const result={time:new Date().toISOString(),sha:crypto.createHash('sha256').update(html).digest('hex'),script:crypto.createHash('sha256').update(fs.readFileSync(__filename)).digest('hex'),test:'authored inn geometry/cache budget',browser:'Node / bundled THREE geometry, no render/performance claim',result:{meshes:a.group.children.length,vertices,sharedGeometry:true,independentTransforms:true}};
fs.appendFileSync((process.env.EB_EVIDENCE||'/workspace/.magic-over-setup/overnight-evidence')+'/ledger.jsonl',JSON.stringify(result)+'\n');console.log('PASS',JSON.stringify(result));
