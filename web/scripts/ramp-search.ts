/**
 * How many ordinal steps fit between this site's surfaces and the CVD floor.
 *
 *   pnpm exec node --experimental-strip-types scripts/ramp-search.ts
 *
 * # The question, and why it needed searching rather than deciding
 *
 * `palette.spec.ts` measured the five-step ramp the design system contributed and found it at 10.9
 * light / 10.7 dark against the three-step ramp's 15.0 / 17.1. The design asked for a
 * reconstruction rather than an adjustment. This is that search: it optimises the ramp against the
 * same check, instead of constructing one and hoping.
 *
 * # What an ordinal ramp is, stated as constraints, after getting it wrong twice
 *
 * A first pass maximised pairwise separation over five free Lab points and cleared the bar
 * easily — with `#d092ff #7f2ab7 #a42e0f #797d85 #60be7f`. That is a categorical palette. It has
 * no lightness order and its hues jump, which is exactly what an ordinal scale must not be: the
 * whole point is that the order is visible in the colour rather than looked up in a legend. An
 * under-constrained optimiser will always find that answer, because separation is what it was
 * asked for and order was not.
 *
 * A second pass added monotone lightness and a hue arc, and returned infeasible everywhere,
 * because it also required *every* step to clear 2.2:1 against the ground. Only the step nearest
 * its own ground has to: the far end is dark against a light surface, or light against a dark one,
 * and clears by construction. Requiring it of all five forces the whole ramp into one corner of
 * the space and there is nothing there.
 *
 * So: monotone L* with a real gap, hue inside one arc, chroma bounded and moving smoothly (no grey
 * step in a coloured ramp), and the near step at 2.2:1 or better.
 *
 * # The answer
 *
 * The search is stochastic — random restarts, simulated annealing — so a run returns values within
 * about half a point of these rather than these exactly. The verdicts do not move.
 *
 *   light  4 steps   18.9  clears 15.0    arc 14 deg   L* span 68
 *   light  5 steps   15.1  clears 15.0    arc 39 deg   L* span 69
 *   dark   4 steps   17.2  clears 17.1    arc 40 deg   L* span 55
 *   dark   5 steps   13.2  FAILS  17.1
 *
 * **The dark surface is what binds.** Its contrast floor pushes the dark end of the ramp up in
 * lightness, leaving an L* span of about 55 against light mode's 68, and tritanopia takes most of
 * what a blue ramp has left. Five steps do not fit in what remains.
 *
 * Four steps fit in both modes — but dark clears by 0.1, which is not a margin, and both winning
 * ramps drift off the site's blue into violet and pink. A ramp that clears the bar and abandons
 * the palette to do it has traded one problem for another.
 *
 * **Recommendation: three steps.** They are measured, they are licensed for every form including
 * all-pairs, and they already ship. A quintile then gets position and a direct label, which is
 * what the site does today. This script stays so the question stays answerable — if the surfaces
 * change, or a wider hue budget is granted, run it again rather than deciding again.
 */

import { deltaE2000, toLab, parseHex, simulate, pairs, contrast, type Rgb }
  from "../src/lib/plot/palette.ts";
const VIS = ["normal","protan","deutan","tritan"] as const;
const W = { x: 0.95047, y: 1.0, z: 1.08883 };
const compand = (c: number) => c <= 0.0031308 ? c*12.92 : 1.055*c**(1/2.4)-0.055;
function fromLab(L: number, a: number, b: number): Rgb | null {
  const fy=(L+16)/116, fx=fy+a/500, fz=fy-b/200;
  const inv=(t:number)=> t>6/29 ? t**3 : 3*(6/29)**2*(t-4/29);
  const [X,Y,Z]=[inv(fx)*W.x, inv(fy)*W.y, inv(fz)*W.z];
  const o=[3.2404542*X-1.5371385*Y-0.4985314*Z, -0.9692660*X+1.8760108*Y+0.0415560*Z,
           0.0556434*X-0.2040259*Y+1.0572252*Z].map(compand);
  if (o.some(v=>v<-0.004||v>1.004)) return null;
  const [r,g,bb]=o.map(v=>Math.round(Math.min(1,Math.max(0,v))*255));
  return {r:r!,g:g!,b:bb!};
}
const hex=(c:Rgb)=>"#"+[c.r,c.g,c.b].map(x=>x.toString(16).padStart(2,"0")).join("");
const sep=(r:string[])=>Math.min(...VIS.map(v=>{const L=r.map(h=>toLab(simulate(parseHex(h),v)));
  return Math.min(...pairs(L).map(([a,b])=>deltaE2000(a,b)));}));

