"""Compile the exact pure helpers and Rust unit tests from impact_native.rs.

Usage: python3 test_native_logic.py /path/to/patched/horizon checkout
The extraction avoids compiling the whole windowed game just for three unit
tests; end-to-end tests still require the actual client.
"""
import pathlib, subprocess, sys, tempfile

root=pathlib.Path(sys.argv[1])
source=(root/'crates/horizon-client/src/game/impact_native.rs').read_text()

def item(marker):
    begin=source.index(marker)
    open_brace=source.index('{',begin)
    depth=0
    for i in range(open_brace,len(source)):
        if source[i]=='{':depth+=1
        if source[i]=='}':
            depth-=1
            if not depth:return source[begin:i+1]
    raise AssertionError('unterminated '+marker)

body='\n\n'.join(item(x) for x in [
    'fn hostile(', 'fn ore_matches(', 'fn number(', 'fn xray_rules(', 'fn switch(', 'mod tests {'])
with tempfile.TemporaryDirectory(prefix='impact-native-unit-') as d:
    file=pathlib.Path(d)/'logic.rs'; exe=pathlib.Path(d)/'logic-test'
    file.write_text(body)
    subprocess.run(['rustc','--edition','2024','--test',str(file),'-o',str(exe)],check=True)
    subprocess.run([str(exe)],check=True)
    # This is the exact independent ballistics/section math used by the host.
    math_path=root/'crates/horizon-client/src/game/impact_native_math.rs'
    wrapper=pathlib.Path(d)/'math.rs'
    wrapper.write_text('#[path="'+str(math_path.resolve())+'"]\nmod native_math;\n')
    math_exe=pathlib.Path(d)/'math-test'
    subprocess.run(['rustc','--edition','2024','--test',str(wrapper),'-o',str(math_exe)],check=True)
    subprocess.run([str(math_exe)],check=True)

    # Exact single-block bridge centreline (ascending even on reverse selection).
    bridge_path=root/'crates/horizon-client/src/game/impact_native_bridge.rs'
    bridge_wrapper=pathlib.Path(d)/'bridge.rs'
    bridge_wrapper.write_text('#[path="'+str(bridge_path.resolve())+'"]\nmod bridge;\n')
    bridge_exe=pathlib.Path(d)/'bridge-test'
    subprocess.run(['rustc','--edition','2024','--test',str(bridge_wrapper),'-o',str(bridge_exe)],check=True)
    subprocess.run([str(bridge_exe)],check=True)

    # Extract the exact mesher filter function, without compiling the renderer.
    mesh=(root/'crates/horizon-client/src/mesher.rs').read_text()
    start=mesh.index('pub const XRAY_ALL:')
    end=mesh.index('\n/// Per-state data',start)
    xray=pathlib.Path(d)/'xray.rs'
    xray.write_text(mesh[start:end]+"""\n#[test] fn block_categories() {\nassert_eq!(xray_category("minecraft:diamond_ore"),1);\nassert_eq!(xray_category("deepslate_diamond_ore"),1);\nassert_eq!(xray_category("minecraft:stone"),0);\nassert_eq!(xray_category("minecraft:lava"),1<<11);\nassert_eq!(xray_category("custom_ore"),1<<12);\nassert_eq!(XRAY_ALL,(1<<13)-1);\n}\n""")
    xray_exe=pathlib.Path(d)/'xray-test'
    subprocess.run(['rustc','--edition','2024','--test',str(xray),'-o',str(xray_exe)],check=True)
    subprocess.run([str(xray_exe)],check=True)
