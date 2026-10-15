#!/usr/bin/env python3
"""Exercise isolated real processes over actual DNS-SD and pinned TLS on this Mac.
Synthetic messages only. Does not certify physical-router discovery.
"""
import argparse, json, pathlib, select, subprocess, tempfile, time, os, uuid
ROOT = pathlib.Path(__file__).resolve().parents[2]
BIN = ROOT / 'src-tauri/target/debug/examples/local-chat-peer'
class Peer:
    def __init__(self, directory, name, service):
        self.process = subprocess.Popen([str(BIN), str(directory), name], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, bufsize=1, env={**os.environ,"LOCAL_CHAT_TEST_SERVICE":service})
    def call(self, op, **args):
        self.process.stdin.write(json.dumps({'op': op, **args}) + '\n'); self.process.stdin.flush()
        ready, _, _ = select.select([self.process.stdout], [], [], 5)
        if not ready: raise RuntimeError('Peer response timed out')
        line = self.process.stdout.readline()
        if not line: raise RuntimeError(self.process.stderr.read())
        response = json.loads(line)
        if 'error' in response: raise RuntimeError(response['error'])
        return response['ok']
    def stop(self):
        try: self.call('quit'); self.process.wait(timeout=5)
        except Exception: self.process.kill(); self.process.wait()
def until(test, seconds=45):
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        if test(): return
        time.sleep(.2)
    raise AssertionError('Convergence deadline exceeded')
def run(count):
    with tempfile.TemporaryDirectory(prefix='local-chat-integration-') as folder:
        peers=[]
        service="_lct-"+uuid.uuid4().hex[:8]+"._tcp"
        try:
            peers=[Peer(pathlib.Path(folder)/str(i), f'Integration peer {i+1}',service) for i in range(count)]
            until(lambda: all(sum(p['status']=='connected' for p in peer.call('status')['peers']) == count-1 for peer in peers))
            for i, peer in enumerate(peers): peer.call('send', content=f'Synthetic LAN integration message {i}', code=i%2==0)
            until(lambda: all(len(peer.call('messages')) == count for peer in peers))
            room=peers[0].call('create',name='Synthetic integration room')['room']
            peers[0].call('create',name='Synthetic integration room')
            until(lambda: all(sum(r['name']=='Synthetic integration room' for r in peer.call('status')['rooms'])==2 for peer in peers))
            try:
                peers[1].call('rename',room=room,name='Unauthorized rename')
                raise AssertionError('Noncreator management was accepted')
            except RuntimeError as failure:
                assert 'creator' in str(failure)
            peers[0].call('rename',room=room,name='Renamed integration room')
            until(lambda: all(any(r['id']==room and r['name']=='Renamed integration room' for r in peer.call('status')['rooms']) for peer in peers))
            peers[0].call('archive',room=room)
            until(lambda: all(any(r['id']==room and r['archived'] for r in peer.call('status')['rooms']) for peer in peers))
            peers[0].call('restore',room=room)
            until(lambda: all(any(r['id']==room and not r['archived'] for r in peer.call('status')['rooms']) for peer in peers))
            old = peers.pop(); old.process.kill(); old.process.wait(timeout=5)
            peers[0].call('send', content='Synthetic offline catch-up message')
            restarted=Peer(pathlib.Path(folder)/str(count-1), 'Restarted integration peer',service); peers.append(restarted)
            until(lambda: all(len(peer.call('messages')) == count+1 for peer in peers))
            assert all(len({m['id'] for m in peer.call('messages')}) == count+1 for peer in peers)
            return {'processes':count,'discovery':'native Bonjour on selected physical interface','transport':'pinned TLS','concurrent_messages':count,'crash_restart_catchup':'pass','creator_room_management':'pass','duplicate_room_names':'pass','duplicates':0,'result':'pass'}
        except Exception:
            for i, peer in enumerate(peers):
                print(json.dumps({'diagnostic_peer':i,'status':peer.call('status'),'message_count':len(peer.call('messages'))}),flush=True)
            raise
        finally:
            for peer in peers:
                peer.stop()
                diagnostic=peer.process.stderr.read()
                if diagnostic: print(diagnostic,flush=True)
if __name__ == '__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--counts',nargs='+',type=int,default=[2,5,20]);args=parser.parse_args()
    for count in args.counts: print(json.dumps(run(count)),flush=True)