/** A presentable ordinal ramp: monotone L*, one narrow hue arc, chroma moving smoothly. */
function optimise(n: number, surface: string, dark: boolean, maxArc = 40, restarts = 500, iters = 3000) {
  const rnd=(a:number,b:number)=>a+Math.random()*(b-a);
  let best={ramp:[] as string[],score:-1,arc:0,span:0,chroma:[] as number[]};
  const score=(Ls:number[],hs:number[],Cs:number[])=>{
    for(let i=1;i<n;i++) if(Ls[i]!<=Ls[i-1]!+6) return -1;
    if(Math.max(...hs)-Math.min(...hs)>maxArc) return -1;
    for(const C of Cs) if(C<18||C>58) return -1;
    for(let i=1;i<n;i++) if(Math.abs(Cs[i]!-Cs[i-1]!)>14) return -1;   // no grey step in a coloured ramp
    const cols=Ls.map((L,i)=>fromLab(L,Cs[i]!*Math.cos(hs[i]!*Math.PI/180),Cs[i]!*Math.sin(hs[i]!*Math.PI/180)));
    if(cols.some(c=>!c)) return -1;
    const ramp=cols.map(c=>hex(c!));
    if(Math.min(...ramp.map(h=>contrast(parseHex(h),parseHex(surface))))<2.2) return -1;
    return sep(ramp);
  };
  for(let r=0;r<restarts;r++){
    const lo=dark?rnd(34,42):rnd(12,22), hi=dark?rnd(80,93):rnd(58,68);
    let Ls=Array.from({length:n},(_,i)=>lo+i*((hi-lo)/(n-1)));
    let h0=rnd(200,300); let hs=Array.from({length:n},(_,i)=>h0+i*rnd(-8,8));
    let Cs=Array.from({length:n},()=>rnd(25,45));
    let s=score(Ls,hs,Cs), T=8;
    for(let i=0;i<iters;i++){
      const nL=[...Ls],nh=[...hs],nC=[...Cs];
      const j=Math.floor(Math.random()*n), w=Math.random();
      if(w<0.4) nL[j]!+=rnd(-T,T); else if(w<0.7) nh[j]!+=rnd(-T*2,T*2); else nC[j]!+=rnd(-T,T);
      const cs=score(nL,nh,nC);
      if(cs>s){Ls=nL;hs=nh;Cs=nC;s=cs;}
      T=Math.max(0.3,T*0.9985);
    }
    if(s>best.score) best={ramp:Ls.map((L,i)=>hex(fromLab(L,Cs[i]!*Math.cos(hs[i]!*Math.PI/180),Cs[i]!*Math.sin(hs[i]!*Math.PI/180))!)),
      score:s,arc:Math.max(...hs)-Math.min(...hs),span:Ls[n-1]!-Ls[0]!,chroma:Cs.map(c=>+c.toFixed(0))};
  }
  return best;
}
for (const [mode,surface,bar,dark] of [["light","#fcfcfb",15.0,false],["dark","#1a1a19",17.1,true]] as const) {
  for (const n of [4, 5]) {
    const b = optimise(n, surface, dark);
    console.log(`  ${mode.padEnd(5)} ${n} steps  worst-pair ${b.score.toFixed(1).padStart(5)}  ${b.score>=bar?"CLEARS":"fails "} ${bar}   arc ${b.arc.toFixed(0).padStart(3)}deg  L* span ${b.span.toFixed(0)}  C ${b.chroma.join("/")}`);
    console.log(`        ${b.ramp.join("  ")}   contrast ${b.ramp.map(h=>contrast(parseHex(h),parseHex(surface)).toFixed(2)).join(" ")}`);
  }
}
