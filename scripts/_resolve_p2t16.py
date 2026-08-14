import subprocess

files = subprocess.check_output(['git', 'diff', '--name-only', '--diff-filter=U']).decode().split()

resolutions = {
    'handbook/en/developer/execution-chain-status.md': [
        lambda h, m: h[:-1] + m[1:],
        lambda h, m: h[:6] + m[6:],
    ],
    'handbook/zh-CN/developer/execution-chain-status.md': [
        lambda h, m: h[:-1] + m[1:],
    ],
    'handbook/en/reference/capability-status.md': [
        lambda h, m: [
            '| Independent verification and Task acceptance | implemented; public C1 native-proven | production WorkspaceRead and RegisteredCheckRun reach registered independent verifiers; RegisteredCheck requires exact CAS Evidence, descriptor/file digests and clean safety observations before a passed report, checkpoint, one-time continuation authority and Loop `OBSERVE`; WorkspaceRead reaches a CAS-backed passed report and evidence-bound `COMPLETED` through the distinct daemon acceptance authority |'
        ],
    ],
    'handbook/zh-CN/reference/capability-status.md': [
        lambda h, m: [
            '| \u72ec\u7acb\u9a8c\u8bc1\u4e0e Task \u9a8c\u6536 | implemented\uff1b\u516c\u5171 C1 native-proven | \u751f\u4ea7 WorkspaceRead \u4e0e RegisteredCheckRun \u53ef\u5230\u8fbe\u767b\u8bb0\u7684\u72ec\u7acb verifier\uff1bRegisteredCheck \u53ea\u6709\u5728 CAS Evidence\u3001\u7cbe\u786e descriptor/file digest \u4e0e\u5168\u90e8\u5b89\u5168\u89c2\u5bdf\u901a\u8fc7\u540e\u624d\u4ea7\u751f passed report\u3001checkpoint\u3001\u4e00\u6b21\u6027 continuation authority \u4e0e Loop `OBSERVE`\uff1bWorkspaceRead \u518d\u7ecf\u72ec\u7acb daemon acceptance authority \u5b8c\u6210 evidence-bound `COMPLETED` |'
        ],
    ],
    'handbook/zh-CN/developer/task-pipeline.md': [
        'head',
    ],
}

for f, rlist in resolutions.items():
    with open(f, encoding='utf-8') as fh:
        lines = fh.read().split('\n')
    out = []
    i = 0
    ridx = 0
    while i < len(lines):
        if lines[i].startswith('<<<<<<<'):
            j = i + 1
            head = []
            while j < len(lines) and not lines[j].startswith('======='):
                head.append(lines[j])
                j += 1
            k = j + 1
            main = []
            while k < len(lines) and not lines[k].startswith('>>>>>>>'):
                main.append(lines[k])
                k += 1
            res = rlist[ridx]
            if callable(res):
                merged = res(head, main)
            elif res == 'head':
                merged = head
            elif res == 'main':
                merged = main
            out.extend(merged)
            ridx += 1
            i = k + 1
        else:
            out.append(lines[i])
            i += 1
    with open(f, 'w', encoding='utf-8') as fh:
        fh.write('\n'.join(out))
    print('resolved', f, ridx, 'regions')

leftover = []
for f in files:
    c = open(f, encoding='utf-8').read()
    if '<<<<<<<' in c or '=======' in c or '>>>>>>>' in c:
        leftover.append(f)
print('remaining conflict-marked files:', leftover)
