#!/usr/bin/env python3
# very VERY simple python tool to help add commands
import json

with open("commands.json", 'r') as file:
    full = json.load(file)
d = full["commands"]

"""
"bnb": {
  "value": {
    "StringResponse": "Brand new bot!"
  },
  "admin_only": false,
  "subcommands": {"sc":{"value":{"StringResponse":"Yes, we got subcommands!"}}},
  "hidden": true
},
"""

def to_bool(string):
    return string != '' and (string.lower()[0] in ['y', 't', '1'])

while True:
    name = input('command name: ').strip().lower()
    if name == '':
        break
    if name in d:
        print('already done')
        continue
    q = {}

    y = input('generic or string resp? empty for string: ').strip()
    if y == '':
        resp = input('What is the response? Empty cancels: ').strip()
        if resp == '':
            continue
        q["value"] = {"StringResponse": resp}
    else:
        resp = input('What is the mapping? Empty cancels: ').strip()
        if resp == '':
            continue
        q["value"] = {"Generic": resp}

    if to_bool(input("admin only? y for true, otherwise false: ").strip()):
        q["admin_only"] = True
    if to_bool(input("hidden? y for true, otherwise false: ").strip()):
        q["hidden"] = True

    d[name] = q

for k in d:
    if "admin_only" in d[k]:
        if d[k]["admin_only"] == False:
            del d[k]["admin_only"]
    if "hidden" in d[k]:
        if d[k]["hidden"] == False:
            del d[k]["hidden"]

full["commands"] = d

print('committing to file')
with open('commands.json', 'w', encoding='utf-8') as file:
    json.dump(full, file, indent=2, ensure_ascii=False)
