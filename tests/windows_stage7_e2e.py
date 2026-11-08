"""DESTRUCTIVE Stage 7 E2E. Use ONLY a disposable world.

The launcher normally disables Agent Control. This instrumented test alone
uses its local endpoint to arrange fixtures and observe results; module actions
are issued through the in-game /ih commands or ordinary UI/mouse input.
Set IMPACT_QA_ENDPOINT; optional IMPACT_QA_SHOTS. Run phases separately.
"""
import json, math, os, socket, subprocess, sys, time
from pathlib import Path
endpoint=Path(os.environ['IMPACT_QA_ENDPOINT'])
shots=Path(os.environ.get('IMPACT_QA_SHOTS','/tmp/impact7-shots'))
shots.mkdir(parents=True,exist_ok=True)

def call(line, timeout=15):
    info=json.loads(endpoint.read_text())
    with socket.create_connection(('127.0.0.1',info['port']),timeout=timeout) as s:
        s.settimeout(timeout);f=s.makefile('rwb',buffering=0)
        f.write(('auth '+info['token']+'\n').encode());assert json.loads(f.readline()).get('ok')
        f.write((line+'\n').encode());v=json.loads(f.readline())
    if v.get('error'):raise RuntimeError(f'{line}: {v}')
    return v

def wait(fn,timeout=30,label='condition'):
    deadline=time.monotonic()+timeout;last=None;error=None
    while time.monotonic()<deadline:
        try:
            last=fn()
            if last:return last
        except (OSError,ValueError,RuntimeError,KeyError,subprocess.CalledProcessError) as e:error=e
        time.sleep(.15)
    raise AssertionError(f'{label}: last={last}, error={error}')

def local(s):
    if call('state')['screen']!='playing':call('key esc tap');wait(lambda:call('state')['screen']=='playing',label='close screen')
    call('key t tap');call('type /ih '+s);call('key enter tap')
    wait(lambda:call('state')['screen']==('mod' if s=='menu' else 'playing'),label='chat closed')
    time.sleep(.6)

def command(s):return call('command '+s)
def status():return call('status')['impact_native']
def block(x,y,z):return call(f'block {x} {y} {z}')['block']
def pos():return call('state')['player']['position']
def tp(x,y,z):
    command(f'tp @s {x} {y} {z}')
    return wait(lambda:math.dist(pos(),[x,y,z])<.3,label='server teleport')
def fill(s,positions,expected):
    for _ in range(25):
        command(s)
        try:return wait(lambda:all(block(*p)==expected for p in positions),timeout=1.5,label='fill')
        except AssertionError:time.sleep(.3)
    raise AssertionError('fill not synchronized: '+s)
def give(item,count=64):
    call(f'give minecraft:{item} {count}')
    wait(lambda:call('inventory')['counts'].get('minecraft:'+item,0)>=count,label='inventory '+item)
def mob(kind):return next((m for m in call('entities 64')['entities'] if m['type']=='minecraft:'+kind),None)
def ui_click(part):
    w=wait(lambda:next((w for w in call('ui').get('widgets',[]) if part in w.get('label','')),None),label='UI '+part)
    x,y,wid,h=w['bounds'];call(f'click-at {x+wid/2} {y+h/2} left')
    return w

def select_point(p):
    call('look-at block '+' '.join(map(str,p)))
    call('mouse right click')
    time.sleep(.4)

def bridge_done(expected,timeout=150):
    def done():
        s=status()
        if s['bridge_next'] is None:
            messages=call('messages 15')['chat']
            outcome=None
            for m in messages:  # Newest first; don't accept success from a previous bridge.
                if '[Impact Native] Мост выбран.' in m:
                    break
                if any(marker in m for marker in ('[Impact Native] Мост построен.', '[Impact Native] Мост остановлен', '[Impact Native] Мост отменён', '[Impact Native] Navigation stopped')):
                    outcome=m
                    break
            if outcome and 'Мост построен.' not in outcome:
                raise AssertionError('bridge aborted: '+outcome)
            return bool(outcome) and all(block(*b)!='minecraft:air' for b in expected)
        return False
    try:wait(done,timeout=timeout,label='bridge completed')
    except Exception:
        print('BRIDGE DEBUG',json.dumps({'status':status(),'player':pos(),'messages':call('messages 12')}),flush=True)
        raise

