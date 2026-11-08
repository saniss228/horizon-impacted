"""Physical-input smoke test of the 1.0.0 Windows launcher under Wine/Xvfb.
Run only in a disposable world on a 1280x720 display. The launcher must have
already been started with a test --control-socket argument, with output
redirected to IMPACT_QA_GAME_LOG. It must ignore that argument for this session.
No test endpoint or external control program participates in module behavior.
"""
import os, subprocess, time
from pathlib import Path

log = Path(os.environ['IMPACT_QA_GAME_LOG'])
endpoint = Path(os.environ['IMPACT_QA_DISABLED_ENDPOINT'])
screenshot = Path(os.environ.get('IMPACT_QA_SCREENSHOT', '/tmp/impact7-launcher-menu.png'))

def xdo(*args):
    subprocess.run(['xdotool', *map(str,args)],check=True,stdout=subprocess.DEVNULL)

def wait_text(text, timeout=90):
    end=time.monotonic()+timeout
    while time.monotonic()<end:
        assert not endpoint.exists(), 'launcher opened a forbidden Agent Control endpoint'
        if log.exists() and text in log.read_text(errors='replace'):return
        time.sleep(.15)
    raise AssertionError('not found in launcher log: '+text)

wait_text('Agent Control disabled for this session')
wait_text('joined the game')
wait_text('first frame after')
window=subprocess.check_output(['xdotool','search','--name','^Horizon$'],text=True).split()[-1]
xdo('windowfocus','--sync',window);time.sleep(.7)
xdo('key','F8');time.sleep(1.5)
subprocess.run(['import','-window','root',str(screenshot)],check=True)
# The main view at 1280x720 places XRay in column 3, row 5.
xdo('mousemove',640,415);xdo('click',1);wait_text('X-Ray ON: terrain is hidden')
time.sleep(.5);xdo('click',1);wait_text('X-Ray OFF: rebuilding normal terrain')
xdo('key','Escape');time.sleep(.4)
assert not endpoint.exists()
print('PASS exact launcher in a path with spaces: prepared resource path, F8, XRay on/off and Agent Control disabled',flush=True)
