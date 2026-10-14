#!/usr/bin/env python3
"""Read-only macOS RSS/CPU sampling. Launch the app separately through Finder.
The RSS sum includes shared pages and is not a physical-footprint measurement.
"""
import argparse,json,pathlib,subprocess,time,re

def processes():
    lines=subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,rss=,time=,comm='],text=True).splitlines()
    result={}
    for line in lines:
        fields=line.strip().split(None,4)
        if len(fields)!=5:continue
        pid,parent,rss,cpu,command=fields
        match=re.fullmatch(r'(?:(\d+)-)?(\d+):(\d+(?:\.\d+)?)',cpu)
        if not match:continue
        days,minutes,seconds=match.groups();cpu_seconds=int(days or 0)*86400+int(minutes)*60+float(seconds)
        result[int(pid)]={'pid':int(pid),'parent':int(parent),'rss_kib':int(rss),'cpu_seconds':cpu_seconds,'command':command}
    return result
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('mode',choices=['baseline','sample']);parser.add_argument('--output',required=True);parser.add_argument('--seconds',type=int,default=30);args=parser.parse_args();path=pathlib.Path(args.output)
    if args.mode=='baseline':path.write_text(json.dumps({'before':list(processes())}));print('Baseline process IDs saved. Launch Local Chat through Finder.');raise SystemExit
    baseline=set(json.loads(path.read_text())['before']);first=processes()
    app_ids={pid for pid,p in first.items() if p['command'].endswith('/Local Chat.app/Contents/MacOS/local-chat')}
    assert len(app_ids)==1,f'Expected one Local Chat process, got {app_ids}'
    web_ids={pid for pid,p in first.items() if pid not in baseline and 'com.apple.WebKit.' in p['command']}
    selected=app_ids|web_ids;samples=[];start=time.monotonic();initial=first
    while time.monotonic()-start<args.seconds:
        current=processes();samples.append(sum(current[pid]['rss_kib'] for pid in selected if pid in current));time.sleep(1)
    final=processes();elapsed=time.monotonic()-start
    cpu=sum(max(0,final[pid]['cpu_seconds']-initial[pid]['cpu_seconds']) for pid in selected if pid in final and pid in initial)
    report={'seconds':elapsed,'average_cpu_percent_one_core':100*cpu/elapsed,'average_additive_rss_mib':sum(samples)/len(samples)/1024,'peak_additive_rss_mib':max(samples)/1024,'processes':[{'pid':pid,'command':first[pid]['command'],'initial_rss_mib':first[pid]['rss_kib']/1024}for pid in sorted(selected)],'attribution':'Local Chat plus WebKit processes created after the saved pre-launch baseline. No other app launch during sampling. RSS includes shared pages.'}
    path.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