wait(lambda:endpoint.exists(),timeout=30,label='endpoint')
wait(lambda:call('state').get('player'),timeout=180,label='world')
if call('state')['screen']!='playing':call('key esc tap')
local('stop')
phase=sys.argv[1] if len(sys.argv)>1 else 'visual'

if phase=='visual':
    local('friends clear')
    local('menu')
    labels=[w.get('label','') for w in call('ui').get('widgets',[])]
    for title in ('Bow Players','Attack Players','AutoCrit','Свои','Очистить список','Bridge: select','XRay','Tracers'):
        assert any(title in text for text in labels),(title,labels)
    subprocess.run(['import','-window','root',str(shots/'menu.png')],check=False)
    # Lower rows are intentionally offscreen at this GUI scale. Exercise the
    # real wheel rather than treating initially clipped buttons as missing.
    subprocess.run(['xdotool','click','--repeat','5','--delay','80','5'],check=True)
    wait(lambda:any('Список своих' in w.get('label','') for w in call('ui').get('widgets',[])),label='scrolled friend list button')
    assert any('X Diamond' in w.get('label','') for w in call('ui').get('widgets',[]))
    subprocess.run(['import','-window','root',str(shots/'menu-scrolled.png')],check=False)
    ui_click('Список своих')
    wait(lambda:call('state')['screen']=='playing',label='list button closes menu')
    wait(lambda:any('Список своих пуст.' in m for m in call('messages 8')['chat']),label='list button shows chat')
    print('PASS menu controls, real wheel scrolling and friend-list button',flush=True)
    x,y,z=100,100,200
    command(f'tp @s {x+3.5} {y+15} {z+3.5}')
    fill(f'fill {x} {y-1} {z} {x+16} {y-1} {z+6} minecraft:stone',[(x,y-1,z),(x+16,y-1,z+6)],'minecraft:stone')
    fill(f'fill {x} {y} {z} {x+16} {y+5} {z+6} minecraft:air',[(x,y,z),(x+16,y+5,z+6)],'minecraft:air')
    tp(x+2.5,y,z+3.5)
    command(f'setblock {x+8} {y+1} {z+3} minecraft:diamond_ore')
    command(f'setblock {x+9} {y+1} {z+3} minecraft:iron_ore')
    wait(lambda:block(x+8,y+1,z+3)=='minecraft:diamond_ore',label='diamond fixture')
    call('look -90 0');local('xray on');wait(lambda:status()['visual_xray'],label='xray on')
    time.sleep(5);subprocess.run(['import','-window','root',str(shots/'xray-all.png')],check=True)
    local('xrayfilter diamond off')
    s=wait(lambda:(v if (v:=status())['xray_mask'] & 1 == 0 else None),label='diamond hidden')
    time.sleep(5);subprocess.run(['import','-window','root',str(shots/'xray-no-diamond.png')],check=True)
    local('xrayblock stone on')
    wait(lambda:['minecraft:stone',True] in status()['xray_rules'],label='stone override')
    time.sleep(5);subprocess.run(['import','-window','root',str(shots/'xray-stone.png')],check=True)
    assert block(x+8,y+1,z+3)=='minecraft:diamond_ore'
    local('xrayblock clear');local('xrayfilter diamond on');local('xray off')
    print('PASS XRay categories and specific stone override propagate while world blocks remain unchanged; screenshots saved',flush=True)
    command('kill @e[type=minecraft:zombie,distance=..100]')
    command(f'summon minecraft:zombie {x+7.5} {y} {z+3.5} {{NoAI:1b}}')
    wait(lambda:mob('zombie'),label='zombie')
    call('look -90 0');local('tracers on')
    s=wait(lambda:(v if (v:=status())['tracers'] and v['esp_line_count']>=1 else None),label='tracer line')
    local('tracers off');wait(lambda:not status()['tracers'],label='tracer off')
    print('PASS hostile tracer adds an on-top world line:',s['esp_line_count'],flush=True)

