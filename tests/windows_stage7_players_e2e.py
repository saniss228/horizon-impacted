"""DESTRUCTIVE two-client friend/players E2E in a disposable world only.

Set IMPACT_QA_ENDPOINT to instrumented Windows host's endpoint and
IMPACT_QA_GUEST_ENDPOINT to a second, unmodified Horizon offline guest.
First run `open` with host, then launch guest named FriendQA on 127.0.0.1:25581,
then run `players`. The shipping launcher does not open either control socket.
"""
import json, math, os, socket, subprocess, sys, time
from pathlib import Path
host=Path(os.environ['IMPACT_QA_ENDPOINT'])
guest=Path(os.environ.get('IMPACT_QA_GUEST_ENDPOINT','/tmp/impact7-guest.json'))

def call(line,p=host,timeout=15):
    i=json.loads(p.read_text())
    with socket.create_connection(('127.0.0.1',i['port']),timeout=timeout) as s:
        s.settimeout(timeout);f=s.makefile('rwb',buffering=0)
        f.write(('auth '+i['token']+'\n').encode());assert json.loads(f.readline()).get('ok')
        f.write((line+'\n').encode());v=json.loads(f.readline())
    if v.get('error'):raise RuntimeError(f'{line}: {v}')
    return v

def wait(fn,timeout=30,label='condition'):
    end=time.monotonic()+timeout;last=None;err=None
    while time.monotonic()<end:
        try:
            last=fn()
            if last:return last
        except (OSError,RuntimeError,ValueError,KeyError) as e:err=e
        time.sleep(.12)
    raise AssertionError(f'{label}: last={last} error={err}')
def status():return call('status')['impact_native']
def local(s):
    if call('state')['screen']!='playing':call('key esc tap');wait(lambda:call('state')['screen']=='playing')
    call('key t tap');call('type /ih '+s);call('key enter tap')
    wait(lambda:call('state')['screen']==('mod' if s=='menu' else 'playing'),label='close chat')
    time.sleep(.7)
def command(s):return call('command '+s)
def give(item,count=1):
    call(f'give minecraft:{item} {count}')
    wait(lambda:call('inventory')['counts'].get('minecraft:'+item,0)>=count,timeout=45,label='inventory '+item)
def pos():return call('state')['player']['position']
def gp():return call('state',guest)['player']['position']
def gstatus():
    return next((g for g in call('lan status')['guests'] if g['name']=='FriendQA'),None)
def hp():return gstatus()['health']
def tp_host(x,y,z):command(f'tp @s {x} {y} {z}');wait(lambda:math.dist(pos(),[x,y,z])<.35,label='host tp')
def tp_guest(x,y,z):command(f'tp FriendQA {x} {y} {z}');wait(lambda:math.dist(gp(),[x,y,z])<.5,timeout=40,label='guest tp')
def aim_guest():
    x,y,z=gp();call(f'look-at {x} {y+0.9} {z}')
def ui_click(part):
    w=wait(lambda:next((w for w in call('ui').get('widgets',[]) if part in w.get('label','')),None),label='UI '+part)
    x,y,width,height=w['bounds'];call(f'click-at {x+width/2} {y+height/2} left')

wait(lambda:host.exists(),label='host control')
wait(lambda:call('state').get('player'),timeout=180,label='host world')
phase=sys.argv[1]
if phase=='open':
    local('stop')
    x,y,z=350,100,200
    command(f'tp @s {x+3.5} {y+15} {z+5.5}')
    for _ in range(20):
        command(f'fill {x-5} {y-1} {z-8} {x+25} {y-1} {z+12} minecraft:stone')
        try:wait(lambda:call(f'block {x} {y-1} {z}')['block']=='minecraft:stone',timeout=2);break
        except AssertionError:time.sleep(.5)
    command(f'fill {x-5} {y} {z-8} {x+25} {y+5} {z+12} minecraft:air')
    tp_host(x+.5,y,z+.5)
    if not call('lan status')['open']:call('lan open 25581 commands')
    assert call('lan status')['open']
    print('LAN ready for offline FriendQA on 127.0.0.1:25581',flush=True)
    raise SystemExit(0)

if phase=='persist_check':
    wait(lambda:status()['friends_count']==1,timeout=45,label='persisted friend reloaded')
    local('friends list')
    wait(lambda:any('Свои (1): FriendQA' in line for line in call('messages 8')['chat']),label='persisted friend name')
    print('PASS FriendQA is restored after restarting the client',flush=True)
    raise SystemExit(0)

