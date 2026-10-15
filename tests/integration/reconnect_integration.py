#!/usr/bin/env python3
"""Exercise real discovery, reconnect, concurrent resets and retained catch-up.

Build the test-only local-chat-peer example first. Uses isolated identities and DNS-SD.
"""
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import tempfile
import uuid
from peer_integration import Peer, until


def run():
    with tempfile.TemporaryDirectory(prefix='local-chat-reconnect-') as folder:
        peers = []
        service = '_lct-' + uuid.uuid4().hex[:8] + '._tcp'
        try:
            peers = [Peer(Path(folder)/str(i), f'Reconnect peer {i}', service) for i in range(3)]
            def connected():
                return all(sum(p['status']=='connected' for p in peer.call('status')['peers'])==2 for peer in peers)
            until(connected)
            original = [peer.call('status') for peer in peers]
            room = peers[0].call('create', name='Reconnect verification')['room']
            until(lambda: all(any(r['id']==room for r in p.call('status')['rooms']) for p in peers))
            draft = 'Keep this draft\n\twith exact whitespace  '
            for i, peer in enumerate(peers):
                peer.call('send', room=room, content=f'Synthetic reconnect message {i}')
            until(lambda: all(len(p.call('messages', room=room))==3 for p in peers))
            peers[0].call('draft', room=room, content=draft)
            for _ in range(3):
                peers[0].call('reconnect')
                until(connected)
            with ThreadPoolExecutor(max_workers=3) as pool:
                for result in [pool.submit(p.call, 'reconnect') for p in peers]: result.result()
            until(connected)
            assert peers[0].call('draft', room=room)=={'content':draft,'code':True}
            offline = peers.pop()
            offline.process.kill(); offline.process.wait(timeout=5)
            peers[1].call('send', room=room, content='Synthetic retained catch-up')
            peers[0].call('reconnect')
            until(lambda: len(peers[0].call('messages', room=room))==4)
            peers.append(Peer(Path(folder)/'2', 'Restarted reconnect peer', service))
            until(connected)
            until(lambda: all(len(p.call('messages', room=room))==4 for p in peers))
            for before, peer in zip(original, peers):
                now=peer.call('status')
                assert (now['identity'],now['context'])==(before['identity'],before['context'])
                messages=peer.call('messages',room=room)
                assert len({m['id'] for m in messages})==4
            return {'result':'pass','processes':3,'manual_resets':7,
                    'concurrent_resets':'pass','discovery_and_pinned_tls':'pass',
                    'identity_context_rooms_and_draft_preserved':True,
                    'offline_catchup_without_duplicates':True,
                    'scope':'Isolated processes on one physical Mac; not a multi-computer LAN certification'}
        finally:
            for peer in peers: peer.stop()


if __name__=='__main__':
    print(json.dumps(run()),flush=True)