elif phase=='bridge':
    command('clear @s');wait(lambda:not call('inventory')['counts'],label='clear test inventory')
    give('cobblestone',64)
    x,y,z=200,100,200
    command(f'tp @s {x+2.5} {y+15} {z+.5}')
    fill(f'fill {x-2} {y-4} {z-2} {x+8} {y+5} {z+2} minecraft:air',[(x+1,y-1,z),(x+6,y,z)],'minecraft:air')
    fill(f'fill {x-2} {y-5} {z-2} {x+8} {y-5} {z+2} minecraft:stone',[(x+1,y-5,z),(x+6,y-5,z)],'minecraft:stone')
    command(f'setblock {x} {y-1} {z} minecraft:stone');command(f'setblock {x+4} {y-1} {z} minecraft:stone')
    wait(lambda:block(x,y-1,z)=='minecraft:stone' and block(x+4,y-1,z)=='minecraft:stone',label='bridge anchors')
    tp(x+.5,y,z+.5);local('bridge');wait(lambda:status()['bridge_select'],label='bridge selecting')
    select_point([x,y-1,z]);wait(lambda:status()['bridge_first']==[x,y-1,z],label='first point')
    tp(x+4.5,y,z+.5);select_point([x+4,y-1,z])
    bridge_done([[x+i,y-1,z] for i in range(1,4)])
    assert all(block(x+i,y-1,z)=='minecraft:cobblestone' for i in range(1,4))
    print('PASS selects both endpoints using view+right click; flat 3-block bridge uses actual hotbar blocks',flush=True)

    x=230
    command(f'tp @s {x+2.5} {y+15} {z+.5}')
    fill(f'fill {x-2} {y-4} {z-2} {x+8} {y+6} {z+2} minecraft:air',[(x+1,y-1,z),(x+6,y,z)],'minecraft:air')
    fill(f'fill {x-2} {y-5} {z-2} {x+8} {y-5} {z+2} minecraft:stone',[(x+1,y-5,z),(x+6,y-5,z)],'minecraft:stone')
    command(f'setblock {x} {y-1} {z} minecraft:stone');command(f'setblock {x+5} {y+1} {z} minecraft:stone')
    wait(lambda:block(x,y-1,z)=='minecraft:stone' and block(x+5,y+1,z)=='minecraft:stone',label='slope anchors')
    tp(x+.5,y,z+.5);local('bridge');select_point([x,y-1,z]);wait(lambda:status()['bridge_first']==[x,y-1,z])
    tp(x+5.5,y+2,z+.5);select_point([x+5,y+1,z]);wait(lambda:status()['bridge_waiting'] is True,label='waiting for lower endpoint')
    tp(x+.5,y,z+.5)
    expected=[[x+1,y-1,z],[x+2,y-1,z],[x+3,y,z],[x+4,y,z]]
    bridge_done(expected,180)
    print('PASS bridge builds ascending intermediate step between anchors of different heights:',[(p,block(*p)) for p in expected],flush=True)

