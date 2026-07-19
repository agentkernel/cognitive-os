import sys

with open(r"d:\agent-kernel\AgentOS-Architecture copy 2.md", "r", encoding="utf-8-sig") as f:
    text = f.read()

# Fix Edit 9: Add Context Plane row in section 4.3
# Find bold text for execution plane
m = text.find("**\u6267\u884c\u4e0e\u6570\u636e\u5e73\u9762**")
if m != -1:
    end = text.find("\n**", m + 15)
    if end == -1:
        end = text.find("\n\u5e73\u9762\u53ef\u4ee5", m)
    if end != -1:
        insert = "\n**\u4e0a\u4e0b\u6587\u5e73\u9762\uff08\u65b0\u589e\uff09**\u6cbb\u7406\u6ce8\u610f\u529b\u3001\u4fe1\u606f\u8d28\u91cf\u4e0e\u8ba4\u77e5\u6210\u672c\uff1a\n\u4e0a\u4e0b\u6587\u5982\u4f55\u6309 purpose\u3001\u6743\u9650\u3001\u9884\u7b97\u88c5\u5165\uff1f\u4fe1\u606f\u635f\u5931\u5982\u4f55\u6cbb\u7406\uff1f\n"
        text = text[:end] + insert + text[end:]
        print("Edit 9: OK - Context Plane added to sec 4.3")
    else:
        print("Edit 9: FAIL - could not find end of execution plane section")
else:
    print("Edit 9: FAIL - execution plane text not found")

# Fix Edit 22: Decision 10
d10 = text.find("### \u51b3\u7b56\u5341\uff1a\u5b66\u4e60\u53d7\u7ecf\u63a7\u53d1\u5e03")
if d10 != -1:
    d10_eol = text.find("\n", d10)
    if d10_eol != -1:
        d10_section_end = text.find("\n---", d10_eol)
        if d10_section_end == -1:
            d10_section_end = text.find("\n## 23.", d10_eol)
        if d10_section_end != -1:
            new_decisions = """
### \u51b3\u7b56\u5341\u4e00\uff1aContext Engineering \u4e0e State Engineering \u540c\u7b49\u91cd\u8981\uff08\u65b0\u589e\uff09
\u4e0a\u4e0b\u6587\u7684\u865a\u62df\u5316\u3001\u89e3\u6790\u3001\u8d28\u91cf\u6cbb\u7406\u548c\u751f\u547d\u5468\u671f\u7ba1\u7406\u662f AgentOS \u7684\u6838\u5fc3\u673a\u5236\uff0c\u4e0d\u662f\u5916\u56f4\u7684\u6570\u636e\u51c6\u5907\u5de5\u4f5c\u3002

### \u51b3\u7b56\u5341\u4e8c\uff1a\u4fe1\u606f\u635f\u5931\u5fc5\u987b\u53ef\u58f0\u660e\u3001\u53ef\u5ba1\u8ba1\uff08\u65b0\u589e\uff09
\u6240\u6709\u4e0a\u4e0b\u6587\u53d8\u6362\uff08\u6458\u8981\u3001\u538b\u7f29\u3001\u7b5b\u9009\uff09\u5fc5\u987b\u4fdd\u7559\u635f\u5931\u58f0\u660e\uff0c\u4e0d\u80fd\u9759\u9ed8\u4e22\u5f03\u4fe1\u606f\u3002
"""
            text = text[:d10_section_end] + new_decisions + text[d10_section_end:]
            print("Edit 22: OK - decisions 11 and 12 added")
        else:
            print("Edit 22: FAIL - end of section 22 not found")
    else:
        print("Edit 22: FAIL - no EOL after decision 10")
else:
    print("Edit 22: FAIL - decision 10 not found")

# Write back
with open(r"d:\agent-kernel\AgentOS-Architecture copy 2.md", "w", encoding="utf-8") as f:
    f.write("\uFEFF" + text)
print("File written: {} chars".format(len(text)))
