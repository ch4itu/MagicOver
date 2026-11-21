const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');

const candidate=process.env.EB_SOURCE||path.join(__dirname,'../../index.html');
const read=file=>new Map([...fs.readFileSync(file,'utf8').matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/g)]
    .map(([_,attrs,body],index)=>[/data-source="([^"]+)"/.exec(attrs)?.[1]||index,body]));
const before=read(process.argv[2]);
const after=read(candidate);
assert.equal(after.size,before.size);
const expected=new Set(['gui.js','gui-models.js','world.js','farlands-map.js','farlands.js','rpg-local-quests.js','r14-adventure-journal.js']);
for(const [name,body] of after){
    new vm.Script(body,{filename:String(name)});
    if(!expected.has(name))assert.ok(body===before.get(name),name+' must match the previous verified checkpoint');
}
assert(!fs.readFileSync(candidate,'utf8').includes('window.__view='));
console.log('PASS all scripts compile and scripts outside the intended scope match the previous checkpoint');