elif phase=='bridge_safety':
    command('clear @s');wait(lambda:not call('inventory')['counts'],label='empty hotbar')
    x,y,z=380,100,200
    command(f'tp @s {x+6.5} {y+15} {z+.5}')
    fill(f'fill {x-2} {y-4} {z-2} {x+14} {y+5} {z+2} minecraft:air',[(x+1,y-1,z),(x+11,y-1,z)],'minecraft:air')
    fill(f'fill {x-2} {y-5} {z-2} {x+14} {y-5} {z+2} minecraft:stone',[(x+1,y-5,z),(x+11,y-5,z)],'minecraft:stone')
    command(f'setblock {x} {y-1} {z} minecraft:stone');command(f'setblock {x+12} {y-1} {z} minecraft:stone')
    wait(lambda:block(x,y-1,z)=='minecraft:stone' and block(x+12,y-1,z)=='minecraft:stone',label='safety anchors')
    tp(x+.5,y,z+.5);local('bridge');select_point([x,y-1,z]);wait(lambda:status()['bridge_first']==[x,y-1,z])
    tp(x+12.5,y,z+.5);select_point([x+12,y-1,z])
    wait(lambda:status()['bridge_next'] is None and any('[Impact Native] Мост остановлен:' in m for m in call('messages 8')['chat']),timeout=35,label='no materials stops construction')
    assert all(block(x+i,y-1,z)=='minecraft:air' for i in range(1,12))
    print('PASS bridge stops with no suitable hotbar blocks and places nothing',flush=True)
    give('cobblestone',64)
    tp(x+.5,y,z+.5);local('bridge');select_point([x,y-1,z]);wait(lambda:status()['bridge_first']==[x,y-1,z])
    tp(x+12.5,y,z+.5);select_point([x+12,y-1,z])
    wait(lambda:status()['bridge_next'] is not None and status()['bridge_waiting'] is False,label='bridge active')
    call('key shift down')
    try:
        wait(lambda:status()['bridge_next'] is None,timeout=20,label='manual input cancels bridge')
    finally:
        call('key shift up')
    assert any('Мост остановлен' in m or 'Navigation stopped' in m for m in call('messages 8')['chat'])
    assert any(block(x+i,y-1,z)=='minecraft:air' for i in range(1,12))
    wait(lambda:not call('state')['player']['sneaking'],label='released shift is not stuck')
    print('PASS manual input cancels unfinished construction and shift releases normally',flush=True)
    tp(x+6.5,y+15,z+.5)
    fill(f'fill {x+1} {y-1} {z} {x+11} {y-1} {z} minecraft:air',[(x+1,y-1,z),(x+11,y-1,z)],'minecraft:air')
    command(f'setblock {x+11} {y-2} {z} minecraft:lava')
    wait(lambda:block(x+11,y-2,z)=='minecraft:lava',label='lava fixture')
    tp(x+.5,y,z+.5);local('bridge');select_point([x,y-1,z]);wait(lambda:status()['bridge_first']==[x,y-1,z])
    tp(x+12.5,y,z+.5);select_point([x+12,y-1,z])
    wait(lambda:status()['bridge_next'] is None and any('[Impact Native] Мост остановлен:' in m for m in call('messages 8')['chat']),timeout=35,label='lava stops construction')
    assert block(x+11,y-1,z)=='minecraft:air'
    print('PASS bridge refuses to place over a lava column',flush=True)

elif phase=='bridge_diagonal':
    command('clear @s');wait(lambda:not call('inventory')['counts'],label='clear test inventory')
    give('cobblestone',64)
    x,y,z=310,100,200
    command(f'tp @s {x+2.5} {y+15} {z+1.5}')
    fill(f'fill {x-2} {y-4} {z-2} {x+6} {y+5} {z+5} minecraft:air',[(x+1,y-1,z),(x+3,y,z+3)],'minecraft:air')
    fill(f'fill {x-2} {y-5} {z-2} {x+6} {y-5} {z+5} minecraft:stone',[(x+1,y-5,z),(x+3,y-5,z+3)],'minecraft:stone')
    command(f'setblock {x} {y-1} {z} minecraft:stone');command(f'setblock {x+4} {y-1} {z+3} minecraft:stone')
    wait(lambda:block(x,y-1,z)=='minecraft:stone' and block(x+4,y-1,z+3)=='minecraft:stone',label='diagonal anchors')
    tp(x+.5,y,z+.5);local('bridge');select_point([x,y-1,z]);wait(lambda:status()['bridge_first']==[x,y-1,z],label='diagonal first point')
    tp(x+4.5,y,z+3.5);select_point([x+4,y-1,z+3])
    expected=[[x+3,y-1,z+3],[x+3,y-1,z+2],[x+2,y-1,z+2],[x+2,y-1,z+1],[x+1,y-1,z+1],[x+1,y-1,z]]
    bridge_done(expected,180)
    assert all(block(*p)=='minecraft:cobblestone' for p in expected)
    print('PASS diagonal bridge follows a one-block-wide four-connected path:',expected,flush=True)