assert phase in ('players','persist_add')
wait(lambda:guest.exists(),timeout=90,label='guest control')
wait(lambda:call('state',guest).get('player'),timeout=180,label='guest joined world')
wait(lambda:gstatus() and gstatus()['loaded'],timeout=90,label='LAN guest loaded')
if phase=='persist_add':
    local('stop');local('friends clear')
    x,y,z=350,100,200
    tp_host(x+.5,y,z+.5);tp_guest(x+2.5,y,z+.5)
    wait(lambda:'FriendQA' in (call('status')['name_tags'] or []),timeout=60,label='host sees guest')
    start_hp=hp();aim_guest();local('friends select');wait(lambda:status()['friend_select'],label='select mode')
    aim_guest();call('mouse left click');wait(lambda:status()['friends_count']==1,label='friend added')
    time.sleep(.5);assert hp()==start_hp,(start_hp,hp())
    print('PASS selected FriendQA without damage; restart host before running persist_check',flush=True)
    raise SystemExit(0)

local('stop');local('crits off');local('friends clear')
command('clear @s');wait(lambda:not call('inventory')['counts'],label='clear test inventory')
x,y,z=350,100,200
command('gamemode survival FriendQA')
# Prevent ordinary hunger healing from masking damage or breaking equality checks.
command('gamerule minecraft:natural_health_regeneration false')
tp_host(x+.5,y,z+.5);tp_guest(x+10.5,y,z+.5)
command('effect give FriendQA minecraft:instant_health 2 10')
wait(lambda:hp()>=19.9,label='guest healed')
wait(lambda:'FriendQA' in (call('status')['name_tags'] or []),timeout=60,label='host sees remote player')
# While aiming an ordinary drawn bow, a genuinely moving guest should get a lead.
give('bow');call('select minecraft:bow');local('pbow on')
call('mouse right down');wait(lambda:call('status')['using'] is not None,label='bow draw')
call('look 0 0',guest);call('key d down',guest)
observation=None
end=time.monotonic()+18
while time.monotonic()<end:
    p=gp(); lead=status().get('lead')
    if lead and abs(lead['target'][0]-p[0])>0.06:
        observation=(p,lead);break
    time.sleep(.12)
call('key d up',guest);call('mouse right up')
assert observation,('no observable player lead',gp(),status())
print('PASS Bow Players computes lead for actual moving remote player:',observation,flush=True)
local('pbow off')

# Tracer to enemy exists before adding them; use an isolated fixture with no mobs.
tp_guest(x+2.5,y,z+.5);aim_guest();local('tracers on')
wait(lambda:status()['esp_line_count']>=1,label='player tracer before friend')
# The menu click that closes the screen must not leak as a queued world attack.
start_hp=hp();local('menu');ui_click('Свои')
wait(lambda:call('state')['screen']=='playing',label='friends button closes menu')
wait(lambda:status()['friend_select'],label='friend click mode')
time.sleep(.5);assert hp()==start_hp,('menu click hit target',start_hp,hp())
aim_guest();call('mouse left click')
wait(lambda:status()['friends_count']==1,label='friend added')
time.sleep(.8);assert hp()==start_hp,(start_hp,hp())
wait(lambda:status()['esp_line_count']==0,label='friend excluded from tracers')
local('friends list')
wait(lambda:any('Свои (1): FriendQA' in line for line in call('messages 8')['chat']),label='friend list shown in chat')
seen=sum('Свои (1): FriendQA' in line for line in call('messages 30')['chat'])
local('menu')
subprocess.run(['xdotool','click','--repeat','5','--delay','80','5'],check=True)
ui_click('Список своих')
wait(lambda:call('state')['screen']=='playing',label='friend list closes menu')
wait(lambda:sum('Свои (1): FriendQA' in line for line in call('messages 30')['chat'])>seen,label='friend list button shows FriendQA in chat')
assert hp()==start_hp,('list button hit selected friend',start_hp,hp())
print('PASS selection LMB adds real player without damage; list button shows FriendQA and tracer disappears',flush=True)

# Manual and automatic hits are both blocked at attack/send tick.
give('diamond_sword');call('select minecraft:diamond_sword')
aim_guest();call('mouse left click');time.sleep(1.5);assert hp()==start_hp,(start_hp,hp())
local('range 3');local('pattack on');time.sleep(3.0)
assert hp()==start_hp,(start_hp,hp())
print('PASS manual and automatic melee do not reduce marked friend health',flush=True)
local('pattack off');local('pbow on');give('bow');call('select minecraft:bow');call('mouse right down')
time.sleep(1.5);assert status()['lead'] is None,status()
call('mouse right up');local('pbow off');print('PASS Bow Players excludes friend',flush=True)

# The separate clear button restores enemy targeting and damage.
local('menu');ui_click('Очистить список');wait(lambda:status()['friends_count']==0,label='friends cleared')
call('key esc tap');wait(lambda:call('state')['screen']=='playing')
call('select minecraft:diamond_sword');local('pattack on')
wait(lambda:hp()<start_hp,timeout=15,label='unmarked player is attackable')
local('pattack off');local('tracers off')
print('PASS clear-list button restores targeting and a server-authoritative damage event:',start_hp,'->',hp(),flush=True)
