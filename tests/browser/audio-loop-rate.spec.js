import { test, expect } from '@playwright/test';

test('real WebAudio loops keep identity, isolation and failure cleanup',async({page})=>{
  await page.goto('/tests/browser/audio-loop-rate.html');
  await page.evaluate(()=>window.probeReady);
  await expect.poll(()=>page.evaluate(()=>window.probeStatus())).toBe('ready-close');
  const before=await page.evaluate(()=>{
    const s=window.rateProbe;
    return {
      contexts:s.contexts.length,gains:s.gains,
      sources:s.sources.map(r=>({rate:r.param.value,starts:r.starts,stops:r.stops,disconnects:r.disconnects,stage:r.stage})),
      writes:s.sources.map(r=>s.writes.filter(w=>w.param===r.param).map(w=>w.value)),
      mixWrites:s.writes.filter(w=>!s.sources.some(r=>r.param===w.param)).length,
    };
  });
  expect(before.contexts).toBe(1);expect(before.gains).toBe(4);
  expect(before.mixWrites).toBe(12); // initial setup + explicit mute/unmute only; rates never write gains
  expect(before.sources).toHaveLength(5); // two valid, two failed nodes, one survivor
  expect(before.writes[0]).toEqual([1.5,0.75,1,0.5,2,1,1.5,1]);expect(before.writes[1]).toEqual([]);
  expect(before.sources[0]).toMatchObject({rate:1,starts:1,stops:1,disconnects:1});
  expect(before.sources[1]).toMatchObject({rate:1,starts:1,stops:1,disconnects:1});
  for(const index of [2,3])expect(before.sources[index]).toMatchObject({starts:0,disconnects:1});
  await page.evaluate(async()=>{const s=window.rateProbe;await s.contexts[0].close();s.closed=true;});
  await expect.poll(()=>page.evaluate(()=>window.probeStatus())).toBe('PASS');
  expect(await page.evaluate(()=>window.rateProbe.sources[4].param.value)).toBe(1);
  expect(await page.evaluate(()=>window.rateProbe.sources[4].disconnects)).toBe(1);
});