elif phase=='bridge_reverse':
    command('clear @s');wait(lambda:not call('inventory')['counts'],label='clear test inventory')
    give('cobblestone',64)
    x,y,z=260,100,200
    command(f'tp @s {x+2.5} {y+15} {z+.5}')
    fill(f'fill {x-2} {y-4} {z-2} {x+8} {y+6} {z+2} minecraft:air',[(x+1,y-1,z),(x+6,y,z)],'minecraft:air')
    fill(f'fill {x-2} {y-5} {z-2} {x+8} {y-5} {z+2} minecraft:stone',[(x+1,y-5,z),(x+6,y-5,z)],'minecraft:stone')
    # Select the high end first; lower end is on the positive-X side.
    command(f'setblock {x} {y+1} {z} minecraft:stone');command(f'setblock {x+5} {y-1} {z} minecraft:stone')
    wait(lambda:block(x,y+1,z)=='minecraft:stone' and block(x+5,y-1,z)=='minecraft:stone',label='reverse slope anchors')
    tp(x+.5,y+2,z+.5);local('bridge');select_point([x,y+1,z]);wait(lambda:status()['bridge_first']==[x,y+1,z],label='high point first')
    tp(x+5.5,y,z+.5);select_point([x+5,y-1,z])
    expected=[[x+4,y-1,z],[x+3,y-1,z],[x+2,y,z],[x+1,y,z]]
    bridge_done(expected,180)
    assert all(block(*p)=='minecraft:cobblestone' for p in expected)
    print('PASS high-to-low selection and reversed horizontal direction produce a one-block-wide slope, built from the lower end',flush=True)
    # Walk the finished slope down using ordinary movement, not teleportation.
    call('look -90 0');call('key w down')
    try:
        wait(lambda:pos()[0]>=x+4.3,timeout=20,label='walk down the slope')
    finally:
        call('key w up')
    time.sleep(1.5)
    pl=call('state')['player']; p=pl['position']
    assert pl['on_ground'] and abs(p[1]-y)<.08 and x+4.4<p[0]<x+6 and abs(p[2]-(z+.5))<.35,p
    print('PASS ordinary walking descends the completed slope to its lower end:',p,flush=True)

elif phase=='crits':
    command('clear @s');wait(lambda:not call('inventory')['counts'],label='clear test inventory')
    x,y,z=280,100,200
    command(f'tp @s {x+2.5} {y+15} {z+.5}')
    fill(f'fill {x-3} {y-1} {z-3} {x+8} {y-1} {z+3} minecraft:stone',[(x,y-1,z),(x+5,y-1,z)],'minecraft:stone')
    fill(f'fill {x-3} {y} {z-3} {x+8} {y+5} {z+3} minecraft:air',[(x,y,z),(x+5,y+3,z)],'minecraft:air')
    tp(x+.5,y,z+.5);command('kill @e[type=minecraft:zombie,distance=..100]')
    command(f'summon minecraft:zombie {x+2.5} {y} {z+.5} {{NoAI:1b}}')
    m=wait(lambda:mob('zombie'),label='crit zombie')
    give('diamond_sword',1);call('select minecraft:diamond_sword')
    local('crits on');wait(lambda:status()['crits'],label='crit mode')
    call('look-at entity '+str(m['id']));time.sleep(1.3)
    call('hurtlog') # drain previous records
    before=mob('zombie')['health'];initial_y=pos()[1];call('mouse left click')
    max_y=initial_y; deadline=time.monotonic()+10
    while time.monotonic()<deadline:
        max_y=max(max_y,pos()[1])
        current=mob('zombie')
        if current and current['health']<before:break
        time.sleep(.08)
    else:raise AssertionError(('critical damage did not arrive',status(),pos(),call('messages 6')))
    logs=call('hurtlog')
    attacks=[a for p in logs['players'] for a in p['attacks'] if a['id']==m['id']]
    assert attacks and any(a['crit'] and a['landed'] for a in attacks),(max_y,logs)
    assert max_y>initial_y+0.1,(max_y,initial_y,logs)
    print('PASS manual AutoCrit used a real jump and server recorded a landed critical:',initial_y,max_y,attacks,flush=True)
    local('crits off')
else:raise SystemExit('phase must be visual, bridge, bridge_safety, bridge_diagonal, bridge_reverse or crits')
